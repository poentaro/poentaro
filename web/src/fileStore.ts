import NEW_FILE_TEMPLATE from '../../new_file_template.poentaro?raw'
import { DEMO_FILE_NAMES, DEMO_FILES } from './defaultSource'
import {
  FILE_EXTENSION,
  stripFileExtension,
  withFileExtension,
} from './fileExtension'
import {
  fromWasmState,
  storeResult,
  toWasmState,
  operationContext as wasmOperationContext,
} from './fileStoreWasm'
import type {
  FileOperationContext,
  FileStoreFailure,
  FileStoreResponse,
  FileText,
  PathChange,
  FileStoreState as WasmFileStoreState,
} from './poentaroWasm'
import { poentaroWasm } from './poentaroWasm'

export { mergeBackendResult } from './fileStoreMerge'
export {
  loadedFiles,
  loadedFileText,
  NOT_LOADED,
} from './fileText'
export { DEMO_FILE_NAMES }

import { loadedFileText, NOT_LOADED } from './fileText'

const DEMO_FILE_NAME_SET = new Set(DEMO_FILE_NAMES)

const DEMO_FILE_CONTENT: Record<string, string> = Object.fromEntries(
  DEMO_FILES.map((file) => [file.name, file.content]),
)

const DEMO_FILE_IDS: Record<string, string> = Object.fromEntries(
  DEMO_FILE_NAMES.map((name) => [name, `poentaro:reference:${name}`]),
)

export const FILE_STORE_KEY = 'poentaro:files:v1'
export const STORAGE_KEY = 'poentaro:source:v5'

export const DEFAULT_FILE_STORE: FileStoreState = {
  active: DEMO_FILE_NAMES[0] ?? '',
  userFiles: {},
  bin: {},
  fileIds: {},
}

export function generateFileId(): string {
  return crypto.randomUUID()
}

export interface FileStoreState {
  active: string
  userFiles: Record<string, FileText>
  bin: Record<string, FileText>
  /** Stable ID per file name (active and binned); survives renames. */
  fileIds: Record<string, string>
  /** Explicitly created (possibly empty) folders. Optional so stores saved
   * before folders existed still load; folders that merely hold files are
   * implied by the file paths and need no entry. */
  folders?: string[]
}

export function fileIdForName(state: FileStoreState, name: string): string {
  const id = isDemoFile(name) ? DEMO_FILE_IDS[name] : state.fileIds[name]
  if (!id) throw new Error(`Missing file ID for ${name}`)
  return id
}

function isDemoFile(name: string): boolean {
  return DEMO_FILE_NAME_SET.has(name)
}

/** The text of a file, or `not-loaded` when it has not been fetched yet. A
 * name that is in neither the demo files nor the store is `not-loaded` too,
 * never an empty score. */
export function fileContent(state: FileStoreState, name: string): FileText {
  if (isDemoFile(name)) return loadedFileText(DEMO_FILE_CONTENT[name] ?? '')
  return state.userFiles[name] ?? NOT_LOADED
}

/** The text of a file when it is loaded; `undefined` while it is not. */
export function loadedText(
  state: FileStoreState,
  name: string,
): string | undefined {
  const text = fileContent(state, name)
  return text.tag === 'loaded' ? text.val : undefined
}

/** The text of a binned file when it is loaded. */
export function loadedBinText(
  state: FileStoreState,
  name: string,
): string | undefined {
  const text = state.bin[name]
  return text?.tag === 'loaded' ? text.val : undefined
}

/** Names shown in the top level of the file-switcher dropdown — user files
 * only; the read-only demo files live in its nested "Demo" submenu
 * (`DEMO_FILE_NAMES`). */
export function sortedUserFileNames(state: FileStoreState): string[] {
  return Object.keys(state.userFiles).sort((a, b) => a.localeCompare(b))
}

/** All names `selectFile`/persistence may treat as a valid `active` value —
 * every demo file plus every user file. */
function activeFileNames(state: FileStoreState): string[] {
  return [...DEMO_FILE_NAME_SET, ...sortedUserFileNames(state)]
}

export function sortedBinNames(state: FileStoreState): string[] {
  return Object.keys(state.bin).sort((a, b) => a.localeCompare(b))
}

function reservedNames(state: FileStoreState): Set<string> {
  return new Set([
    ...DEMO_FILE_NAMES,
    ...Object.keys(state.userFiles),
    ...Object.keys(state.bin),
  ])
}

function uniqueName(base: string, taken: Set<string>): string {
  if (!taken.has(base)) return base
  const dot = base.lastIndexOf('.')
  const stem = dot > 0 ? base.slice(0, dot) : base
  const ext = dot > 0 ? base.slice(dot) : FILE_EXTENSION
  let n = 2
  while (taken.has(`${stem} ${n}${ext}`)) n++
  return `${stem} ${n}${ext}`
}

/** Strips the redundant extension for display — every file is a
 * `.poentaro` file, so showing it in pickers/tabs is just noise. */
export function displayFileName(name: string): string {
  return stripFileExtension(name)
}

function sanitizeFileName(raw: string): string {
  const trimmed = raw.trim()
  return withFileExtension(trimmed || 'untitled')
}

export function updateActiveContent(
  state: FileStoreState,
  content: string,
): FileStoreState {
  if (isDemoFile(state.active)) return state
  // Text typed against a file that never loaded would overwrite it.
  if (state.userFiles[state.active]?.tag === 'not-loaded') return state
  return {
    ...state,
    userFiles: {
      ...state.userFiles,
      [state.active]: loadedFileText(content),
    },
  }
}

