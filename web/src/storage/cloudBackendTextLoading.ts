import type { FileStoreState } from '../fileStore'
import { loadedFileText } from '../fileText'
import { pendingContentByFileId } from './cloudBackendOutbox'
import type { ServerFile } from './cloudBackendTypes'
import type { FileTextCache } from './fileTextCache'
import type { BaseSnapshots } from './outbox/baseSnapshots'
import type { OutboxLooper } from './outbox/outboxLooper'

export interface TextLoadingDependencies {
  looper: OutboxLooper
  bases: BaseSnapshots
  texts: FileTextCache
  fetchServerFile(fileId: string): Promise<ServerFile | undefined>
  /** The server revision each listed file had at the last `load()`. */
  revisionById: ReadonlyMap<string, number>
  /** The id of the file `name` in the state's active files or bin. */
  idOf(state: FileStoreState, name: string): string
}

/** The text of one file, from unsent edits, the on-device cache or the
 * server, in that order. Calls for one file share one request. */
export function createTextLoader(deps: TextLoadingDependencies) {
  const inFlight = new Map<string, Promise<string>>()

  async function readText(fileId: string): Promise<string> {
    const pending = pendingContentByFileId(deps.looper.snapshot()).get(fileId)
    if (pending !== undefined) return pending
    const revision = deps.revisionById.get(fileId)
    if (revision !== undefined) {
      const cached = await deps.texts
        .read(fileId, revision)
        .catch(() => undefined)
      if (cached !== undefined) return cached
    }
    const file = await deps.fetchServerFile(fileId)
    if (file === undefined)
      throw new Error('The file is no longer on the server')
    // The server's text is the base for merging, unless unsent edits made
    // while it was loading are already waiting on an older one.
    const unsent = deps.looper
      .snapshot()
      .queue.lanes.some(
        (lane) => lane.fileId === fileId && lane.messages.length > 0,
      )
    if (!unsent) await deps.bases.recordBase(fileId, file.content)
    return file.content
  }

  function shared(fileId: string): Promise<string> {
    const existing = inFlight.get(fileId)
    if (existing) return existing
    const started = readText(fileId).finally(() => inFlight.delete(fileId))
    inFlight.set(fileId, started)
    return started
  }

  return {
    /** `state` with the text of the file `name` loaded; the same state when
     * it already is. */
    async ensureLoaded(
      state: FileStoreState,
      name: string,
    ): Promise<FileStoreState> {
      const inFiles = name in state.userFiles
      const entry = inFiles ? state.userFiles[name] : state.bin[name]
      if (entry === undefined || entry.tag === 'loaded') return state
      const text = loadedFileText(await shared(deps.idOf(state, name)))
      return inFiles
        ? { ...state, userFiles: { ...state.userFiles, [name]: text } }
        : { ...state, bin: { ...state.bin, [name]: text } }
    },
  }
}
