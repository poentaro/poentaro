import {
  DEMO_FILE_NAMES,
  type FileStoreState,
  fileIdForName,
  isReadOnlyFile,
  loadedFileText,
  loadedText,
  NOT_LOADED,
  createFile as pureCreateFile,
  deleteFile as pureDeleteFile,
  duplicateFile as pureDuplicateFile,
  importSharedFile as pureImportSharedFile,
  renameFile as pureRenameFile,
  restoreFile as pureRestoreFile,
  updateActiveContent as pureUpdateActiveContent,
} from '../fileStore'
import {
  type AppFileCalls,
  appClient,
  type ClientOptions,
} from '../protocol/clients'
import { ensureWasmInit } from '../wasmInit'
import { createFolderOperations } from './cloudBackendFolders'
import { createServerReads } from './cloudBackendListing'
import { addedName } from './cloudBackendNaming'
import {
  applyPendingStructure,
  createCloudOutbox,
  outboxLaneError,
  outboxSaveStatus,
  pendingContentByFileId,
  pendingNameByFileId,
  recordServerRevisions,
} from './cloudBackendOutbox'
import { createPlaceholders } from './cloudBackendPlaceholders'
import { createTextLoader } from './cloudBackendTextLoading'
import type {
  CloudBackend,
  CloudBackendConfig,
  CloudBackendError,
  LocalRename,
} from './cloudBackendTypes'
import { createVersionCalls } from './cloudBackendVersions'
import type { FileTextCache } from './fileTextCache'
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
 * `StorageBackend` implementation backed by `poentaro app`'s `files/*` and
 * `folders/*` (D1-backed). Every structural op calls the matching pure
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
  /** `poentaro app`'s file calls; the worker at `workerHost` by default. */
  app?: (options: ClientOptions) => Promise<AppFileCalls>
  looper?: OutboxLooper
  bases?: BaseSnapshots
  texts?: FileTextCache
  ensureWasm?: () => Promise<void>
}

export function createCloudBackend(
  config: CloudBackendConfig,
  dependencies: CloudBackendDependencies = {},
): CloudBackend {
  const app =
    dependencies.app ??
    ((options: ClientOptions) =>
      appClient(config.token, { host: config.workerHost, ...options }))

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

  const { looper, bases, texts } = createCloudOutbox(
    { app, account: config.account },
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
  const { fetchServerFile, listFiles } = createServerReads(() => app({}), texts)

  const revisionById = new Map<string, number>()
  const textLoader = createTextLoader({
    looper,
    bases,
    texts,
    fetchServerFile,
    revisionById,
    idOf: (state, name) =>
      name in state.userFiles
        ? activeFileId(state, name)
        : trashedFileId(state, name),
  })

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
    // A file whose text has not loaded must never be saved as empty.
    const content = loadedText(state, state.active)
    if (content === undefined) return
    await whenReady()
    await looper.enqueue(fileIdForName(state, state.active), {
      tag: 'save-content',
      val: { content },
    })
  }

  /** Queues the creation of the file `nextState` added relative to `state`
   * (create, import, duplicate); the file already exists locally. */
  async function createRemoteFile(
    state: FileStoreState,
    nextState: FileStoreState,
  ): Promise<FileStoreState> {
    const name = addedName(state, nextState)
    if (!name) return nextState
    const content = loadedText(nextState, name)
    // A new file is always loaded; there is nothing to create without text.
    if (content === undefined) return nextState
    const id = fileIdForName(nextState, name)
    activeIdByName.set(name, id)
    await whenReady()
    await looper.enqueue(id, {
      tag: 'create-file',
      val: { name, content },
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
      const response = { files: await listFiles() }
      const userFiles: FileStoreState['userFiles'] = {}
      const bin: FileStoreState['bin'] = {}
      const fileIds: Record<string, string> = {}
      activeIdByName.clear()
      trashedIdByName.clear()
      const { files: listed, folders } = placeholders.classify(
        pendingSnapshot.halted
          ? response.files
          : applyPendingStructure(response.files, pendingSnapshot),
      )
      revisionById.clear()
      for (const file of listed) {
        fileIds[file.name] = file.id
        revisionById.set(file.id, file.revision)
        if (file.trashedAt === null) {
          userFiles[file.name] = NOT_LOADED
          activeIdByName.set(file.name, file.id)
        } else {
          bin[file.name] = NOT_LOADED
          trashedIdByName.set(file.name, file.id)
        }
      }
      // An unreadable outbox must not stop the files from loading.
      if (!looper.snapshot().halted) {
        await recordServerRevisions(looper, response.files)
      }
      for (const [id, content] of pendingContentByFileId(pendingSnapshot)) {
        const name = [...activeIdByName].find(([, value]) => value === id)?.[0]
        if (name !== undefined) userFiles[name] = loadedFileText(content)
      }
      return {
        active: DEMO_FILE_NAMES[0] ?? '',
        userFiles,
        bin,
        fileIds,
        ...(folders.length > 0 ? { folders } : {}),
      }
    },

    ensureLoaded: textLoader.ensureLoaded,

    createFile: (state, folder, name) =>
      ensureWasm().then(() =>
        createRemoteFile(state, pureCreateFile(state, folder, name)),
      ),
    importFile: (state, filename, content) =>
      createRemoteFile(state, pureImportSharedFile(state, filename, content)),

    async duplicateFile(state) {
      await ensureWasm()
      // A copy of a file that was never read would be empty.
      const loaded = await textLoader.ensureLoaded(state, state.active)
      return createRemoteFile(loaded, pureDuplicateFile(loaded))
    },

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

    ...createVersionCalls(() => app({})),

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
