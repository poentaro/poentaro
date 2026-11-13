import type { FileStoreState } from '../fileStore'
import { sortedBinNames } from '../fileStore'
import type { CloudLoadFailure } from '../hooks/useCloudStoreLoader'
import type { FolderShares } from '../hooks/useFolderShares'
import type { SharedFolderViewer } from '../hooks/useSharedFolderViewer'
import type { DisplaySaveStatus } from '../hooks/useStorageBackend'
import type { SyncedShareViewerStatus } from '../hooks/useSyncedShareViewer'
import type { SelectionMove, SelectionUnit } from '../poentaroWasm'
import type { SharePayload } from '../shareUrl'
import type { SyncedShareFailure } from '../syncedShare/errors'
import {
  playFromCurrentMeasureShortcutLabel,
  shortcutLabel,
} from '../utils/shortcutLabels'
import { ExportControls } from './ExportControls'
import { FileSwitcher } from './FileSwitcher'
import { MetronomeToggleButton } from './MetronomeToggleButton'
import { PlayAllButton } from './PlayAllButton'
import { PlayFromCurrentMeasureButton } from './PlayFromCurrentMeasureButton'
import { PlayMeasureButton } from './PlayMeasureButton'
import { ProfileControl } from './ProfileControl'
import { ShareBanner } from './ShareBanner'
import { SharedFolderSwitcher } from './SharedFolderSwitcher'
import { SyncedShareErrorDialog } from './SyncedShareErrorDialog'

interface MeasureRange {
  start: number
  end: number
}

export interface SyncedShareHeaderProps {
  sharedPreview: SharePayload | null
  onImportShared: () => void
  onDismissShared: () => void
  /** Non-null while a `#synced=` link is being viewed. Takes a back seat to
   * `sharedPreview` if both are somehow present at once (a documented edge
   * case, not expected in practice). */
  viewerActive: boolean
  /** The shared folder being viewed (static or synced link), if any. */
  sharedFolder: SharedFolderViewer | null
  viewerStatus: SyncedShareViewerStatus
  viewerFilename: string | null
  onImportSyncedShare: () => void
  onDiscardSyncedShare: () => void
  /** Whether the active file can have a live link (cloud files only) --
   * see `useSyncedShareOwner.ts`. */
  canSync: boolean
  isSynced: boolean
  syncedShareLink: string | null
  /** Whether this browser has an account session -- see
   * `useSyncedShareOwner.ts`. */
  isSignedIn: boolean
  /** The signed-in account's email, for the header chip and the "Synced
   * as" identity row; `null` while signed out. */
  accountEmail: string | null
  onStartSync: () => Promise<string | null>
  onStopSync: () => void
  /** Opens the account dialog (`AccountAuthModal`) on its sign-in form. */
  onSignIn: () => void
  /** Opens the change password dialog. */
  onChangePassword: () => void
  /** Opens the dialog that connects an AI agent with an API key. */
  onOpenApiKeys: () => void
  /** Ends this browser's session, stopping any active sync first. */
  onSignOut: () => void
  /** Drives the full-screen error dialog (task 9) -- non-null while a
   * Synced Share request has failed. */
  syncFailure: SyncedShareFailure | null
  onDismissSyncFailure: () => void
  folderShares: FolderShares
}

