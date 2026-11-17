import { poentaroWasm } from '../../poentaroWasm'
import type { BaseSnapshots } from './baseSnapshots'
import type { OutboxLooper, OutboxSnapshot } from './outboxLooper'

export const MERGED_NOTICE = 'Merged with changes from another device'

/** What the user needs to settle a conflict the automatic merge could not. */
export interface ConflictDetails {
  fileId: string
  theirs: string
  mine: string
  mergedWithMarkers: string
  serverRevision: bigint
}

export interface ServerFile {
  revision: number
  content: string
}

export interface ConflictFlowDeps {
  looper: OutboxLooper
  bases: BaseSnapshots
  fetchServerFile(fileId: string): Promise<ServerFile | undefined>
  onNotice(message: string): void
}

export interface ConflictFlow {
  conflicts(): ConflictDetails[]
  subscribe(listener: (conflicts: ConflictDetails[]) => void): () => void
  stop(): void
}

/** Without a base there is nothing to merge against, so both sides are shown. */
function markersWithoutBase(mine: string, theirs: string): string {
  return `<<<<<<< mine\n${mine}\n=======\n${theirs}\n>>>>>>> theirs\n`
}

/** The text of the save waiting at the head of a lane, if that is what it holds. */
function headSaveContent(
  snapshot: OutboxSnapshot,
  fileId: string,
): string | undefined {
  const head = snapshot.queue.lanes.find((lane) => lane.fileId === fileId)
    ?.messages[0]?.message
  return head?.tag === 'save-content' ? head.val.content : undefined
}

function needsMergeFileIds(snapshot: OutboxSnapshot): string[] {
  return snapshot.queue.lanes
    .filter((lane) => lane.status.tag === 'needs-merge')
    .map((lane) => lane.fileId)
}

/**
 * Settles `NeedsMerge` lanes: a clean three-way merge is saved at once, a
 * conflicting one is kept for the user. The merge itself is a wasm call;
 * this only fetches the inputs and applies the outcome. Lanes of other files
 * keep draining meanwhile, since nothing here blocks the looper.
 */
export function createConflictFlow(deps: ConflictFlowDeps): ConflictFlow {
  const { looper, bases, fetchServerFile, onNotice } = deps
  const conflicts = new Map<string, ConflictDetails>()
  const inProgress = new Set<string>()
  const listeners = new Set<(conflicts: ConflictDetails[]) => void>()

  const publish = () => {
    const current = [...conflicts.values()]
    for (const listener of listeners) listener(current)
  }

  const settle = async (fileId: string): Promise<void> => {
    const mine = headSaveContent(looper.snapshot(), fileId)
    if (mine === undefined) return
    const server = await fetchServerFile(fileId)
    const theirs = server?.content ?? ''
    const serverRevision = BigInt(server?.revision ?? 0)
    const base = await bases.readBase(fileId)
    const outcome =
      base === undefined
        ? undefined
        : poentaroWasm().mergeThreeWay(base, mine, theirs)
    if (outcome?.tag === 'clean') {
      // The user kept typing while the server copy was fetched: merge again.
      if (headSaveContent(looper.snapshot(), fileId) !== mine) {
        return settle(fileId)
      }
      await looper.resolve(fileId, {
        tag: 'merged-and-save',
        val: { content: outcome.val.text, serverRevision },
      })
      onNotice(MERGED_NOTICE)
      return
    }
    conflicts.set(fileId, {
      fileId,
      theirs,
      mine,
      mergedWithMarkers:
        outcome?.tag === 'conflicted'
          ? outcome.val.text
          : markersWithoutBase(mine, theirs),
      serverRevision,
    })
    publish()
  }

  const handle = (snapshot: OutboxSnapshot) => {
    const waiting = needsMergeFileIds(snapshot)
    const stale = [...conflicts.keys()].filter((id) => !waiting.includes(id))
    for (const fileId of stale) conflicts.delete(fileId)
    if (stale.length > 0) publish()
    for (const fileId of waiting) {
      if (conflicts.has(fileId) || inProgress.has(fileId)) continue
      inProgress.add(fileId)
      void settle(fileId)
        .catch(() => undefined)
        .finally(() => inProgress.delete(fileId))
    }
  }

  const unsubscribe = looper.subscribe(handle)
  return {
    conflicts: () => [...conflicts.values()],
    subscribe(listener) {
      listeners.add(listener)
      return () => listeners.delete(listener)
    },
    stop: unsubscribe,
  }
}
