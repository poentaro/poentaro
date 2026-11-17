import type { FileStoreState } from '../fileStore'
import { activeFileLabel, fileTreeView } from '../fileStoreQueries'
import type {
  FileActiveLabel,
  FileTreeViewRow,
  ShareOverview,
} from '../poentaroWasm'
import { useWasmReady } from './useWasmReady'

export interface FileTreeView {
  rows: FileTreeViewRow[]
  activeLabel: FileActiveLabel
}

/** What the file list and toolbar button render, derived in Rust. Until the
 * main-thread wasm component is ready (the first paint can come before it)
 * there are no rows and the active name is shown whole. */
export function useFileTreeView(
  store: FileStoreState,
  expanded: string[],
  activeName: string,
  shares: ShareOverview,
  query: string,
  selected: string[],
): FileTreeView {
  return useWasmReady()
    ? {
        rows: fileTreeView(store, expanded, shares, query, selected),
        activeLabel: activeFileLabel(activeName),
      }
    : {
        rows: [],
        activeLabel: {
          folderNames: [],
          fileName: activeName,
          folderPath: undefined,
        },
      }
}
