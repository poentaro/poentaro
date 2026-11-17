import type { FileStoreState } from '../fileStore'
import {
  fileMoveTargets,
  folderDeleteSummary,
  folderMoveError,
  foldersToReveal,
  moveDestination,
  parentFolder,
} from '../fileStoreQueries'
import type { ShareOverview } from '../poentaroWasm'
import { DeleteFolderDialog } from './DeleteFolderDialog'
import { MoveToDialog } from './MoveToDialog'
import { RenameFileDialog } from './RenameFileDialog'

export interface FolderDialogState {
  kind: 'rename' | 'move' | 'delete'
  folder: string
}

interface FolderDialogsProps {
  store: FileStoreState
  dialog: FolderDialogState | null
  /** The owner's live links, for the delete warning. */
  shares: ShareOverview
  onClose: () => void
  onMoveFolder: (from: string, to: string) => void
  onDeleteFolder: (folder: string) => void
  expandFolders: (folders: string[]) => void
}

/** The rename / move-to / delete dialogs opened from a folder row's menu. */
export function FolderDialogs({
  store,
  dialog,
  shares,
  onClose,
  onMoveFolder,
  onDeleteFolder,
  expandFolders,
}: FolderDialogsProps) {
  const folder = dialog?.folder ?? ''
  const onOpenChange = (open: boolean) => {
    if (!open) onClose()
  }
  const moveAndReveal = (from: string, to: string) => {
    onMoveFolder(from, to)
    expandFolders(foldersToReveal(to))
  }
  const deleteSummary =
    dialog?.kind === 'delete'
      ? folderDeleteSummary(store, folder, shares)
      : undefined
  return (
    <>
      <RenameFileDialog
        open={dialog?.kind === 'rename'}
        onOpenChange={onOpenChange}
        path={folder}
        kind="folder"
        onRename={moveAndReveal}
        renameError={(from, to) => folderMoveError(store, from, to)}
      />
      <MoveToDialog
        open={dialog?.kind === 'move'}
        onOpenChange={onOpenChange}
        path={folder}
        currentFolder={
          dialog?.kind === 'move' ? parentFolder(folder) : undefined
        }
        targets={dialog?.kind === 'move' ? fileMoveTargets(store, folder) : []}
        destination={(target) => moveDestination(folder, target)}
        moveError={(to) => folderMoveError(store, folder, to)}
        onMove={moveAndReveal}
      />
      <DeleteFolderDialog
        open={dialog?.kind === 'delete'}
        onOpenChange={onOpenChange}
        folder={folder}
        message={deleteSummary?.message}
        shareWarning={deleteSummary?.shareWarning}
        onConfirm={onDeleteFolder}
      />
    </>
  )
}
