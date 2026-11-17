import { displayFileName, type FileStoreState, fileContent } from '../fileStore'
import { binEntries } from '../fileStoreQueries'
import type { FileOpError } from '../hooks/useFileOperations'
import type { PendingDownload } from '../hooks/usePoentaroWorkerTypes'
import type {
  StorageBackendPreference,
  StorageBackendTarget,
} from '../hooks/useStorageBackend'
import type { AccountDialog } from '../hooks/useSyncedShareOwner'
import { withRenamedKey } from '../storage/cloudBackendNaming'
import type { CloudBackend } from '../storage/cloudBackendTypes'
import type { StorageBackend } from '../storage/types'
import { AccountDialogs } from './AccountDialogs'
import { BinModal } from './BinModal'
import { DownloadRenameModal } from './DownloadRenameModal'
import { ErrorModal } from './ErrorModal'
import { FileHistoryModal } from './fileHistory/FileHistoryModal'
import { StorageSettingsModal } from './StorageSettingsModal'
import { SyncPanelContainer } from './syncPanel/SyncPanelContainer'
import { withServerContent } from './syncPanel/withServerContent'

interface AppOverlaysProps {
  fileOpError: FileOpError | null
  setFileOpError: (error: FileOpError | null) => void
  storageSettingsOpen: boolean
  setStorageSettingsOpen: (open: boolean) => void
  backend: StorageBackend
  isLoadingCloud: boolean
  preference: StorageBackendPreference
  switchBackend: (target: StorageBackendTarget) => Promise<void>
  store: FileStoreState
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void
  refreshSaveStatus: (syncedStore?: FileStoreState) => void
  selectedMeasureRange: { start: number; end: number } | null
  binOpen: boolean
  setBinOpen: (open: boolean) => void
  syncPanelOpen: boolean
  hasUnsavedEdits: boolean
  setSyncPanelOpen: (open: boolean) => void
  historyOpen: boolean
  setHistoryOpen: (open: boolean) => void
  /** Id of the active file, whose history the History panel shows. */
  fileId: string
  onRestoreVersion: (content: string) => void
  onRestore: (name: string) => void
  restoringFileName?: string | null
  pendingDownload: PendingDownload | null
  onConfirmDownload: (filename: string) => void
  onCancelDownload: () => void
  accountDialog: AccountDialog | null
  onCloseAccountDialog: () => void
  /** Opens the account dialog's sign-in form (the storage dialog's
   * shortcut). */
  onSignIn: () => void
}

/** Modals and the hidden test-probe span that sit above the main
 * editor/preview workspace. Grouped here purely to keep App.tsx's JSX under
 * the project's per-file line limit. */
export function AppOverlays({
  fileOpError,
  setFileOpError,
  storageSettingsOpen,
  setStorageSettingsOpen,
  backend,
  isLoadingCloud,
  preference,
  switchBackend,
  store,
  setStore,
  refreshSaveStatus,
  selectedMeasureRange,
  binOpen,
  setBinOpen,
  syncPanelOpen,
  hasUnsavedEdits,
  setSyncPanelOpen,
  historyOpen,
  setHistoryOpen,
  fileId,
  onRestoreVersion,
  onRestore,
  restoringFileName,
  pendingDownload,
  onConfirmDownload,
  onCancelDownload,
  accountDialog,
  onCloseAccountDialog,
  onSignIn,
}: AppOverlaysProps) {
  /** A store that already matches the server is not a user edit, so it must
   * not arm an autosave (which would show "Unsaved" and re-send it). */
  const applySyncedStore = (next: FileStoreState) => {
    setStore(next)
    refreshSaveStatus(next)
  }

  return (
    <>
      <ErrorModal
        open={fileOpError !== null}
        onOpenChange={(open) => {
          if (!open) setFileOpError(null)
        }}
        title={fileOpError?.title ?? ''}
        message={fileOpError?.message ?? ''}
        stack={fileOpError?.stack}
      />
      <StorageSettingsModal
        open={storageSettingsOpen}
        onOpenChange={setStorageSettingsOpen}
        backend={backend}
        isLoadingCloud={isLoadingCloud}
        preference={preference}
        switchBackend={switchBackend}
        store={store}
        setStore={setStore}
        refreshSaveStatus={refreshSaveStatus}
        onSignIn={onSignIn}
      />
      <AccountDialogs open={accountDialog} onClose={onCloseAccountDialog} />
      <BinModal
        open={binOpen}
        onOpenChange={setBinOpen}
        entries={binOpen ? binEntries(store) : []}
        onRestore={onRestore}
        restoringName={restoringFileName}
      />
      <SyncPanelContainer
        open={syncPanelOpen}
        onOpenChange={setSyncPanelOpen}
        backend={backend}
        store={store}
        onServerContent={(fileId, content) =>
          applySyncedStore(withServerContent(store, fileId, content))
        }
        replaceStore={applySyncedStore}
        hasUnsavedEdits={hasUnsavedEdits}
        onLocalRename={({ from, to }) =>
          setStore((previous) => withRenamedKey(previous, from, to))
        }
      />
      {backend.kind === 'cloud' ? (
        <FileHistoryModal
          open={historyOpen}
          onOpenChange={setHistoryOpen}
          backend={backend as CloudBackend}
          fileId={fileId}
          fileName={displayFileName(store.active)}
          currentContent={fileContent(store, store.active)}
          onRestore={onRestoreVersion}
        />
      ) : null}
      <DownloadRenameModal
        pending={pendingDownload}
        onConfirm={onConfirmDownload}
        onCancel={onCancelDownload}
      />
      <span
        data-testid="selected-measure-range"
        aria-hidden="true"
        style={{ display: 'none' }}
      >
        {selectedMeasureRange
          ? `${selectedMeasureRange.start}-${selectedMeasureRange.end}`
          : ''}
      </span>
    </>
  )
}
