// Read-only questions the file list and its dialogs ask Rust about a
// `FileStoreState`. Pure conversions around the wasm exports; all tree, path
// and naming logic lives in Rust. Needs the wasm component initialised.

import { DEMO_FILE_NAMES, DEMO_FILES } from './defaultSource'
import type { FileStoreState } from './fileStore'
import { operationContext, toWasmState } from './fileStoreWasm'
import type {
  BinEntry,
  FileActiveLabel,
  FileTreeRow,
  FileTreeViewRow,
  FolderDeleteSummary,
  FolderShareInfo,
  FolderStaticShare,
  MoveTarget,
  SelectionDeleteSummary,
  SelectionMovePlan,
  SelectionUnit,
  ShareOverview,
} from './poentaroWasm'
import { poentaroWasm } from './poentaroWasm'

/** The user's files and folders as display rows, depth-first, folders
 * before files, sorted (computed in Rust). Empty if the paths conflict. */
export function fileTreeRows(state: FileStoreState): FileTreeRow[] {
  const response = poentaroWasm().fileStoreTree(toWasmState(state))
  return response.tag === 'ok' ? response.val : []
}

/** The visible rows of the file list (collapsed folders hidden, the active
 * file's ancestors forced open, the read-only `Demo/` row last), computed in
 * Rust from the raw `expanded` set, the search `query` (blank shows
 * everything) and the `selected` file paths (each row's checkbox). Empty if
 * the paths conflict. */
export function fileTreeView(
  state: FileStoreState,
  expanded: string[],
  shares: ShareOverview,
  query: string,
  selected: string[],
): FileTreeViewRow[] {
  const response = poentaroWasm().fileStoreTreeView(
    toWasmState(state),
    expanded,
    DEMO_FILE_NAMES,
    shares,
    query,
    selected,
  )
  return response.tag === 'ok' ? response.val : []
}

/** The folder choices of the "Move to" picker: the top level (empty `path`)
 * plus every folder. Pass the folder being moved as `excludedFolder` to leave
 * it and its descendants out. Computed in Rust. */
export function fileMoveTargets(
  state: FileStoreState,
  excludedFolder?: string,
): MoveTarget[] {
  return poentaroWasm().fileStoreMoveTargets(toWasmState(state), excludedFolder)
}

/** Where `source` lands when moved into `folder` (top level when omitted). */
export function moveDestination(
  source: string,
  folder?: string,
): string | undefined {
  return poentaroWasm().fileStoreMoveDestination(source, folder)
}

/** Why renaming/moving `from` to `to` would be rejected, as a message from
 * Rust; `undefined` when it would succeed. */
export function renameError(
  state: FileStoreState,
  from: string,
  to: string,
): string | undefined {
  return poentaroWasm().fileStoreRenameError(
    toWasmState(state),
    from,
    to,
    operationContext(DEMO_FILES, ''),
  )
}

/** The active file split into its folder names and own name (computed in Rust). */
export function activeFileLabel(active: string): FileActiveLabel {
  return poentaroWasm().fileStoreActiveLabel(active, DEMO_FILE_NAMES)
}

/** The folder holding `path` (a file or a folder), `undefined` at the top
 * level. Rust splits the path; this reuses the active-file label for it. */
export function parentFolder(path: string): string | undefined {
  return activeFileLabel(path).folderPath
}

/** Why creating the file `name` inside `folder` would be rejected, as a
 * message from Rust; `undefined` when it would succeed. */
export function createFileError(
  state: FileStoreState,
  folder: string | undefined,
  name: string,
): string | undefined {
  return poentaroWasm().fileStoreCreateFileError(
    toWasmState(state),
    folder,
    name,
    operationContext(DEMO_FILES, ''),
  )
}

/** Why creating the folder `name` inside `parent` would be rejected, as a
 * message from Rust; `undefined` when it would succeed. */
export function createFolderError(
  state: FileStoreState,
  parent: string | undefined,
  name: string,
): string | undefined {
  return poentaroWasm().fileStoreCreateFolderError(
    toWasmState(state),
    parent,
    name,
    operationContext(DEMO_FILES, ''),
  )
}

/** Why renaming/moving the folder `from` to `to` would be rejected, as a
 * message from Rust; `undefined` when it would succeed. */
export function folderMoveError(
  state: FileStoreState,
  from: string,
  to: string,
): string | undefined {
  return poentaroWasm().fileStoreFolderMoveError(
    toWasmState(state),
    from,
    to,
    operationContext(DEMO_FILES, ''),
  )
}

/** What the "Delete folder" confirmation says (file count, sentence and what
 * happens to live links), from Rust; `undefined` when the folder does not
 * exist. */
export function folderDeleteSummary(
  state: FileStoreState,
  folder: string,
  shares: ShareOverview,
): FolderDeleteSummary | undefined {
  return poentaroWasm().fileStoreFolderDeleteSummary(
    toWasmState(state),
    folder,
    shares,
  )
}

/** What a folder's Share dialog shows (counts, own link, enclosing share,
 * own-link warning), from Rust; `undefined` when the folder does not exist. */
export function folderShareInfo(
  state: FileStoreState,
  shares: ShareOverview,
  folder: string,
): FolderShareInfo | undefined {
  return poentaroWasm().fileStoreFolderShareInfo(
    toWasmState(state),
    shares,
    folder,
  )
}

/** The static link payload of a folder (Rust encodes it and decides whether
 * it is short enough to offer); `undefined` when the folder does not exist. */
export function folderStaticShare(
  state: FileStoreState,
  folder: string,
): FolderStaticShare | undefined {
  return poentaroWasm().fileStoreFolderStaticShare(toWasmState(state), folder)
}

/** The bin as sorted entries: leaf name plus the folder it was deleted from. */
export function binEntries(state: FileStoreState): BinEntry[] {
  return poentaroWasm().fileStoreBinEntries(toWasmState(state))
}

/** `folder` and every folder above it, outermost first: what the file list
 * must expand to show a folder that was just created or moved. */
export function foldersToReveal(folder: string): string[] {
  return poentaroWasm().fileStoreFoldersToReveal(folder)
}

/** Every file path, for "Select all". */
export function selectionAll(state: FileStoreState): string[] {
  return poentaroWasm().fileStoreSelectionAll(toWasmState(state))
}

/** `selected` after clicking the checkbox of `key` (a file, or a folder
 * meaning every file inside it). */
export function selectionToggle(
  state: FileStoreState,
  selected: string[],
  key: string,
): string[] {
  return poentaroWasm().fileStoreSelectionToggle(
    toWasmState(state),
    selected,
    key,
  )
}

/** What an action on the selection applies to: whole folders whose files are
 * all selected, and the other selected files. */
export function selectionUnits(
  state: FileStoreState,
  selected: string[],
): SelectionUnit[] {
  return poentaroWasm().fileStoreSelectionUnits(toWasmState(state), selected)
}

/** The moves that put the selection into `folder` (top level when omitted),
 * or why they would be rejected. */
export function selectionMovePlan(
  state: FileStoreState,
  selected: string[],
  folder?: string,
): SelectionMovePlan {
  return poentaroWasm().fileStoreSelectionMovePlan(
    toWasmState(state),
    selected,
    folder,
    operationContext(DEMO_FILES, ''),
  )
}

/** What the "Delete" confirmation for the selection says. */
export function selectionDeleteSummary(
  state: FileStoreState,
  selected: string[],
  shares: ShareOverview,
): SelectionDeleteSummary {
  return poentaroWasm().fileStoreSelectionDeleteSummary(
    toWasmState(state),
    selected,
    shares,
  )
}
