import { useEffect } from 'react'
import type { FileStoreState } from '../fileStore'
import { folderShareInfo } from '../fileStoreQueries'
import type { FolderShares } from '../hooks/useFolderShares'
import { useFolderStaticShareLink } from '../hooks/useStaticShareLink'
import { ShareModal, type Tab } from './ShareModal'

export interface FolderShareTarget {
  folder: string
  /** The tab shown first: "Share folder" opens the static one, "Sharing
   * settings" the synced one. */
  tab: Tab
}

interface FolderShareModalProps {
  store: FileStoreState
  /** The folder being shared; `null` while the dialog is closed. */
  target: FolderShareTarget | null
  onClose: () => void
  folderShares: FolderShares
  isSignedIn: boolean
  accountEmail: string | null
  onSignIn: () => void
  /** Reads the text of the folder's files, which the snapshot holds. */
  onLoadFolderText?: (folder: string) => Promise<void>
}

/** The Share dialog of one folder: wires `ShareModal` to the owner's share
 * overview and the folder's static snapshot link. What it says (counts,
 * warnings, whether the folder is already shared) all comes from Rust. */
export function FolderShareModal({
  store,
  target,
  onClose,
  folderShares,
  isSignedIn,
  accountEmail,
  onSignIn,
  onLoadFolderText,
}: FolderShareModalProps) {
  const folder = target?.folder ?? null
  useEffect(() => {
    if (folder !== null) void onLoadFolderText?.(folder)
  }, [folder, onLoadFolderText])
  const info =
    folder === null
      ? undefined
      : folderShareInfo(store, folderShares.overview, folder)
  // A link made before every file is read would leave some of them out.
  const textsPending = Object.entries(store.userFiles).some(
    ([name, text]) =>
      folder !== null &&
      name.startsWith(`${folder}/`) &&
      text.tag === 'not-loaded',
  )
  const staticLink = useFolderStaticShareLink(
    target !== null && !textsPending,
    store,
    folder,
  )
  return (
    <ShareModal
      open={target !== null}
      onOpenChange={(open) => {
        if (!open) onClose()
      }}
      kind="folder"
      name={info?.name ?? ''}
      staticLink={staticLink}
      initialTab={target?.tab ?? 'static'}
      startWarning={info?.ownLinkWarning ?? null}
      canSync={folderShares.canShare}
      isSynced={info?.shareId !== undefined}
      syncedShareLink={
        info?.shareId === undefined
          ? null
          : folderShares.linkFor(info.shareId, info.name)
      }
      isSignedIn={isSignedIn}
      accountEmail={accountEmail}
      onStartSync={() =>
        folder === null
          ? Promise.resolve(null)
          : folderShares.startSync(folder, info?.name ?? '')
      }
      onStopSync={() => {
        if (folder !== null) void folderShares.stopSync(folder)
      }}
      onSignIn={onSignIn}
    />
  )
}
