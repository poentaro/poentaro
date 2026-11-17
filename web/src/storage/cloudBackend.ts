import {
  DEMO_FILE_NAMES,
  type FileStoreState,
  fileContent,
  fileIdForName,
  isReadOnlyFile,
  createFile as pureCreateFile,
  deleteFile as pureDeleteFile,
  duplicateFile as pureDuplicateFile,
  importSharedFile as pureImportSharedFile,
  renameFile as pureRenameFile,
  restoreFile as pureRestoreFile,
  updateActiveContent as pureUpdateActiveContent,
} from '../fileStore'
import { callWorker, createWorkerClient } from '../syncedShare/workerClient'
import { ensureWasmInit } from '../wasmInit'
import { createFolderOperations } from './cloudBackendFolders'
import { addedName } from './cloudBackendNaming'
import {
  applyPendingStructure,
  createCloudOutbox,
  outboxLaneError,
  outboxSaveStatus,
  pendingContentByFileId,
  pendingNameByFileId,
  recordListedBases,
} from './cloudBackendOutbox'
import { createPlaceholders } from './cloudBackendPlaceholders'
import type {
  CloudBackend,
  CloudBackendConfig,
  CloudBackendError,
  LocalRename,
} from './cloudBackendTypes'
import { createVersionCalls } from './cloudBackendVersions'
import type { BaseSnapshots } from './outbox/baseSnapshots'
import { type ConflictFlow, createConflictFlow } from './outbox/conflictFlow'
import type { OutboxLooper, OutboxSnapshot } from './outbox/outboxLooper'
import { watchForEndedSession } from './outbox/watchForEndedSession'
import type { SaveStatus } from './types'

export type {
  CloudBackend,
  CloudBackendConfig,
  CloudBackendError,
} from './cloudBackendTypes'

/**
 * `StorageBackend` implementation backed by `crates/live-share-worker`'s
 * `/files/*` routes (D1-backed). Every structural op calls the matching pure
 * transform from `fileStore.ts`, applies it to local state at once and queues
 * the HTTP call in the outbox (`outbox/`), which delivers it in the
 * background; the transform's resulting `FileStoreState` is returned
 * (reconciled by the caller via `mergeBackendResult`). A `name_taken` answer
 * is resolved by the outbox rewriting the queued name; this backend then
 * reports the local rename through `onLocalRename`. Helpers live in
 * `cloudBackendNaming.ts`, `cloudBackendTypes.ts`, `cloudBackendFolders.ts`
 * and `cloudBackendPlaceholders.ts` (empty folders).
 */
export interface CloudBackendDependencies {
  looper?: OutboxLooper
  bases?: BaseSnapshots
  ensureWasm?: () => Promise<void>
}

