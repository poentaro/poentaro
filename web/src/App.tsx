import { useCallback } from 'react'
import { AppHeader } from './components/AppHeader'
import { AppOverlays } from './components/AppOverlays'
import { AppWorkspace } from './components/AppWorkspace'
import { AssetLoadingBanner } from './components/AssetLoadingBanner'
import { ExportAudioToast } from './components/ExportAudioToast'
import { PartToggles } from './components/PartToggles'
import { PresetMenu } from './components/PresetMenu'
import { SectionJumpToolbar } from './components/SectionJumpToolbar'
import { SequenceJumpToolbar } from './components/SequenceJumpToolbar'
import {
  SharedViewerNotice,
  showsSharedViewerNotice,
} from './components/SharedViewerNotice'
import { useAppController } from './hooks/useAppController'
import './App.css'
import './file-switcher.css'
import './preview.css'
import './shared-viewer.css'
import './preview-toolbar.css'
import './preview-play-controls.css'

export default function App() {
  const {
    store,
    setStore,
    backend,
    isLoadingCloud,
    cloudLoadFailure,
    saveStatus,
    saveLabel,
    autosaveDeadline,
    preference,
    switchBackend,
    forceSave,
    refreshSaveStatus,
    editorCollapsed,
    setPartFocusDraft,
    setEditorCollapsed,
    creatingFile,
    creatingFolder,
    deletingFileName,
    duplicatingFile,
    renamingFileName,
    restoringFileName,
    fileOpError,
    setFileOpError,
    handleCreate,
    handleCreateFolder,
    handleDuplicate,
    handleRename,
    handleDelete,
    handleMoveFolder,
    handleDeleteFolder,
    handleMoveSelection,
    handleDeleteSelection,
    handleRestore,
    sharedPreview,
    syncedShareViewerActive,
    source,
    readOnly,
    syncedShare,
    fileId,
    editorRef,
    partFocusHandleRef,
    soundfont,
    fonts,
    wasm,
    soundfontReady,
    pdfFontsReady,
    hiddenParts,
    soloedParts,
    partToggles,
    handlePartPillToggle,
    presetMenuProps,
    handleSoloToggle,
    parts,
    partDeclarations,
    documents,
    pendingDownload,
    requestDownload,
    confirmPendingDownload,
    cancelPendingDownload,
    wavUrl,
    wavFilename,
    mp3Url,
    mp3Filename,
    noteTimings,
    audioAvailable,
    pdfAvailable,
    pdfExporting,
    diagnostics,
    diagnosticViewZones,
    rendering,
    audioGenerating,
    exportPdf,
    splitPdfExporting,
    exportSplitPdf,
    midiAvailable,
    midiExporting,
    exportMidi,
    splitMidiExporting,
    exportSplitMidi,
    splitWavExporting,
    exportSplitWav,
    mp3Available,
    mp3Exporting,
    exportMp3,
    splitMp3Exporting,
    exportSplitMp3,
    generateFullAudio,
    selectedMeasureRange,
    measureAudioGenerating,
    measureAudioPlaying,
    measureAudioNoteTimings,
    measureAudioElement,
    measureSpans,
    selectedSequenceRange,
    sequenceJumpToolbarProps,
    notifySelection,
    playSelectedMeasures,
    playFromCurrentMeasure,
    playAll,
    metronomeEnabled,
    setMetronomeEnabled,
    stopMeasurePlayback,
    highlightedDocuments,
    noteSpans,
    lyricSpans,
    previewInstrument,
    previewPercussion,
    stopPreviewInstrument,
    previewAudioPlaying,
    handleSourceChange,
    handleSelect,
    handleFormatScore,
    handleEditSelection,
    describeSelection,
    importingFile,
    handleImportFile,
    panels,
    setSelectedLineRange,
    handleSectionJump,
    sectionJumpToolbarProps,
    handleNoteRangeSelect,
    handleEditorSelectionChange,
    selectedNoteRangePlaybackInfo,
    selectedNoteCells,
    selectedNoteByteRanges,
    handleLyricRangeSelect,
    handleLyricEditorSelectionChange,
    selectedLyricCells,
    handleMeasureRangeSelect,
    handlePlayNoteSelection,
    noPartsSelected,
    staffNotation,
    previewLayout,
  } = useAppController()
  const previewInteractive = wasm.status === 'ready'
  const handleAccountSignOut = useCallback(() => {
    syncedShare.onSignOut()
    if (preference.backend === 'cloud') void switchBackend({ kind: 'local' })
  }, [syncedShare, preference, switchBackend])
  return (
    <div className="app">
      <AssetLoadingBanner
        soundfontStatus={soundfont.status}
        soundfontLoadedBytes={soundfont.loadedBytes}
        soundfontTotalBytes={soundfont.totalBytes}
        fontsStatus={fonts.status}
        fontsLoadedBytes={fonts.loadedBytes}
        fontsTotalBytes={fonts.totalBytes}
        wasmStatus={wasm.status}
        wasmLoadedBytes={wasm.loadedBytes}
        wasmTotalBytes={wasm.totalBytes}
      />
      <AppHeader
        audioAvailable={audioAvailable}
        selectedMeasureRange={selectedMeasureRange}
        selectedSequenceRange={selectedSequenceRange}
        measureAudioGenerating={measureAudioGenerating}
        soundfontReady={soundfontReady}
        measureAudioPlaying={measureAudioPlaying}
        playSelectedMeasures={playSelectedMeasures}
        playFromCurrentMeasure={playFromCurrentMeasure}
        playAll={playAll}
        metronomeEnabled={metronomeEnabled}
        onMetronomeEnabledChange={setMetronomeEnabled}
        notePlaybackSelectionActive={selectedNoteRangePlaybackInfo !== null}
        playNoteSelection={handlePlayNoteSelection}
        stopMeasurePlayback={stopMeasurePlayback}
        store={store}
        onSelect={handleSelect}
        onCreate={handleCreate}
        onCreateFolder={handleCreateFolder}
        onDuplicate={handleDuplicate}
        onRename={handleRename}
        onDelete={handleDelete}
        onMoveFolder={handleMoveFolder}
        onDeleteFolder={handleDeleteFolder}
        onMoveSelection={handleMoveSelection}
        onDeleteSelection={handleDeleteSelection}
        onOpenStorageSettings={() => panels.setStorageSettingsOpen(true)}
        saveStatus={saveStatus}
        saveLabel={saveLabel}
        autosaveDeadline={autosaveDeadline}
        creatingFile={creatingFile}
        creatingFolder={creatingFolder}
        deletingFileName={deletingFileName}
        duplicatingFile={duplicatingFile}
        renamingFileName={renamingFileName}
        isLoadingCloud={isLoadingCloud}
        cloudLoadFailure={cloudLoadFailure}
        onOpenBin={() => panels.setBinOpen(true)}
        onOpenSyncPanel={
          backend.kind === 'cloud'
            ? () => panels.setSyncPanelOpen(true)
            : undefined
        }
        onOpenHistory={
          backend.kind === 'cloud'
            ? () => panels.setHistoryOpen(true)
            : undefined
        }
        hasDocuments={documents.length > 0}
        rendering={rendering}
        audioGenerating={audioGenerating}
        wavUrl={wavUrl}
        onGenerateAudio={generateFullAudio}
        pdfAvailable={pdfAvailable}
        pdfFontsReady={pdfFontsReady}
        pdfExporting={pdfExporting}
        onExportPdf={exportPdf}
        splitPdfExporting={splitPdfExporting}
        onExportSplitPdf={exportSplitPdf}
        midiAvailable={midiAvailable}
        midiExporting={midiExporting}
        onExportMidi={exportMidi}
        splitMidiExporting={splitMidiExporting}
        onExportSplitMidi={exportSplitMidi}
        splitWavExporting={splitWavExporting}
        onExportSplitWav={exportSplitWav}
        mp3Available={mp3Available}
        mp3Exporting={mp3Exporting}
        mp3Url={mp3Url}
        onExportMp3={exportMp3}
        splitMp3Exporting={splitMp3Exporting}
        onExportSplitMp3={exportSplitMp3}
        partsCount={parts.length}
        importing={importingFile}
        onImportFile={handleImportFile}
        syncedShare={syncedShare}
        onSignOut={handleAccountSignOut}
      />
      <AppOverlays
        fileOpError={fileOpError}
        setFileOpError={setFileOpError}
        storageSettingsOpen={panels.storageSettingsOpen}
        setStorageSettingsOpen={panels.setStorageSettingsOpen}
        backend={backend}
        isLoadingCloud={isLoadingCloud}
        preference={preference}
        switchBackend={switchBackend}
        store={store}
        setStore={setStore}
        refreshSaveStatus={refreshSaveStatus}
        selectedMeasureRange={selectedMeasureRange}
        binOpen={panels.binOpen}
        setBinOpen={panels.setBinOpen}
        syncPanelOpen={panels.syncPanelOpen}
        hasUnsavedEdits={autosaveDeadline !== null}
        setSyncPanelOpen={panels.setSyncPanelOpen}
        historyOpen={panels.historyOpen}
        setHistoryOpen={panels.setHistoryOpen}
        fileId={fileId}
        onRestoreVersion={handleSourceChange}
        onRestore={handleRestore}
        restoringFileName={restoringFileName}
        pendingDownload={pendingDownload}
        onConfirmDownload={confirmPendingDownload}
        onCancelDownload={cancelPendingDownload}
        accountDialog={syncedShare.accountDialog}
        onCloseAccountDialog={syncedShare.onCloseAccountDialog}
        onSignIn={syncedShare.onSignIn}
      />
      <ExportAudioToast
        open={
          audioGenerating ||
          mp3Exporting ||
          splitWavExporting ||
          splitMp3Exporting
        }
        label={
          audioGenerating
            ? 'Generating WAV…'
            : mp3Exporting
              ? 'Generating MP3…'
              : splitWavExporting
                ? 'Exporting WAV (ZIP)…'
                : 'Exporting MP3 (ZIP)…'
        }
      />
      <SectionJumpToolbar {...sectionJumpToolbarProps} />
      <SequenceJumpToolbar {...sequenceJumpToolbarProps} />
      <PartToggles
        parts={parts}
        hiddenParts={new Set(hiddenParts)}
        presetMenu={<PresetMenu {...presetMenuProps} />}
        soloedParts={new Set(soloedParts)}
        onPartToggle={handlePartPillToggle}
        onSoloToggle={handleSoloToggle}
      />
      {syncedShareViewerActive &&
      showsSharedViewerNotice(syncedShare.viewerStatus) ? (
        <SharedViewerNotice
          status={syncedShare.viewerStatus}
          traceId={syncedShare.viewerTraceId}
        />
      ) : (
        <AppWorkspace
          editorCollapsed={editorCollapsed}
          setEditorCollapsed={setEditorCollapsed}
          hideEditor={sharedPreview !== null || syncedShareViewerActive}
          editorRef={editorRef}
          partFocusHandleRef={partFocusHandleRef}
          fileId={fileId}
          source={source}
          handleSourceChange={handleSourceChange}
          handleFormatScore={handleFormatScore}
          handleEditSelection={handleEditSelection}
          describeSelection={describeSelection}
          readOnly={readOnly}
          diagnostics={diagnostics}
          diagnosticViewZones={diagnosticViewZones}
          measureSpans={measureSpans}
          setSelectedLineRange={setSelectedLineRange}
          notifySelection={notifySelection}
          setEditPartsOpen={panels.setEditPartsOpen}
          setEditMetadataOpen={panels.setEditMetadataOpen}
          forceSave={forceSave}
          measureAudioPlaying={measureAudioPlaying}
          stopMeasurePlayback={stopMeasurePlayback}
          selectedMeasureRange={selectedMeasureRange}
          measureAudioGenerating={measureAudioGenerating}
          soundfontReady={soundfontReady}
          playSelectedMeasures={playSelectedMeasures}
          notePlaybackSelectionActive={selectedNoteRangePlaybackInfo !== null}
          playNoteSelection={handlePlayNoteSelection}
          editPartsOpen={panels.editPartsOpen}
          partDeclarations={partDeclarations}
          parts={parts}
          handlePartDeclarationChange={panels.handlePartDeclarationChange}
          handleShiftPartOctave={panels.handleShiftPartOctave}
          previewInstrument={previewInstrument}
          previewPercussion={previewPercussion}
          stopPreviewInstrument={stopPreviewInstrument}
          previewAudioPlaying={previewAudioPlaying}
          editMetadataOpen={panels.editMetadataOpen}
          parsedMetadata={panels.parsedMetadata}
          metadataPresetName={panels.metadataPresetName}
          handleMetadataFieldChange={panels.handleMetadataFieldChange}
          // The preview's click/hover resolvers call `poentaroWasm()`
          // synchronously, so its SVG only appears once the main-thread
          // component is instantiated — a click can never land before then.
          documents={previewInteractive ? documents : []}
          highlightedDocuments={previewInteractive ? highlightedDocuments : []}
          rendering={rendering || !previewInteractive}
          handleSectionJump={handleSectionJump}
          handleNoteRangeSelect={handleNoteRangeSelect}
          handleEditorSelectionChange={handleEditorSelectionChange}
          selectedNoteCells={selectedNoteCells}
          selectedNoteByteRanges={selectedNoteByteRanges}
          noteSpans={noteSpans}
          handleLyricRangeSelect={handleLyricRangeSelect}
          handleLyricEditorSelectionChange={handleLyricEditorSelectionChange}
          selectedLyricCells={selectedLyricCells}
          lyricSpans={lyricSpans}
          handleMeasureRangeSelect={handleMeasureRangeSelect}
          audioGenerating={audioGenerating}
          wavUrl={wavUrl}
          wavFilename={wavFilename}
          mp3Exporting={mp3Exporting}
          mp3Url={mp3Url}
          mp3Filename={mp3Filename}
          onRequestAudioDownload={(url, filename) =>
            requestDownload(url, filename, false)
          }
          noteTimings={noteTimings}
          measureAudioNoteTimings={measureAudioNoteTimings}
          measureAudioElement={measureAudioElement}
          noPartsSelected={noPartsSelected}
          staffNotation={staffNotation}
          previewLayout={previewLayout}
          partToggles={partToggles}
          onPartFocusDraftChange={setPartFocusDraft}
        />
      )}
    </div>
  )
}
