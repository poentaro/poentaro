import {
  type DecodeError,
  type FileName,
  type HistoryFilter,
  type HistoryRow,
  type Message,
  poentaroWasm,
  type Queue,
  type QueueSummary,
  type Resolution,
  type SendRequest,
  type SendResult,
  type StoredRecord,
} from '../../poentaroWasm'
import type { Scheduler } from './outboxScheduler'
import type { OutboxStore } from './outboxStore'

/** Only these keys belong to the queue codec; anything else in the store
 * (e.g. `base:<fileId>` snapshots) is left alone when persisting. */
const CODEC_KEY_PREFIXES = ['lane:', 'meta:']
/** Delivered messages kept for the history view; written by the looper but
 * not part of the queue codec. */
const SENT_KEY_PREFIX = 'sent:'

function isCodecKey(key: string): boolean {
  return CODEC_KEY_PREFIXES.some((prefix) => key.startsWith(prefix))
}

export interface OutboxSnapshot {
  queue: Queue
  summary: QueueSummary
  /** Set when the stored queue could not be decoded. The looper then neither
   * sends nor writes, so the stored data is never lost. */
  halted: DecodeError | null
}

export interface OutboxLooperDeps {
  store: OutboxStore
  /** Delivers one request. Decides nothing: classification is `sendMessage`'s job. */
  sendMessage(request: SendRequest): Promise<SendResult>
  /** Builds the wake-up scheduler around the looper's own wake callback. */
  createScheduler(onWake: () => void): Scheduler
  runAsLeader(task: (signal: AbortSignal) => Promise<void>): () => void
  now(): number
  random(): number
  /** Called after a successful send has been applied and persisted. A thrown
   * error is ignored so it never breaks the drain loop. */
  onAcknowledged?(request: SendRequest, revision: bigint): void | Promise<void>
  /** Called after a discard resolution has been persisted. Errors are ignored. */
  onDiscarded?(fileId: string): void | Promise<void>
  /** Tells the other tabs of this account that the stored queue changed. */
  changes?: OutboxChangeChannel
}

export interface OutboxChangeChannel {
  announce(): void
  listen(onChange: () => void): () => void
}

export interface OutboxLooper {
  start(): Promise<void>
  stop(): void
  enqueue(fileId: string, message: Message): Promise<void>
  resolve(fileId: string, resolution: Resolution): Promise<void>
  resolveSignedIn(): Promise<void>
  recordRevision(fileId: string, revision: bigint): Promise<void>
  subscribe(listener: (snapshot: OutboxSnapshot) => void): () => void
  snapshot(): OutboxSnapshot
  /** Queued and delivered messages that pass the filter, newest first. The
   * delivered ones are those last read from the store; `refreshHistory`
   * re-reads them (e.g. after another tab delivered something). */
  historyRows(fileNames: FileName[], filter: HistoryFilter): HistoryRow[]
  refreshHistory(): Promise<void>
  /** Resolves once the current drain (if any) has finished and persisted. */
  whenIdle(): Promise<void>
}

function decodeErrorOf(error: unknown): DecodeError | null {
  const payload = (error as { payload?: unknown } | null)?.payload
  if (
    typeof payload === 'object' &&
    payload !== null &&
    'tag' in payload &&
    (payload.tag === 'unsupported-version' || payload.tag === 'corrupt')
  ) {
    return payload as DecodeError
  }
  return null
}

