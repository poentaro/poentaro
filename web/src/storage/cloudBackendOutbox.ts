import type { SendRequest } from '../poentaroWasm'
import {
  callWorker,
  type createWorkerClient,
} from '../syncedShare/workerClient'
import type { CloudBackendError } from './cloudBackendTypes'
import { type BaseSnapshots, createBaseSnapshots } from './outbox/baseSnapshots'
import { createMemoryOutboxStore } from './outbox/memoryOutboxStore'
import { broadcastChanges } from './outbox/outboxChanges'
import { runAsLeader } from './outbox/outboxLock'
import {
  createOutboxLooper,
  type OutboxLooper,
  type OutboxSnapshot,
} from './outbox/outboxLooper'
import { createScheduler } from './outbox/outboxScheduler'
import {
  createIndexedDbOutboxStore,
  type OutboxStore,
} from './outbox/outboxStore'
import { sendMessage } from './outbox/sendMessage'
import type { SaveStatus } from './types'

export interface CloudOutboxConfig {
  client: ReturnType<typeof createWorkerClient>
  identityToken: string
  /** Signed-in account; scopes the on-device outbox. */
  account: string
  /** The file's current local name, for a `RestoreFile` message. */
  restoreName(fileId: string): string
}

/** The looper hooks that keep `bases` equal to the last text the server has. */
export function baseSnapshotHooks(bases: BaseSnapshots) {
  return {
    async onAcknowledged(request: SendRequest): Promise<void> {
      const { message, fileId } = request
      if (message.tag === 'save-content' || message.tag === 'create-file') {
        await bases.recordBase(fileId, message.val.content)
      } else if (message.tag === 'trash-file') {
        await bases.dropBase(fileId)
      }
    },
    onDiscarded: (fileId: string): Promise<void> => bases.dropBase(fileId),
  }
}

/** Records the listed text of every file as its base, except for files with
 * unsent messages: their base must stay the last text the server is known to
 * have for them. */
export async function recordListedBases(
  looper: OutboxLooper,
  bases: BaseSnapshots,
  files: ReadonlyArray<{ id: string; content: string }>,
): Promise<void> {
  const unsent = new Set(
    looper
      .snapshot()
      .queue.lanes.filter((lane) => lane.messages.length > 0)
      .map((lane) => lane.fileId),
  )
  for (const file of files) {
    if (!unsent.has(file.id)) await bases.recordBase(file.id, file.content)
  }
}

/** The looper and bases to use: injected ones (tests) or the production pair. */
export function createCloudOutbox(
  config: CloudOutboxConfig,
  injected: { looper?: OutboxLooper; bases?: BaseSnapshots },
): CloudOutbox {
  if (!injected.looper) {
    const production = createCloudOutboxLooper(config)
    return {
      looper: production.looper,
      bases: injected.bases ?? production.bases,
    }
  }
  return {
    looper: injected.looper,
    bases: injected.bases ?? createBaseSnapshots(createMemoryOutboxStore()),
  }
}

export interface CloudOutbox {
  looper: OutboxLooper
  bases: BaseSnapshots
}

/** The production looper: IndexedDB store, leader-tab lock, real timers.
 * The looper and the base snapshots share one store. */
function createCloudOutboxLooper(config: CloudOutboxConfig): CloudOutbox {
  const store: OutboxStore = createIndexedDbOutboxStore(config.account)
  const bases = createBaseSnapshots(store)
  const { client, identityToken, restoreName } = config
  const fetchServerContent = async (fileId: string): Promise<string> => {
    const response = await callWorker(
      client.POST('/files/list', { body: { identityToken } }),
    )
    return response.files.find((file) => file.id === fileId)?.content ?? ''
  }
  const looper = createOutboxLooper({
    ...baseSnapshotHooks(bases),
    store,
    sendMessage: (request) =>
      sendMessage(request, {
        client,
        identityToken,
        fetchServerContent,
        restoreName,
      }),
    createScheduler: (onWake) =>
      createScheduler({
        now: () => Date.now(),
        setTimer: (callback, delayMs) => setTimeout(callback, delayMs),
        clearTimer: (handle) => clearTimeout(handle),
        onWake,
      }),
    runAsLeader,
    changes: broadcastChanges(config.account),
    now: () => Date.now(),
    random: () => Math.random(),
  })
  return { looper, bases }
}

/** The save status the outbox implies, or null when it implies none. */
export function outboxSaveStatus(snapshot: OutboxSnapshot): SaveStatus | null {
  const reasons = snapshot.summary.lanes.map((lane) => lane.reason)
  if (snapshot.halted) return 'error'
  if (
    reasons.some((r) => ['needs-sign-in', 'needs-merge', 'failed'].includes(r))
  ) {
    return 'error'
  }
  if (reasons.some((r) => ['waiting-to-retry', 'possibly-stuck'].includes(r))) {
    return 'offline'
  }
  return reasons.length > 0 ? 'saving' : null
}

