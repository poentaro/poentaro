import { useEffect, useState } from 'react'
import type { FileStoreState } from '../fileStore'
import { activeFileLabel, fileTreeView } from '../fileStoreQueries'
import type {
  FileActiveLabel,
  FileTreeViewRow,
  ShareOverview,
} from '../poentaroWasm'
import { ensureWasmInit } from '../wasmInit'

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
  const [wasmReady, setWasmReady] = useState(false)

  useEffect(() => {
    let cancelled = false
    void ensureWasmInit().then(() => {
      if (!cancelled) setWasmReady(true)
    })
    return () => {
      cancelled = true
    }
  }, [])

  return wasmReady
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