export function createOutboxLooper(deps: OutboxLooperDeps): OutboxLooper {
  const { store, sendMessage, runAsLeader, now, random } = deps
  const nowMs = () => BigInt(now())

  let queue: Queue = { lanes: [], revisions: [] }
  let halted: DecodeError | null = null
  let started = false
  let isLeader = false
  let stopLeading: (() => void) | null = null
  let stopListening: (() => void) | null = null
  let persisted = new Map<string, string>()
  let sentRecords: StoredRecord[] = []
  let persistChain: Promise<void> = Promise.resolve()
  let drainPromise: Promise<void> | null = null
  let drainRequested = false
  const listeners = new Set<(snapshot: OutboxSnapshot) => void>()

  /** Until the stored queue has been read (which needs the wasm component),
   * there is nothing to summarise, and asking wasm would throw. */
  let loaded = false

  const snapshot = (): OutboxSnapshot => ({
    queue,
    summary: loaded
      ? poentaroWasm().outboxSummarize(queue, nowMs())
      : {
          lanes: [],
          totalPending: 0,
          lanesNeedingAttention: 0,
          hasUnsynced: false,
        },
    halted,
  })

  const notify = () => {
    if (listeners.size === 0) return
    const current = snapshot()
    for (const listener of listeners) listener(current)
  }

  const persistNow = async (): Promise<void> => {
    if (halted) return
    const records = poentaroWasm().outboxEncodeQueue(queue)
    const next = new Map(records.map((record) => [record.key, record.value]))
    const puts = records.filter(
      (record) => persisted.get(record.key) !== record.value,
    )
    const deletes = [...persisted.keys()].filter((key) => !next.has(key))
    if (puts.length === 0 && deletes.length === 0) return
    await store.apply({ puts, deletes })
    persisted = next
  }

  /** Serialised so concurrent mutations never interleave their store writes;
   * each run encodes whatever the queue is by the time it executes. */
  const persist = (): Promise<void> => {
    persistChain = persistChain.then(persistNow, persistNow)
    return persistChain
  }

  const load = async (): Promise<void> => {
    const records: StoredRecord[] = await store.loadAll()
    sentRecords = records.filter((record) =>
      record.key.startsWith(SENT_KEY_PREFIX),
    )
    const owned = records.filter((record) => isCodecKey(record.key))
    try {
      queue = poentaroWasm().outboxDecodeQueue(owned)
      halted = null
      persisted = new Map(owned.map((record) => [record.key, record.value]))
    } catch (error) {
      const decodeError = decodeErrorOf(error)
      if (!decodeError) throw error
      halted = decodeError
    }
  }

  /** Keeps the delivered message for the history view and prunes old ones.
   * Written before the queue drops the message: a crash in between resends
   * the message and rewrites the same record. */
  const recordSent = async (
    queueBefore: Queue,
    request: SendRequest,
    revision: bigint,
  ): Promise<void> => {
    const write = poentaroWasm().outboxRecordSent(
      queueBefore,
      sentRecords,
      request.fileId,
      request.messageId,
      revision,
      nowMs(),
    )
    const puts = write.put ? [write.put] : []
    if (puts.length === 0 && write.deletes.length === 0) return
    await store.apply({ puts, deletes: write.deletes })
    const deleted = new Set(write.deletes)
    sentRecords = [
      ...sentRecords.filter((record) => !deleted.has(record.key)),
      ...puts.filter((put) => !deleted.has(put.key)),
    ]
  }

  const wakeAtNextDue = () => {
    const next = poentaroWasm().outboxNextWakeMs(queue)
    scheduler?.wakeAt(next === undefined ? null : Number(next))
  }

  const runHook = async (hook: () => void | Promise<void>): Promise<void> => {
    try {
      await hook()
    } catch {
      // A failing hook must never break the drain loop.
    }
  }

  const drainOnce = async (signal: AbortSignal): Promise<void> => {
    while (!signal.aborted && !halted) {
      const begun = poentaroWasm().outboxBeginSend(queue, nowMs())
      queue = begun.queue
      await persist()
      notify()
      const { request } = begun
      if (!request) break
      const result = await sendMessage(request)
      if (result.tag === 'ok') {
        await runHook(() => recordSent(queue, request, result.val.revision))
      }
      queue = poentaroWasm().outboxOnResult(
        queue,
        request.fileId,
        request.messageId,
        result,
        nowMs(),
        random(),
      )
      await persist()
      notify()
      if (result.tag === 'ok') {
        await runHook(() => deps.onAcknowledged?.(request, result.val.revision))
      }
    }
    wakeAtNextDue()
  }

  let leaderSignal: AbortSignal | null = null

  const requestDrain = (): Promise<void> => {
    if (!isLeader || !leaderSignal) return Promise.resolve()
    if (drainPromise) {
      drainRequested = true
      return drainPromise
    }
    const signal = leaderSignal
    drainPromise = (async () => {
      try {
        do {
          drainRequested = false
          await drainOnce(signal)
        } while (drainRequested && !signal.aborted)
      } finally {
        drainPromise = null
      }
    })()
    return drainPromise
  }

  /** Exists only while started: stopping disposes it, starting rebuilds it. */
  let scheduler: Scheduler | null = null

  /** Mutations and the new leader's re-read of the store run one at a
   * time: a re-read that started before a mutation and answered after it
   * would otherwise put back the queue from before the mutation. */
  let exclusive: Promise<void> = Promise.resolve()
  const oneAtATime = (task: () => Promise<void>): Promise<void> => {
    const run = exclusive.then(task)
    exclusive = run.catch(() => undefined)
    return run
  }

  const mutate = (change: () => Queue): Promise<void> =>
    oneAtATime(async () => {
      if (halted) {
        throw new Error(
          'The outbox could not be read, so it will not accept new messages',
        )
      }
      // Another tab may have queued messages since this one last read the
      // store; build on what is stored so neither tab's messages are lost.
      if (!isLeader && loaded) await load()
      if (halted) throw new Error('The outbox could not be read')
      queue = change()
      await persist()
      notify()
      deps.changes?.announce()
      void requestDrain().catch(() => undefined)
    })

  const refreshSent = async (): Promise<void> => {
    sentRecords = (await store.loadAll()).filter((record) =>
      record.key.startsWith(SENT_KEY_PREFIX),
    )
  }

  /** A follower tab changed the stored queue: pick it up and deliver it. */
  const adoptExternalChange = async (): Promise<void> => {
    if (!isLeader) {
      await refreshSent()
      notify()
      return
    }
    await drainPromise
    await persistChain
    await load()
    notify()
    void requestDrain().catch(() => undefined)
  }

  return {
    async start() {
      if (started) return
      started = true
      scheduler = deps.createScheduler(() => {
        void requestDrain().catch(() => undefined)
      })
      await load()
      if (!started) return
      loaded = true
      notify()
      stopListening =
        deps.changes?.listen(() => {
          void adoptExternalChange().catch(() => undefined)
        }) ?? null
      stopLeading = runAsLeader(async (signal) => {
        // The previous leader may have changed the stored queue since we loaded.
        await oneAtATime(async () => {
          await load()
          notify()
        })
        isLeader = true
        leaderSignal = signal
        void requestDrain().catch(() => undefined)
        await new Promise<void>((resolve) => {
          if (signal.aborted) resolve()
          else signal.addEventListener('abort', () => resolve(), { once: true })
        })
        isLeader = false
        leaderSignal = null
      })
    },
    stop() {
      started = false
      stopLeading?.()
      stopLeading = null
      stopListening?.()
      stopListening = null
      isLeader = false
      scheduler?.dispose()
      scheduler = null
    },
    enqueue: (fileId, message) =>
      mutate(() =>
        poentaroWasm().outboxEnqueue(
          queue,
          fileId,
          message,
          crypto.randomUUID(),
          nowMs(),
        ),
      ),
    resolve: async (fileId, resolution) => {
      await mutate(() =>
        poentaroWasm().outboxResolve(queue, fileId, resolution),
      )
      if (resolution.tag === 'discard') {
        await runHook(() => deps.onDiscarded?.(fileId))
      }
    },
    resolveSignedIn: () =>
      mutate(() => poentaroWasm().outboxResolveSignedIn(queue)),
    recordRevision: (fileId, revision) =>
      mutate(() =>
        poentaroWasm().outboxRecordRevision(queue, fileId, revision),
      ),
    subscribe(listener) {
      listeners.add(listener)
      if (started) listener(snapshot())
      return () => listeners.delete(listener)
    },
    snapshot,
    historyRows: (fileNames, filter) =>
      loaded
        ? poentaroWasm().outboxHistoryRows(
            queue,
            sentRecords,
            fileNames,
            filter,
            nowMs(),
          )
        : [],
    async refreshHistory() {
      await refreshSent()
      notify()
    },
    async whenIdle() {
      await drainPromise
      await persistChain
    },
  }
}
