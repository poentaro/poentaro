import type { FileStoreState } from '../fileStore'
import type { FolderSnapshot } from '../poentaroWasm'

/**
 * Persistence/save-indicator state exposed by a backend. `'offline'` and
 * `'error'` are only reachable once network-backed backends (e.g. cloud)
 * exist; the local backend is always `'idle'`.
 */
export type SaveStatus = 'idle' | 'saving' | 'saved' | 'error' | 'offline'

/**
 * A pluggable storage backend for the file store. The browser
 * `localStorage` backend and the D1-backed `'cloud'` backend
 * (`cloudBackend.ts`) both implement this interface, so `FileStoreState`
 * mutations are backend-agnostic from the hook/UI layer's perspective.
 *
 * `updateActiveContent` is a plain synchronous state setter with no
 * persistence side effect; `saveContent` is the explicit call that persists
 * the active file's content. Debounce ownership lives in the hook layer
 * (`useStorageBackend`), not here, to avoid two independent debounce
 * mechanisms with unclear ownership.
 */
export interface StorageBackend {
  readonly kind: 'local' | 'cloud'
  load(): Promise<FileStoreState>
  /** New file `name` in `folder` (top level when omitted). */
  createFile(
    state: FileStoreState,
    folder: string | undefined,
    name: string,
  ): Promise<FileStoreState>
  /** New empty folder `name` inside `parent` (top level when omitted). */
  createFolder(
    state: FileStoreState,
    parent: string | undefined,
    name: string,
  ): Promise<FileStoreState>
  duplicateFile(state: FileStoreState): Promise<FileStoreState>
  importFile(
    state: FileStoreState,
    filename: string,
    content: string,
  ): Promise<FileStoreState>
  /** Copies a shared folder into the store as a new top-level folder (named
   * `Name 2` on a clash), all of it in one state update; the cloud backend
   * queues a create per file and a placeholder per empty folder. Rejects
   * when the folder is not valid. */
  importFolder(
    state: FileStoreState,
    snapshot: FolderSnapshot,
  ): Promise<FileStoreState>
  renameFile(
    state: FileStoreState,
    from: string,
    to: string,
  ): Promise<FileStoreState>
  deleteFile(state: FileStoreState, name: string): Promise<FileStoreState>
  /** Renames/moves a whole folder; resolves to the same state when Rust
   * rejects it (collision, invalid path). The cloud backend renames each
   * file on its own, not as one batch, until Phase 3. */
  moveFolder(
    state: FileStoreState,
    from: string,
    to: string,
  ): Promise<FileStoreState>
  /** Moves every file in the folder to the bin; the cloud backend trashes
   * each file on its own, not as one batch, until Phase 3. */
  deleteFolder(state: FileStoreState, folder: string): Promise<FileStoreState>
  restoreFile(state: FileStoreState, name: string): Promise<FileStoreState>
  updateActiveContent(state: FileStoreState, content: string): FileStoreState
  saveContent(state: FileStoreState): Promise<void>
  status(): SaveStatus
}
