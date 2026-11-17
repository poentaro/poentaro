import { useCallback, useEffect, useRef, useState } from 'react'
import type { FileStoreState } from '../fileStore'
import type { FolderSnapshot } from '../poentaroWasm'
import type { SharePayload } from '../shareUrl'
import type { StorageBackend } from '../storage/types'
import {
  callWorker,
  createWorkerClient,
  traceIdOf,
  WorkerRequestError,
} from '../syncedShare/workerClient'
import { clearSyncedShare, parseSyncedShare } from '../syncedShareUrl'
import { useImportToStorage } from './useImportToStorage'

export type SyncedShareViewerStatus =
  /** Not viewing a synced link at all (none in the URL, or already left). */
  | 'none'
  | 'loading'
  | 'synced'
  | 'unreachable'
  | 'ended'
  /** A folder share too big for the worker to serve (413). */
  | 'too-large'

interface FileOpError {
  title: string
  message: string
  stack?: string
}

/**
 * Parses a `?s=` synced-share link (or a legacy `#synced=` hash, if present) on mount and fetches the
 * share's current doc once — the shared cloud file's last saved content,
 * read by the worker straight from its `files` row. Produces the same `{filename,
 * content}` shape `useSharedPreview` does, so it plugs into the identical
 * `source`/`readOnly` derivation in `App.tsx`. Also collapses the editor
 * pane, mirroring `useSharedPreview`.
 *
 * A link to a shared folder answers with the folder's whole tree instead
 * (`folder`): that is exposed as `syncedFolder`, a `FolderSnapshot` shaped
 * like a static folder link's payload, and `useSharedFolderViewer` shows it.
 *
 * There is no push from the server: an owner's later edits only reach a
 * viewer that reloads the page after the owner's cloud autosave has landed,
 * by design — the sharer is expected to tell viewers to refresh when they
 * want an update seen.
 */
export function useSyncedShareViewer(
  setEditorCollapsed: (collapsed: boolean) => void,
  store: FileStoreState,
  backend: StorageBackend,
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void,
  setFileOpError: (error: FileOpError | null) => void,
) {
  const [syncedShareViewerPreview, setSyncedShareViewerPreview] =
    useState<SharePayload | null>(null)
  const [syncedShareViewerStatus, setSyncedShareViewerStatus] =
    useState<SyncedShareViewerStatus>(() =>
      parseSyncedShare() && import.meta.env.VITE_SYNCED_SHARE_HOST
        ? 'loading'
        : 'none',
    )
  /** The failed load's trace ID, shown with the `unreachable`/`too-large`
   * notice. */
  const [syncedShareViewerTraceId, setSyncedShareViewerTraceId] = useState<
    string | null
  >(null)
  const [syncedFolder, setSyncedFolder] = useState<FolderSnapshot | null>(null)
  /** Set once the viewer leaves, so a load still in flight is dropped. */
  const leftRef = useRef(false)
  const { importFile: importToStorage } = useImportToStorage(
    store,
    backend,
    setStore,
    setFileOpError,
  )
  /** Leaves viewer mode: drops the link from the URL and everything it
   * loaded, whatever state the load reached. */
  const leaveSyncedShare = useCallback(() => {
    leftRef.current = true
    clearSyncedShare()
    setSyncedShareViewerPreview(null)
    setSyncedFolder(null)
    setSyncedShareViewerStatus('none')
    setSyncedShareViewerTraceId(null)
  }, [])

  const handleImportSyncedShare = useCallback(async () => {
    if (!syncedShareViewerPreview) return
    try {
      await importToStorage(
        syncedShareViewerPreview.filename,
        syncedShareViewerPreview.content,
      )
      leaveSyncedShare()
    } catch {
      // handled by importToStorage via setFileOpError
    }
  }, [syncedShareViewerPreview, importToStorage, leaveSyncedShare])

  useEffect(() => {
    const parsed = parseSyncedShare()
    if (!parsed) return
    const host = import.meta.env.VITE_SYNCED_SHARE_HOST
    if (!host) return

    setEditorCollapsed(true)

    let cancelled = false
    void callWorker(
      createWorkerClient(host).GET('/shares/{share_id}', {
        params: { path: { share_id: parsed.shareId } },
        cache: 'no-store',
      }),
    )
      .then((doc) => {
        if (cancelled || leftRef.current) return
        if (doc.ended) {
          setSyncedShareViewerPreview(null)
          setSyncedShareViewerStatus('ended')
          return
        }
        if (doc.folder) {
          setSyncedFolder({
            folder: doc.folder.name,
            files: doc.folder.files.map(({ path, content }) => ({
              path,
              content,
            })),
            emptyFolders: doc.folder.emptyFolders,
          })
        } else {
          setSyncedShareViewerPreview({
            filename: doc.filename,
            content: doc.content,
          })
        }
        setSyncedShareViewerStatus('synced')
      })
      .catch((error: unknown) => {
        if (cancelled || leftRef.current) return
        setSyncedShareViewerStatus(failureStatus(error))
        setSyncedShareViewerTraceId(traceIdOf(error))
      })

    return () => {
      cancelled = true
    }
  }, [setEditorCollapsed])

  return {
    syncedFolder,
    leaveSyncedShare,
    syncedShareViewerPreview,
    syncedShareViewerStatus,
    syncedShareViewerTraceId,
    handleImportSyncedShare,
  }
}

/** What a failed `GET /shares/{id}` means for the viewer: an oversized folder
 * says so, anything else is a connection problem worth retrying. (An unknown
 * or stopped share is not a failure: the worker answers `ended: true`.) */
function failureStatus(error: unknown): SyncedShareViewerStatus {
  if (error instanceof WorkerRequestError) {
    if (error.apiError?.code === 'shared_folder_too_large') return 'too-large'
  }
  return 'unreachable'
}