/** The first lane that needs attention, as the error it surfaces. */
export function outboxLaneError(
  snapshot: OutboxSnapshot,
): CloudBackendError | null {
  const { queue, summary } = snapshot
  for (const lane of queue.lanes) {
    if (lane.status.tag === 'needs-merge') {
      return {
        kind: 'conflict',
        currentRevision: Number(lane.status.val.currentRevision),
      }
    }
  }
  for (const lane of summary.lanes) {
    if (lane.reason === 'needs-sign-in') return { kind: 'auth' }
    if (lane.reason === 'failed') {
      return { kind: 'unknown', message: lane.failureReason ?? 'failed' }
    }
  }
  const waiting = summary.lanes.some(
    (lane) =>
      lane.reason === 'waiting-to-retry' || lane.reason === 'possibly-stuck',
  )
  return waiting ? { kind: 'network' } : null
}

/**
 * The text of the newest unsent save of each file, keyed by file id. A fresh
 * listing returns the server's text, which is older than these edits, so they
 * are laid over it to keep what the user typed.
 */
export function pendingContentByFileId(
  snapshot: OutboxSnapshot,
): Map<string, string> {
  const pending = new Map<string, string>()
  for (const lane of snapshot.queue.lanes) {
    const saves = lane.messages.flatMap(({ message }) =>
      message.tag === 'save-content' ? [message.val.content] : [],
    )
    const newest = saves.at(-1)
    if (newest !== undefined) pending.set(lane.fileId, newest)
  }
  return pending
}

/** Every unsent message with the file id of its lane, in the order they were
 * made, across lanes: a batch rename and a per-file rename of the same file
 * must land in that order. A create sorts before anything made in the same
 * millisecond. */
function pendingMessagesInOrder(snapshot: OutboxSnapshot) {
  return snapshot.queue.lanes
    .flatMap((lane) =>
      lane.messages.map(({ message, createdAtMs }) => ({
        fileId: lane.fileId,
        message,
        createdAtMs,
      })),
    )
    .sort(
      (a, b) =>
        Number(a.createdAtMs - b.createdAtMs) ||
        Number(b.message.tag === 'create-file') -
          Number(a.message.tag === 'create-file'),
    )
}

/** The name each file gets once its unsent creates, renames and batch renames
 * are delivered, keyed by file id. */
export function pendingNameByFileId(
  snapshot: OutboxSnapshot,
): Map<string, string> {
  const names = new Map<string, string>()
  for (const { fileId, message } of pendingMessagesInOrder(snapshot)) {
    if (message.tag === 'create-file') names.set(fileId, message.val.name)
    else if (message.tag === 'rename-file') names.set(fileId, message.val.to)
    else if (message.tag === 'rename-files') {
      for (const move of message.val.moves) names.set(move.fileId, move.to)
    }
  }
  return names
}

interface ListedFile {
  id: string
  name: string
  content: string
  revision: number
  trashedAt?: number | null
}

/**
 * The server's listing with every unsent create, rename, trash and restore
 * applied on top, in the order they were made. Without this a reload would
 * drop a file that was created offline and undo renames and deletions that
 * are still waiting to be sent.
 */
export function applyPendingStructure<File extends ListedFile>(
  files: File[],
  snapshot: OutboxSnapshot,
): File[] {
  const byId = new Map(files.map((file) => [file.id, file]))
  const pending = pendingMessagesInOrder(snapshot)
  for (const { fileId, message } of pending) {
    const existing = byId.get(fileId)
    switch (message.tag) {
      case 'create-file':
        if (!existing) {
          byId.set(fileId, {
            id: fileId,
            name: message.val.name,
            content: message.val.content,
            revision: 0,
            trashedAt: null,
          } as File)
        }
        break
      case 'rename-file':
        if (existing) byId.set(fileId, { ...existing, name: message.val.to })
        break
      case 'rename-files':
        for (const { fileId: movedId, to } of message.val.moves) {
          const moved = byId.get(movedId)
          if (moved) byId.set(movedId, { ...moved, name: to })
        }
        break
      case 'trash-file':
        if (existing) byId.set(fileId, { ...existing, trashedAt: 1 })
        break
      case 'restore-file':
        if (existing) byId.set(fileId, { ...existing, trashedAt: null })
        break
      case 'save-content':
        break
    }
  }
  return [...byId.values()]
}
