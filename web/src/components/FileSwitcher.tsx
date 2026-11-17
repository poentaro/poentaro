import { useState } from 'react'
import { fileContent, sortedUserFileNames } from '../fileStore'
import {
  fileMoveTargets,
  moveDestination,
  renameError,
} from '../fileStoreQueries'
import { useExpandedFolders } from '../hooks/useExpandedFolders'
import { useFileSelection } from '../hooks/useFileSelection'
import { useFileTreeView } from '../hooks/useFileTreeView'
import { useFolderRowActions } from '../hooks/useFolderRowActions'
import { useNewFilePrompt } from '../hooks/useNewFilePrompt'
import { useNewFolderPrompt } from '../hooks/useNewFolderPrompt'
import { useFileStaticShareLink } from '../hooks/useStaticShareLink'
import { FileActionsMenu } from './FileActionsMenu'
import { FileBrowserModal } from './FileBrowserModal'
import { FileSelectionBar } from './FileSelectionBar'
import type { FileSwitcherProps } from './FileSwitcherProps'
import { FileSwitcherTrigger } from './FileSwitcherTrigger'
import { type FolderDialogState, FolderDialogs } from './FolderDialogs'
import { FolderShareModal, type FolderShareTarget } from './FolderShareModal'
import { MoveToDialog } from './MoveToDialog'
import { NewEntryDialog } from './NewEntryDialog'
import { RenameFileDialog } from './RenameFileDialog'
import { SaveStatusBadge } from './SaveStatusBadge'
import { ShareModal } from './ShareModal'

