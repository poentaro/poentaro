// Read-only questions and the import step for a shared folder (a synced
// `?s=` link or a static `#share-folder=` link) seen by a viewer. Pure
// conversions around the wasm exports (`poentaro::file_tree::
// shared_folder`); the viewer's tree, first file and the import's unique
// folder name are all decided in Rust. Needs the wasm component initialised.

import { DEMO_FILES } from './defaultSource'
import { type FileStoreState, generateFileId } from './fileStore'
import { operationContext, storeResult, toWasmState } from './fileStoreWasm'
import type {
  FileActiveLabel,
  FileTreeViewRow,
  FolderSnapshot,
} from './poentaroWasm'
import { poentaroWasm } from './poentaroWasm'

/** The visible rows of the viewer's file list, rooted at the shared folder
 * (no Demo row, no share markers). Empty if the folder is not valid. */
export function sharedFolderView(
  snapshot: FolderSnapshot,
  active: string,
  expanded: string[],
): FileTreeViewRow[] {
  const response = poentaroWasm().sharedFolderView(snapshot, active, expanded)
  return response.tag === 'ok' ? response.val : []
}

/** The file a viewer opens first, if the folder holds any. */
export function sharedFolderFirstFile(
  snapshot: FolderSnapshot,
): string | undefined {
  return poentaroWasm().sharedFolderFirstFile(snapshot) ?? undefined
}

/** `state` with the shared folder copied in as a new top-level folder
 * (`Name 2` on a name clash), the first file active. Throws with Rust's
 * message when the folder is not valid. */
export function importSharedFolderState(
  state: FileStoreState,
  snapshot: FolderSnapshot,
): FileStoreState {
  const result = storeResult(
    poentaroWasm().fileStoreImportSharedFolder(
      toWasmState(state),
      snapshot,
      snapshot.files.map(() => generateFileId()),
      operationContext(DEMO_FILES, generateFileId()),
    ),
  )
  if (!result.ok) throw new Error(result.failure.message)
  return result.state
}

/** The open file split into its folders and own name for the picker button
 * (no demo files exist in a shared folder). */
export function sharedFolderActiveLabel(path: string): FileActiveLabel {
  return poentaroWasm().fileStoreActiveLabel(path, [])
}