interface AppHeaderProps {
  audioAvailable?: boolean
  selectedMeasureRange: MeasureRange | null
  selectedSequenceRange: MeasureRange | null
  measureAudioGenerating: boolean
  soundfontReady: boolean
  measureAudioPlaying: boolean
  playSelectedMeasures: () => void
  playFromCurrentMeasure: () => void
  playAll: () => void
  metronomeEnabled: boolean
  onMetronomeEnabledChange: (enabled: boolean) => void
  /** True while a note range-select (see `useNoteSelection`) is active; when
   * set, `PlayMeasureButton` plays only the selected parts, muted elsewhere,
   * over the selection's measure range instead of the measure(s) under the
   * cursor. */
  notePlaybackSelectionActive: boolean
  playNoteSelection: () => void
  stopMeasurePlayback: () => void
  store: FileStoreState
  onSelect: (name: string) => void
  onCreate: (folder: string | undefined, name: string) => void
  onCreateFolder: (
    parent: string | undefined,
    name: string,
  ) => Promise<FileStoreState | undefined>
  onDuplicate: () => void
  onRename: (from: string, to: string) => void
  onDelete: (name: string) => void
  onMoveFolder: (from: string, to: string) => void
  onDeleteFolder: (folder: string) => void
  onMoveSelection: (moves: SelectionMove[]) => void
  onDeleteSelection: (units: SelectionUnit[]) => void
  onOpenStorageSettings: () => void
  saveStatus: DisplaySaveStatus
  saveLabel: string | null
  autosaveDeadline: number | null
  creatingFile?: boolean
  creatingFolder?: boolean
  deletingFileName?: string | null
  duplicatingFile?: boolean
  renamingFileName?: string | null
  isLoadingCloud: boolean
  cloudLoadFailure?: CloudLoadFailure | null
  onOpenBin: () => void
  onOpenSyncPanel?: () => void
  onOpenHistory?: () => void
  hasDocuments: boolean
  rendering: boolean
  audioGenerating?: boolean
  wavUrl?: string | null
  onGenerateAudio?: () => void
  pdfAvailable?: boolean
  pdfFontsReady?: boolean
  pdfExporting?: boolean
  onExportPdf?: () => void
  splitPdfExporting?: boolean
  onExportSplitPdf?: () => void
  midiAvailable?: boolean
  midiExporting?: boolean
  onExportMidi?: () => void
  splitMidiExporting?: boolean
  onExportSplitMidi?: () => void
  splitWavExporting?: boolean
  onExportSplitWav?: () => void
  mp3Available?: boolean
  mp3Exporting?: boolean
  mp3Url?: string | null
  onExportMp3?: () => void
  splitMp3Exporting?: boolean
  onExportSplitMp3?: () => void
  partsCount?: number
  importing?: boolean
  onImportFile?: (file: File, folder?: string) => void
  syncedShare: SyncedShareHeaderProps
  /** The header account chip's only action -- see `App.tsx`'s
   * `handleAccountSignOut`. Everything else `ProfileControl` needs comes
   * off `syncedShare`. */
  onSignOut: () => void
}

