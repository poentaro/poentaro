import '../shared-preview-banner.css'

interface SharedPreviewBannerProps {
  /** Name of the shared folder, when the link is a static folder link. */
  folderName?: string
  /** True while the viewer's own files are still loading: importing then
   * would not see their names and could clash with them. */
  importDisabled: boolean
  onImport: () => void
  onDiscard: () => void
}

export function SharedPreviewBanner({
  folderName,
  importDisabled,
  onImport,
  onDiscard,
}: SharedPreviewBannerProps) {
  return (
    <div className="shared-preview-banner">
      {folderName ? (
        <p>
          Shared folder: <strong>{folderName}</strong>
        </p>
      ) : null}
      <div className="shared-preview-actions">
        <button
          type="button"
          className="shared-preview-import-btn"
          disabled={importDisabled}
          onClick={onImport}
        >
          {folderName ? 'Import this folder' : 'Import this score'}
        </button>
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