export function createCloudBackend(
  config: CloudBackendConfig,
  dependencies: CloudBackendDependencies = {},
): CloudBackend {
  const identityToken = config.token
  const client = createWorkerClient(config.workerHost)

  const renameListeners = new Set<(change: LocalRename) => void>()
  // `FileStoreState.fileIds` is one flat map keyed by display name, shared
  // across `userFiles` and `bin`, and a listing can hold an active and a
  // trashed row under one name (e2e manufactures that), so each bucket's ids
  // are tracked separately: `renameFile`/`deleteFile` and `restoreFile` never
  // resolve the wrong row. Kept in sync alongside every structural op.
  const activeIdByName = new Map<string, string>()
  const trashedIdByName = new Map<string, string>()

  /** Id of the *active* file named `name`, preferring the bucket-aware
   * `activeIdByName` (accurate even when a same-named trashed row exists)
   * over the ambiguous `state.fileIds`, which is only consulted as a
   * fallback (e.g. before the first `load()` has populated the map). */
  function activeFileId(state: FileStoreState, name: string): string {
    return activeIdByName.get(name) ?? fileIdForName(state, name)
  }

  /** Id of the *trashed* file named `name` -- see `activeFileId` above. */
  function trashedFileId(state: FileStoreState, name: string): string {
    return trashedIdByName.get(name) ?? fileIdForName(state, name)
  }

  /** The current local name of `fileId`, for a `RestoreFile` message. */
  function trashedNameOf(fileId: string): string {
    const named = [...activeIdByName, ...trashedIdByName].find(
      ([, id]) => id === fileId,
    )
    return named?.[0] ?? placeholders.nameOf(fileId) ?? ''
  }

  const { looper, bases } = createCloudOutbox(
    {
      client,
      identityToken,
      account: config.account,
      restoreName: trashedNameOf,
    },
    dependencies,
  )
  const ensureWasm = dependencies.ensureWasm ?? ensureWasmInit
  let ready: Promise<void> | null = null
  const placeholders = createPlaceholders(looper, () => whenReady())
  const folderOperations = createFolderOperations({
    ensureWasm,
    whenReady: () => whenReady(),
    looper,
    placeholders,
    activeFileId,
    activeIdByName,
    trashedIdByName,
  })
  async function fetchServerFile(fileId: string) {
    const response = await callWorker(
      client.POST('/files/list', { body: { identityToken } }),
    )
    const file = response.files.find(({ id }) => id === fileId)
    return file && { revision: file.revision, content: file.content }
  }

  const noticeListeners = new Set<(message: string) => void>()
  let conflictFlow: ConflictFlow | null = null
  let startGeneration = 0
  let stopReconciling: (() => void) | null = null
  let stopWatchingSession: (() => void) | null = null

  /** Initializes wasm and starts the looper, once. */
  function whenReady(): Promise<void> {
    if (ready) return ready
    const generation = startGeneration
    ready = ensureWasm().then(async () => {
      // Disposed while wasm was loading: do not take the lock.
      if (generation !== startGeneration) return
      await looper.start()
      // A backend only exists with a valid sign-in, so lanes that stopped
      // for sign-in can go again.
      if (!looper.snapshot().halted) await looper.resolveSignedIn()
      stopReconciling = looper.subscribe(reconcileLocalNames)
      stopWatchingSession = watchForEndedSession(looper, () =>
        config.onSessionEnded?.(),
      )
      conflictFlow = createConflictFlow({
        looper,
        bases,
        fetchServerFile,
        onNotice: (message) => {
          for (const listener of noticeListeners) listener(message)
        },
      })
    })
    return ready
  }

  /** The name the last queued `CreateFile`/`RenameFile`/`RenameFiles` of
   * each file asks for. When the outbox rewrote one after a `name_taken`
   * answer, it differs from the local name, which is then renamed to match. */
  function reconcileLocalNames(snapshot: OutboxSnapshot): void {
    for (const [fileId, wanted] of pendingNameByFileId(snapshot)) {
      const current = [...activeIdByName].find(([, id]) => id === fileId)?.[0]
      if (current === undefined || wanted === current) continue
      activeIdByName.delete(current)
      activeIdByName.set(wanted, fileId)
      for (const listener of renameListeners) {
        listener({ fileId, from: current, to: wanted })
      }
    }
  }

  /** Queues the active file's content in the outbox and returns once it is
   * persisted locally -- delivery happens in the background. */
  async function saveContent(state: FileStoreState): Promise<void> {
    if (isReadOnlyFile(state.active)) return
    await whenReady()
    await looper.enqueue(fileIdForName(state, state.active), {
      tag: 'save-content',
      val: { content: fileContent(state, state.active) },
    })
  }

  /** Tells the outbox each file's server revision, except for files with
   * unsent messages: their recorded revision is what conflict detection
   * compares against. */
  async function recordServerRevisions(
    files: ReadonlyArray<{ id: string; revision: number }>,
  ): Promise<void> {
    const pending = new Set(
      looper
        .snapshot()
        .queue.lanes.filter((lane) => lane.messages.length > 0)
        .map((lane) => lane.fileId),
    )
    for (const file of files) {
      if (!pending.has(file.id)) {
        await looper.recordRevision(file.id, BigInt(file.revision))
      }
    }
  }

  /** Queues the creation of the file `nextState` added relative to `state`
   * (create, import, duplicate); the file already exists locally. */
  async function createRemoteFile(
    state: FileStoreState,
    nextState: FileStoreState,
  ): Promise<FileStoreState> {
    const name = addedName(state, nextState)
    if (!name) return nextState
    const id = fileIdForName(nextState, name)
    activeIdByName.set(name, id)
    await whenReady()
    await looper.enqueue(id, {
      tag: 'create-file',
      val: { name, content: nextState.userFiles[name] ?? '' },
    })
    return nextState
  }

  return {
    kind: 'cloud',

    async load(): Promise<FileStoreState> {
      await whenReady()
      // Taken before the listing request: a message delivered while the
      // listing is in flight leaves the queue, yet the listing may still carry
      // the older text, so the edit must be laid over it from this snapshot.
      const pendingSnapshot = looper.snapshot()
      const response = await callWorker(
        client.POST('/files/list', { body: { identityToken } }),
      )
      const userFiles: Record<string, string> = {}
      const bin: Record<string, string> = {}
      const fileIds: Record<string, string> = {}
      activeIdByName.clear()
      trashedIdByName.clear()
      const { files: listed, folders } = placeholders.classify(
        pendingSnapshot.halted
          ? response.files
          : applyPendingStructure(response.files, pendingSnapshot),
      )
      for (const file of listed) {
        fileIds[file.name] = file.id
        if (file.trashedAt === null) {
          userFiles[file.name] = file.content
          activeIdByName.set(file.name, file.id)
        } else {
          bin[file.name] = file.content
          trashedIdByName.set(file.name, file.id)
        }
      }
      // An unreadable outbox must not stop the files from loading.
      if (!looper.snapshot().halted) {
        await recordServerRevisions(response.files)
        await recordListedBases(looper, bases, response.files)
      }
      for (const [id, content] of pendingContentByFileId(pendingSnapshot)) {
        const name = [...activeIdByName].find(([, value]) => value === id)?.[0]
        if (name !== undefined) userFiles[name] = content
      }
      return {
        active: DEMO_FILE_NAMES[0] ?? '',
        userFiles,
        bin,
        fileIds,
        ...(folders.length > 0 ? { folders } : {}),
      }
    },

    createFile: (state, folder, name) =>
      ensureWasm().then(() =>
        createRemoteFile(state, pureCreateFile(state, folder, name)),
      ),
    importFile: (state, filename, content) =>
      createRemoteFile(state, pureImportSharedFile(state, filename, content)),

    duplicateFile: (state) =>
      ensureWasm().then(() =>
        createRemoteFile(state, pureDuplicateFile(state)),
      ),

    async renameFile(
      state: FileStoreState,
      from: string,
      to: string,
    ): Promise<FileStoreState> {
      await ensureWasm()
      const nextState = pureRenameFile(state, from, to)
      const newName = addedName(state, nextState)
      if (!newName) return nextState
      const id = activeFileId(state, from)
      activeIdByName.delete(from)
      activeIdByName.set(newName, id)
      await whenReady()
      await looper.enqueue(id, { tag: 'rename-file', val: { to: newName } })
      return nextState
    },

    async deleteFile(
      state: FileStoreState,
      name: string,
    ): Promise<FileStoreState> {
      await ensureWasm()
      const nextState = pureDeleteFile(state, name)
      if (nextState === state) return nextState
      const id = activeFileId(state, name)
      activeIdByName.delete(name)
      trashedIdByName.set(name, id)
      await whenReady()
      await looper.enqueue(id, { tag: 'trash-file' })
      return nextState
    },

    ...folderOperations,

    async restoreFile(
      state: FileStoreState,
      name: string,
    ): Promise<FileStoreState> {
      await ensureWasm()
      const nextState = pureRestoreFile(state, name)
      const newName = addedName(state, nextState)
      if (!newName) return nextState
      const id = trashedFileId(state, name)
      trashedIdByName.delete(name)
      activeIdByName.set(newName, id)
      await whenReady()
      await looper.enqueue(id, { tag: 'restore-file' })
      return nextState
    },

    updateActiveContent: (
      state: FileStoreState,
      content: string,
    ): FileStoreState => pureUpdateActiveContent(state, content),

    saveContent,

    fetchServerFile,

    ...createVersionCalls(client, identityToken),

    conflicts: () => conflictFlow,

    onNotice(listener: (message: string) => void): () => void {
      noticeListeners.add(listener)
      return () => noticeListeners.delete(listener)
    },

    outbox: () => looper,
    dispose() {
      stopReconciling?.()
      stopReconciling = null
      stopWatchingSession?.()
      stopWatchingSession = null
      conflictFlow?.stop()
      conflictFlow = null
      startGeneration += 1
      looper.stop()
      ready = null
    },
    bases: () => bases,

    status: (): SaveStatus => outboxSaveStatus(looper.snapshot()) ?? 'idle',

    lastError: (): CloudBackendError | null =>
      outboxLaneError(looper.snapshot()),

    onLocalRename(listener: (change: LocalRename) => void): () => void {
      renameListeners.add(listener)
      return () => renameListeners.delete(listener)
    },

    async forceOverwrite(state: FileStoreState): Promise<void> {
      const id = fileIdForName(state, state.active)
      const conflict = looper
        .snapshot()
        .queue.lanes.find(
          (lane) => lane.fileId === id && lane.status.tag === 'needs-merge',
        )
      if (conflict?.status.tag === 'needs-merge') {
        await looper.recordRevision(id, conflict.status.val.currentRevision)
        await looper.resolve(id, { tag: 'retry' })
        return
      }
      return saveContent(state)
    },
  }
}
