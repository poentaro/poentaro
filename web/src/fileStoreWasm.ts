// Bridge between `fileStore.ts`'s plain-object `FileStoreState` and the Rust
// file-store operations (`poentaro::file_tree`, exposed through the
// wasm component). It only converts shapes: `Record<string, string>` maps
// cross the boundary as ordered key/value lists, and a Rust `err` result
// becomes "no change" or a returned failure. All tree, path and naming logic
// lives in Rust. Every call needs the component initialised (`ensureWasmInit`).

import type { FileStoreState } from './fileStore'
import type {
  FileOperationContext,
  FileStoreFailure,
  poentaroWasm,
  StringEntry,
  FileStoreState as WasmFileStoreState,
} from './poentaroWasm'

const entriesOf = (record: Record<string, string>): StringEntry[] =>
  Object.entries(record).map(([key, value]) => ({ key, value }))

const recordOf = (entries: StringEntry[]): Record<string, string> =>
  Object.fromEntries(entries.map(({ key, value }) => [key, value]))

export function toWasmState(state: FileStoreState): WasmFileStoreState {
  return {
    active: state.active,
    userFiles: entriesOf(state.userFiles),
    bin: entriesOf(state.bin),
    fileIds: entriesOf(state.fileIds),
    folders: state.folders ?? [],
  }
}

/** `folders` is only present when non-empty, so stores that never used
 * folders keep exactly their old persisted shape. */
export function fromWasmState(state: WasmFileStoreState): FileStoreState {
  return {
    active: state.active,
    userFiles: recordOf(state.userFiles),
    bin: recordOf(state.bin),
    fileIds: recordOf(state.fileIds),
    ...(state.folders.length > 0 ? { folders: state.folders } : {}),
  }
}

export function operationContext(
  demoFiles: { name: string; content: string }[],
  newFileId: string,
): FileOperationContext {
  return {
    demoFiles: demoFiles.map(({ name, content }) => ({ path: name, content })),
    newFileId,
  }
}

export type StoreOperationResult =
  | { ok: true; state: FileStoreState }
  | { ok: false; failure: FileStoreFailure }

export function storeResult(
  response: ReturnType<ReturnType<typeof poentaroWasm>['fileStoreCreateFile']>,
): StoreOperationResult {
  return response.tag === 'ok'
    ? { ok: true, state: fromWasmState(response.val) }
    : { ok: false, failure: response.val }
}