export function selectFile(
  state: FileStoreState,
  name: string,
): FileStoreState {
  if (!activeFileNames(state).includes(name)) return state
  return { ...state, active: name }
}

function operationContext(): FileOperationContext {
  return wasmOperationContext(DEMO_FILES, generateFileId())
}

/** Runs a Rust store operation; a rejected operation (collision, missing
 * file, invalid path, ...) leaves the state untouched, returning the very
 * same object so callers can detect "nothing happened" with `===`. */
function applyOrKeep(
  state: FileStoreState,
  run: (
    wasmState: WasmFileStoreState,
    context: FileOperationContext,
  ) => FileStoreResponse,
): FileStoreState {
  const result = storeResult(run(toWasmState(state), operationContext()))
  return result.ok ? result.state : state
}

/** The file store an origin migration carried from the old origin merged
 * into `current` (see `file_tree::merge_migrated_file_store`), or `null` if
 * Rust refused it. */
export function mergeMigratedFileStore(
  current: FileStoreState,
  carried: FileStoreState,
): FileStoreState | null {
  const result = storeResult(
    poentaroWasm().fileStoreMergeMigrated(
      toWasmState(current),
      toWasmState(carried),
      operationContext(),
    ),
  )
  return result.ok ? result.state : null
}

/** `state` with its files saved under the legacy extension renamed (see
 * `file_tree::rename_legacy_extensions`), or `null` when none need it. */
export function renameLegacyExtensions(
  state: FileStoreState,
): FileStoreState | null {
  const renamed = poentaroWasm().fileStoreRenameLegacyExtensions(
    toWasmState(state),
    operationContext(),
  )
  return renamed === undefined ? null : fromWasmState(renamed)
}

/** New file `name` in `folder` (top level when omitted). Kept as is when
 * `createFileError` objects. */
export function createFile(
  state: FileStoreState,
  folder: string | undefined,
  name: string,
): FileStoreState {
  return applyOrKeep(state, (wasmState, context) =>
    poentaroWasm().fileStoreCreateFile(
      wasmState,
      folder,
      name,
      NEW_FILE_TEMPLATE,
      context,
    ),
  )
}

/** New empty folder `name` inside `parent` (top level when omitted); the
 * active file does not change. Kept as is when `createFolderError` objects. */
export function createFolder(
  state: FileStoreState,
  parent: string | undefined,
  name: string,
): FileStoreState {
  return applyOrKeep(state, (wasmState, context) =>
    poentaroWasm().fileStoreCreateFolder(wasmState, parent, name, context),
  )
}

/** Copies the active file next to itself, in the same folder. */
export function duplicateFile(state: FileStoreState): FileStoreState {
  return applyOrKeep(state, (wasmState, context) =>
    poentaroWasm().fileStoreDuplicateFile(wasmState, state.active, context),
  )
}

/** Renames and/or moves a file (`to` may include folders); keeps its id. */
export function renameFile(
  state: FileStoreState,
  from: string,
  toRaw: string,
): FileStoreState {
  return applyOrKeep(state, (wasmState, context) =>
    poentaroWasm().fileStoreRenameFile(wasmState, from, toRaw, context),
  )
}

export function deleteFile(
  state: FileStoreState,
  name: string,
): FileStoreState {
  return applyOrKeep(state, (wasmState, context) =>
    poentaroWasm().fileStoreDeleteFile(wasmState, name, context),
  )
}

export function restoreFile(
  state: FileStoreState,
  name: string,
): FileStoreState {
  return applyOrKeep(state, (wasmState, context) =>
    poentaroWasm().fileStoreRestoreFile(wasmState, name, context),
  )
}

export type FolderMoveResult =
  | { ok: true; state: FileStoreState; changes: PathChange[] }
  | { ok: false; failure: FileStoreFailure }

/** Renames/moves a folder as one all-or-nothing batch (see Rust
 * `file_tree::move_folder` for the collision rules). */
export function moveFolder(
  state: FileStoreState,
  from: string,
  to: string,
): FolderMoveResult {
  const response = poentaroWasm().fileStoreMoveFolder(
    toWasmState(state),
    from,
    to,
    operationContext(),
  )
  return response.tag === 'ok'
    ? {
        ok: true,
        state: fromWasmState(response.val.state),
        changes: response.val.changes,
      }
    : { ok: false, failure: response.val }
}

/** Moves every file in the folder to the bin (each under its full path). */
export function deleteFolder(
  state: FileStoreState,
  folder: string,
): FileStoreState {
  return applyOrKeep(state, (wasmState, context) =>
    poentaroWasm().fileStoreDeleteFolder(wasmState, folder, context),
  )
}

export function isReadOnlyFile(name: string): boolean {
  return isDemoFile(name)
}

export function importSharedFile(
  state: FileStoreState,
  filenameRaw: string,
  content: string,
): FileStoreState {
  let name = sanitizeFileName(filenameRaw)
  if (isDemoFile(name)) {
    name = 'shared.poentaro'
  }

  const taken = reservedNames(state)
  name = uniqueName(name, taken)

  return {
    ...state,
    active: name,
    userFiles: { ...state.userFiles, [name]: loadedFileText(content) },
    fileIds: { ...state.fileIds, [name]: generateFileId() },
  }
}
