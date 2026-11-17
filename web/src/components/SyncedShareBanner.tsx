import type { SyncedShareViewerStatus } from '../hooks/useSyncedShareViewer'
import '../shared-preview-banner.css'

interface SyncedShareBannerProps {
  status: SyncedShareViewerStatus
  filename: string | null
  /** Name of the shared folder when the link is a folder link. */
  folderName: string | null
  /** True while the viewer's own files are still loading: importing then
   * would not see their names and could clash with them. */
  importDisabled: boolean
  onImport: () => void
  onDiscard: () => void
}

export function SyncedShareBanner({
  status,
  filename,
  folderName,
  importDisabled,
  onImport,
  onDiscard,
}: SyncedShareBannerProps) {
  const canImport = status === 'synced' && (folderName || filename)
  return (
    <div className="shared-preview-banner">
      <p>
        {status === 'loading' && 'Loading synced share…'}
        {status === 'synced' && folderName && (
          <>
            Synced folder: <strong>{folderName}</strong>
          </>
        )}
        {status === 'synced' && !folderName && (
          <>
            Synced: <strong>{filename}</strong>
          </>
        )}
        {status === 'unreachable' &&
          'Could not load this synced share — try reloading.'}
        {status === 'too-large' && 'This shared folder is too large to open.'}
        {status === 'ended' && 'This synced share has ended.'}
      </p>
      <div className="shared-preview-actions">
        {canImport && (
          <button
            type="button"
            className="shared-preview-import-btn"
            disabled={importDisabled}
            onClick={onImport}
          >
            {folderName ? 'Import this folder' : 'Import to my scores'}
          </button>
        )}
        <button
          type="button"
          className="shared-preview-discard-btn"
          onClick={onDiscard}
        >
          Discard
        </button>
      </div>
    </div>
  )
}
