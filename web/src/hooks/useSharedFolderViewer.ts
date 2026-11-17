import { useCallback, useEffect, useMemo, useState } from 'react'
import type { FileStoreState } from '../fileStore'
import type { FileTreeViewRow, FolderSnapshot } from '../poentaroWasm'
import { sharedFolderFirstFile, sharedFolderView } from '../sharedFolder'
import { clearShareHash, parseFolderShareFromHash } from '../shareUrl'
import type { StorageBackend } from '../storage/types'
import { ensureWasmInit } from '../wasmInit'
import { useImportToStorage } from './useImportToStorage'

interface FileOpError {
  title: string
  message: string
  stack?: string
}

/** A shared folder open for viewing: read-only, rooted at the shared folder,
 * one file shown at a time. The tree rows come from Rust; this is only the
 * viewer's raw state (which file is open, which folders are unfolded). */
export interface SharedFolderViewer {
  /** `static`: a `#share-folder=` snapshot; `synced`: a live `?s=` link,
   * read once on load. */
  kind: 'static' | 'synced'
  folderName: string
  rows: FileTreeViewRow[]
  /** Path of the open file relative to the shared folder; empty if the
   * folder holds no file. */
  activePath: string
  activeContent: string
  /** A per-file id, so part toggles and undo are not shared between the
   * shared folder's files (they are not in the viewer's own store). */
  fileId: string
  onSelect: (path: string) => void
  onToggleFolder: (key: string) => void
  /** Copies the folder into the viewer's own files, then leaves viewer mode. */
  onImport: () => Promise<void>
  /** Leaves viewer mode without importing. */
  onDiscard: () => void
}

/**
 * Viewer state for both kinds of folder link: the static `#share-folder=`
 * snapshot (parsed here on mount) and the synced one handed in by
 * `useSyncedShareViewer`. The file opened first is the first file in tree
 * order (decided in Rust). Also collapses the editor pane, like the
 * single-file viewers do.
 */
export function useSharedFolderViewer(
  syncedFolder: FolderSnapshot | null,
  leaveSyncedShare: () => void,
  store: FileStoreState,
  backend: StorageBackend,
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void,
  setFileOpError: (error: FileOpError | null) => void,
  setEditorCollapsed: (collapsed: boolean) => void,
): SharedFolderViewer | null {
  const [staticFolder, setStaticFolder] = useState<FolderSnapshot | null>(null)
  const [wasmReady, setWasmReady] = useState(false)
  const [selectedPath, setSelectedPath] = useState<string | null>(null)
  const [expanded, setExpanded] = useState<string[]>([])
  const { importFolder } = useImportToStorage(
    store,
    backend,
    setStore,
    setFileOpError,
  )

  useEffect(() => {
    let cancelled = false
    void parseFolderShareFromHash().then((parsed) => {
      if (cancelled) return
      setStaticFolder(parsed)
      if (parsed) setEditorCollapsed(true)
    })
    void ensureWasmInit().then(() => {
      if (!cancelled) setWasmReady(true)
    })
    return () => {
      cancelled = true
    }
  }, [setEditorCollapsed])

  const snapshot = staticFolder ?? syncedFolder
  const activePath = useMemo(() => {
    if (!snapshot || !wasmReady) return ''
    return snapshot.files.some((file) => file.path === selectedPath)
      ? (selectedPath ?? '')
      : (sharedFolderFirstFile(snapshot) ?? '')
  }, [snapshot, wasmReady, selectedPath])
  const rows = useMemo(
    () =>
      snapshot && wasmReady
        ? sharedFolderView(snapshot, activePath, expanded)
        : [],
    [snapshot, wasmReady, activePath, expanded],
  )

  const onToggleFolder = useCallback((key: string) => {
    setExpanded((previous) =>
      previous.includes(key)
        ? previous.filter((existing) => existing !== key)
        : [...previous, key],
    )
  }, [])

  const leave = useCallback(() => {
    if (staticFolder) {
      clearShareHash()
      setStaticFolder(null)
    } else {
      leaveSyncedShare()
    }
    setSelectedPath(null)
    setExpanded([])
  }, [staticFolder, leaveSyncedShare])

  const onImport = useCallback(async () => {
    if (!snapshot) return
    try {
      await importFolder(snapshot)
      leave()
    } catch {
      // handled by importFolder via setFileOpError
    }
  }, [snapshot, importFolder, leave])

  if (!snapshot) return null
  return {
    kind: staticFolder ? 'static' : 'synced',
    folderName: snapshot.folder,
    rows,
    activePath,
    activeContent:
      snapshot.files.find((file) => file.path === activePath)?.content ?? '',
    fileId: `shared-folder:${snapshot.folder}/${activePath}`,
    onSelect: setSelectedPath,
    onToggleFolder,
    onImport,
    onDiscard: leave,
  }
}