export function AppHeader({
  audioAvailable,
  selectedMeasureRange,
  selectedSequenceRange,
  measureAudioGenerating,
  soundfontReady,
  measureAudioPlaying,
  playSelectedMeasures,
  playFromCurrentMeasure,
  playAll,
  metronomeEnabled,
  onMetronomeEnabledChange,
  notePlaybackSelectionActive,
  playNoteSelection,
  stopMeasurePlayback,
  store,
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
  creatingFile,
  creatingFolder,
  deletingFileName,
  duplicatingFile,
  renamingFileName,
  isLoadingCloud,
  cloudLoadFailure,
  onOpenBin,
  onOpenSyncPanel,
  onOpenHistory,
  hasDocuments,
  rendering,
  audioGenerating,
  wavUrl,
  onGenerateAudio,
  pdfAvailable,
  pdfFontsReady,
  pdfExporting,
  onExportPdf,
  splitPdfExporting,
  onExportSplitPdf,
  midiAvailable,
  midiExporting,
  onExportMidi,
  splitMidiExporting,
  onExportSplitMidi,
  splitWavExporting,
  onExportSplitWav,
  mp3Available,
  mp3Exporting,
  mp3Url,
  onExportMp3,
  splitMp3Exporting,
  onExportSplitMp3,
  partsCount,
  importing,
  onImportFile,
  syncedShare,
  onSignOut,
}: AppHeaderProps) {
  const {
    sharedPreview,
    sharedFolder,
    viewerActive: syncedShareViewerActive,
  } = syncedShare
  return (
    <header className="app-header">
      <ShareBanner syncedShare={syncedShare} importDisabled={isLoadingCloud} />
      {audioAvailable && (
        <MetronomeToggleButton
          enabled={metronomeEnabled}
          disabled={
            measureAudioGenerating || measureAudioPlaying || !soundfontReady
          }
          onChange={onMetronomeEnabledChange}
        />
      )}
      {audioAvailable && (
        <PlayMeasureButton
          disabled={
            (notePlaybackSelectionActive
              ? false
              : selectedMeasureRange === null) ||
            measureAudioGenerating ||
            !soundfontReady
          }
          loading={measureAudioGenerating}
          playing={measureAudioPlaying}
          measureRange={selectedMeasureRange}
          noteSelectionActive={notePlaybackSelectionActive}
          onClick={
            notePlaybackSelectionActive
              ? playNoteSelection
              : playSelectedMeasures
          }
          onPause={stopMeasurePlayback}
          shortcutLabel={shortcutLabel}
        />
      )}
      {audioAvailable && selectedSequenceRange !== null && (
        <PlayFromCurrentMeasureButton
          disabled={
            selectedSequenceRange === null ||
            measureAudioGenerating ||
            !soundfontReady
          }
          loading={measureAudioGenerating}
          playing={measureAudioPlaying}
          currentMeasure={selectedSequenceRange?.start ?? null}
          onClick={playFromCurrentMeasure}
          onPause={stopMeasurePlayback}
          shortcutLabel={playFromCurrentMeasureShortcutLabel}
        />
      )}
      {audioAvailable && (
        <PlayAllButton
          disabled={measureAudioGenerating || !soundfontReady}
          loading={measureAudioGenerating}
          playing={measureAudioPlaying}
          onClick={playAll}
          onPause={stopMeasurePlayback}
        />
      )}
      <div className="app-header-actions">
        {sharedFolder && <SharedFolderSwitcher viewer={sharedFolder} />}
        {!sharedPreview && !syncedShareViewerActive && (
          <FileSwitcher
            store={store}
            triggerLabel={store.active}
            onSelect={onSelect}
            onCreate={onCreate}
            onCreateFolder={onCreateFolder}
            onDuplicate={onDuplicate}
            onRename={onRename}
            onDelete={onDelete}
            onMoveFolder={onMoveFolder}
            onDeleteFolder={onDeleteFolder}
            onMoveSelection={onMoveSelection}
            onDeleteSelection={onDeleteSelection}
            onOpenStorageSettings={onOpenStorageSettings}
            saveStatus={saveStatus}
            saveLabel={saveLabel}
            autosaveDeadline={autosaveDeadline}
            creating={creatingFile}
            creatingFolder={creatingFolder}
            deletingName={deletingFileName}
            duplicating={duplicatingFile}
            renamingName={renamingFileName}
            isLoadingCloud={isLoadingCloud}
            cloudLoadFailure={cloudLoadFailure}
            importing={importing}
            onImportFile={onImportFile}
            binNames={sortedBinNames(store)}
            onOpenBin={onOpenBin}
            onOpenSyncPanel={onOpenSyncPanel}
            onOpenHistory={onOpenHistory}
            canSync={syncedShare.canSync}
            isSynced={syncedShare.isSynced}
            syncedShareLink={syncedShare.syncedShareLink}
            isSignedIn={syncedShare.isSignedIn}
            accountEmail={syncedShare.accountEmail}
            onStartSync={syncedShare.onStartSync}
            onStopSync={syncedShare.onStopSync}
            onSignIn={syncedShare.onSignIn}
            folderShares={syncedShare.folderShares}
          />
        )}
        <SyncedShareErrorDialog
          failure={syncedShare.syncFailure}
          onDismiss={syncedShare.onDismissSyncFailure}
        />
        <ExportControls
          hasDocuments={hasDocuments}
          rendering={rendering}
          audioGenerating={audioGenerating}
          wavUrl={wavUrl}
          soundfontReady={soundfontReady}
          onGenerateAudio={onGenerateAudio}
          pdfAvailable={pdfAvailable}
          pdfFontsReady={pdfFontsReady}
          pdfExporting={pdfExporting}
          onExportPdf={onExportPdf}
          splitPdfExporting={splitPdfExporting}
          onExportSplitPdf={onExportSplitPdf}
          midiAvailable={midiAvailable}
          midiExporting={midiExporting}
          onExportMidi={onExportMidi}
          splitMidiExporting={splitMidiExporting}
          onExportSplitMidi={onExportSplitMidi}
          audioAvailable={audioAvailable}
          splitWavExporting={splitWavExporting}
          onExportSplitWav={onExportSplitWav}
          mp3Available={mp3Available}
          mp3Exporting={mp3Exporting}
          mp3Url={mp3Url}
          onExportMp3={onExportMp3}
          splitMp3Exporting={splitMp3Exporting}
          onExportSplitMp3={onExportSplitMp3}
          partsCount={partsCount}
          isLoadingCloud={isLoadingCloud}
        />
        <ProfileControl
          isSignedIn={syncedShare.isSignedIn}
          accountEmail={syncedShare.accountEmail}
          isSynced={syncedShare.isSynced}
          onSignIn={syncedShare.onSignIn}
          onChangePassword={syncedShare.onChangePassword}
          onOpenApiKeys={syncedShare.onOpenApiKeys}
          onSignOut={onSignOut}
        />
      </div>
    </header>
  )
}
