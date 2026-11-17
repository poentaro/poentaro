import type { SyncedShareHeaderProps } from './AppHeader'
import { SharedPreviewBanner } from './SharedPreviewBanner'
import { SyncedShareBanner } from './SyncedShareBanner'

interface ShareBannerProps {
  syncedShare: SyncedShareHeaderProps
  /** True while the viewer's own cloud files are still loading. */
  importDisabled: boolean
}

/** The header banner for whichever kind of shared link is open, if any: a
 * static folder, a static file, or a synced file or folder. */
export function ShareBanner({ syncedShare, importDisabled }: ShareBannerProps) {
  const { sharedPreview, sharedFolder, viewerActive } = syncedShare
  if (sharedFolder?.kind === 'static') {
    return (
      <SharedPreviewBanner
        folderName={sharedFolder.folderName}
        importDisabled={importDisabled}
        onImport={sharedFolder.onImport}
        onDiscard={sharedFolder.onDiscard}
      />
    )
  }
  if (sharedPreview) {
    return (
      <SharedPreviewBanner
        importDisabled={importDisabled}
        onImport={syncedShare.onImportShared}
        onDiscard={syncedShare.onDismissShared}
      />
    )
  }
  if (!viewerActive) return null
  return (
    <SyncedShareBanner
      status={syncedShare.viewerStatus}
      filename={syncedShare.viewerFilename}
      folderName={sharedFolder?.folderName ?? null}
      importDisabled={importDisabled}
      onImport={
        sharedFolder ? sharedFolder.onImport : syncedShare.onImportSyncedShare
      }
      onDiscard={syncedShare.onDiscardSyncedShare}
    />
  )
}