export function FileSwitcher({
  store,
  triggerLabel,
  onSelect,
  onCreate,
  onCreateFolder,
  onDuplicate,
  onRename,
  onDelete,
  onMoveFolder,
  onDeleteFolder,
  onMoveSelection,
  onDeleteSelection,
  onOpenStorageSettings,
  saveStatus,
  saveLabel,
  autosaveDeadline,
  creating = false,
  creatingFolder = false,
  deletingName = null,
  duplicating = false,
  renamingName = null,
  isLoadingCloud = false,
  cloudLoadFailure = null,
  importing = false,
  onImportFile,
  binNames,
  onOpenBin,
  onOpenSyncPanel,
  onOpenHistory,
  canSync,
  isSynced,
  syncedShareLink,
  isSignedIn,
  accountEmail,
  onStartSync,
  onStopSync,
  onSignIn,
  folderShares,
}: FileSwitcherProps) {
  const names = sortedUserFileNames(store)
  const showEmptyHint = !isLoadingCloud && names.length === 0
  const [filesOpen, setFilesOpen] = useState(false)
  const [query, setQuery] = useState('')
  const {
    expanded,
    toggle: toggleFolder,
    expand: expandFolders,
  } = useExpandedFolders()
  const selection = useFileSelection(store)
  const { rows, activeLabel } = useFileTreeView(
    store,
    expanded,
    triggerLabel,
    folderShares.overview,
    query,
    selection.selected,
  )
  const listRenameError = (from: string, to: string) =>
    renameError(store, from, to)
  const [shareModalOpen, setShareModalOpen] = useState(false)
  const [renameOpen, setRenameOpen] = useState(false)
  const [moveOpen, setMoveOpen] = useState(false)
  const [folderDialog, setFolderDialog] = useState<FolderDialogState | null>(
    null,
  )
  const newFilePrompt = useNewFilePrompt({
    store,
    onCreate,
    onConfirmed: () => setFilesOpen(false),
  })
  const newFolderPrompt = useNewFolderPrompt({
    store,
    onCreateFolder,
    expandFolders,
  })
  const [shareFolder, setShareFolder] = useState<FolderShareTarget | null>(null)
  const fileStaticLink = useFileStaticShareLink(
    shareModalOpen,
    store.active,
    fileContent(store, store.active),
  )
  const folderActions = useFolderRowActions({
    folderShares,
    onImportFile,
    closeList: () => setFilesOpen(false),
    openFolderDialog: setFolderDialog,
    openFolderShare: setShareFolder,
    openNewFile: newFilePrompt.open,
    openNewFolder: newFolderPrompt.open,
  })

  return (
    <div className="file-tab-bar">
      <SaveStatusBadge
        status={saveStatus}
        label={saveLabel}
        onOpen={onOpenSyncPanel}
        autosaveDeadline={autosaveDeadline}
      />
      <div className="export-menu">
        <FileBrowserModal
          open={filesOpen}
          onOpenChange={(open) => {
            setFilesOpen(open)
            if (open) setQuery('')
            else selection.stop()
          }}
          trigger={
            <FileSwitcherTrigger
              activeLabel={activeLabel}
              isLoadingCloud={isLoadingCloud}
            />
          }
          activePath={store.active}
          query={query}
          onQueryChange={setQuery}
          rows={rows}
          isLoadingCloud={isLoadingCloud}
          cloudLoadFailure={cloudLoadFailure}
          storeIsEmpty={showEmptyHint}
          creating={creating}
          creatingFolder={creatingFolder}
          importing={importing}
          onNewFile={() => newFilePrompt.open(activeLabel.folderPath)}
          onNewFolder={() => newFolderPrompt.open(activeLabel.folderPath)}
          onImportFile={
            onImportFile
              ? (file) => {
                  onImportFile(file, activeLabel.folderPath)
                  setFilesOpen(false)
                }
              : undefined
          }
          selection={selection}
          selectionBar={
            <FileSelectionBar
              store={store}
              selection={selection}
              shares={folderShares.overview}
              onRename={onRename}
              onMoveFolder={onMoveFolder}
              onMoveSelection={onMoveSelection}
              onDeleteSelection={onDeleteSelection}
              expandFolders={expandFolders}
            />
          }
          treeProps={{
            renamingName,
            onToggleFolder: toggleFolder,
            onSelect: (selected) => {
              onSelect(selected)
              setFilesOpen(false)
            },
            onRename,
            renameError: listRenameError,
            creating,
            creatingFolder,
            importing,
            folderActions,
          }}
        />
      </div>
      <FileActionsMenu
        activePath={store.active}
        folderPath={activeLabel.folderPath}
        creating={creating}
        creatingFolder={creatingFolder}
        duplicating={duplicating}
        importing={importing}
        isLoadingCloud={isLoadingCloud}
        renamingName={renamingName}
        deletingName={deletingName}
        binNames={binNames}
        onImportFile={onImportFile ? (file) => onImportFile(file) : undefined}
        onNewFile={newFilePrompt.open}
        onNewFolder={newFolderPrompt.open}
        onDuplicate={onDuplicate}
        onRename={() => setRenameOpen(true)}
        onMove={() => setMoveOpen(true)}
        onShare={() => setShareModalOpen(true)}
        onDelete={onDelete}
        onOpenBin={onOpenBin}
        onOpenHistory={onOpenHistory}
        onOpenStorageSettings={onOpenStorageSettings}
      />
      <RenameFileDialog
        open={renameOpen}
        onOpenChange={setRenameOpen}
        path={store.active}
        onRename={onRename}
        renameError={listRenameError}
      />
      <MoveToDialog
        open={moveOpen}
        onOpenChange={setMoveOpen}
        path={store.active}
        currentFolder={activeLabel.folderPath}
        targets={moveOpen ? fileMoveTargets(store) : []}
        destination={(folder) => moveDestination(store.active, folder)}
        moveError={(to) => renameError(store, store.active, to)}
        onMove={onRename}
      />
      <NewEntryDialog {...newFilePrompt.dialogProps} />
      <NewEntryDialog {...newFolderPrompt.dialogProps} />
      <FolderDialogs
        store={store}
        dialog={folderDialog}
        shares={folderShares.overview}
        onClose={() => setFolderDialog(null)}
        onMoveFolder={onMoveFolder}
        onDeleteFolder={onDeleteFolder}
        expandFolders={expandFolders}
      />
      <ShareModal
        open={shareModalOpen}
        onOpenChange={setShareModalOpen}
        kind="file"
        name={store.active}
        staticLink={fileStaticLink}
        canSync={canSync}
        isSynced={isSynced}
        syncedShareLink={syncedShareLink}
        isSignedIn={isSignedIn}
        accountEmail={accountEmail}
        onStartSync={onStartSync}
        onStopSync={onStopSync}
        onSignIn={onSignIn}
      />
      <FolderShareModal
        store={store}
        target={shareFolder}
        onClose={() => setShareFolder(null)}
        folderShares={folderShares}
        isSignedIn={isSignedIn}
        accountEmail={accountEmail}
        onSignIn={onSignIn}
      />
    </div>
  )
}
