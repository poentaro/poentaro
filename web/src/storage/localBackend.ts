import {
  createFile,
  createFolder,
  deleteFile,
  deleteFolder,
  duplicateFile,
  type FileStoreState,
  importSharedFile,
  moveFolder,
  renameFile,
  restoreFile,
  updateActiveContent,
} from '../fileStore'
import {
  deserializeFileStore,
  readInitialFileStore,
  serializeFileStore,
} from '../fileStorePersistence'
import { importSharedFolderState } from '../sharedFolder'
import { ensureWasmInit } from '../wasmInit'
import type { SaveStatus, StorageBackend } from './types'

/**
 * Reads the initial `FileStoreState` from `localStorage`. Exposed separately
 * (in addition to being used by `localBackend.load`) because
 * `useStorageBackend` needs a *synchronous* initial value for
 * `useLocalStorage`'s initializer — `StorageBackend.load` is async to
 * accommodate future network-backed backends, but the local backend's
 * underlying read is always instant, so there is no behavior change in
 * seeding the state this way.
 */
export function readInitialStoreSync(): FileStoreState {
  return readInitialFileStore()
}

/**
 * Deserializes a raw `localStorage` value into a `FileStoreState`. Used
 * directly as `useLocalStorage`'s `deserializer` for the same
 * synchronous-seeding reason as `readInitialStoreSync`.
 */
export function deserializeStoreSync(raw: string): FileStoreState {
  return deserializeFileStore(raw)
}

/** `useLocalStorage`'s `serializer`: the on-disk shape stays plain strings. */
export function serializeStoreSync(state: FileStoreState): string {
  return serializeFileStore(state)
}

/** The store operations run in Rust (wasm), so wait for the component first. */
function afterWasm<T>(run: () => T): Promise<T> {
  return ensureWasmInit().then(run)
}

/**
 * Thin async adapter over `fileStore.ts`'s pure, synchronous functions.
 * `saveContent` is a no-op: `useLocalStorage` (used by `useStorageBackend`)
 * already persists to `localStorage` on every state change, so there is
 * nothing left to explicitly save.
 */
export const localBackend: StorageBackend = {
  kind: 'local',

  load: () => Promise.resolve(readInitialStoreSync()),

  createFile: (state, folder, name) =>
    afterWasm(() => createFile(state, folder, name)),

  createFolder: (state, parent, name) =>
    afterWasm(() => createFolder(state, parent, name)),

  duplicateFile: (state) => afterWasm(() => duplicateFile(state)),

  importFile: (state, filename, content) =>
    Promise.resolve(importSharedFile(state, filename, content)),

  importFolder: (state, snapshot) =>
    afterWasm(() => importSharedFolderState(state, snapshot)),

  renameFile: (state, from, to) => afterWasm(() => renameFile(state, from, to)),

  deleteFile: (state, name) => afterWasm(() => deleteFile(state, name)),

  moveFolder: (state, from, to) =>
    afterWasm(() => {
      const result = moveFolder(state, from, to)
      return result.ok ? result.state : state
    }),

  deleteFolder: (state, folder) => afterWasm(() => deleteFolder(state, folder)),

  restoreFile: (state, name) => afterWasm(() => restoreFile(state, name)),

  updateActiveContent: (state, content) => updateActiveContent(state, content),

  saveContent: () => Promise.resolve(),

  status: (): SaveStatus => 'idle',
}
