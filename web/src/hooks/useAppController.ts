import { useCallback, useRef, useState } from 'react'
import { fileIdForName, selectFile } from '../fileStore'
import type { EditorHandle } from '../types'
import { useAppPanels } from './useAppPanels'
import { useAppSelectionAndNavigation } from './useAppSelectionAndNavigation'
import { useAssetLoader } from './useAssetLoader'
import { useChosenPreset } from './useChosenPreset'
import { useDocumentTitle } from './useDocumentTitle'
import { useFileImport } from './useFileImport'
import { useFileOperations } from './useFileOperations'
import { useFontsLoader } from './useFontsLoader'
import { useKeyboardShortcuts } from './useKeyboardShortcuts'
import { usePartFocusSelectionRouting } from './usePartFocusSelectionRouting'
import { usePartTogglePruning } from './usePartTogglePruning'
import { usePartToggles } from './usePartToggles'
import { usePoentaroWorker } from './usePoentaroWorker'
import { usePresetMenu } from './usePresetMenu'
import { useScoreSource } from './useScoreSource'
import type { SelectedSequenceRange } from './useSequenceNavigation'
import { useStorageBackend } from './useStorageBackend'
import { useUrlFileSync } from './useUrlFileSync'
import { useWasmLoader } from './useWasmLoader'
import { noPartsSelected as computeNoPartsSelected } from './workerHelpers'

/** Wires every hook `App` needs into the one flat object its JSX renders
 * from. Split out of `App` to keep it under the line-count cap. */
export function useAppController() {
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
    flushPendingSave,
    refreshSaveStatus,
  } = useStorageBackend()
  const [editorCollapsed, setEditorCollapsed] = useState(false)
  useUrlFileSync(store, setStore, isLoadingCloud)

  const {
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
  } = useFileOperations(store, setStore, backend)
  const {
    sharedPreview,
    sharedFolder,
    syncedShareViewerActive,
    source,
    readOnly,
    syncedShare,
  } = useScoreSource(
    store,
    backend,
    setStore,
    setFileOpError,
    setEditorCollapsed,
    saveStatus,
  )
  const fileId = sharedFolder?.fileId ?? fileIdForName(store, store.active)
  useDocumentTitle(store.active)
  const editorRef = useRef<EditorHandle>(null)
  const { partFocusHandleRef, selectionEditorRef } =
    usePartFocusSelectionRouting(editorRef)
  const selectedSequenceRangeRef = useRef<SelectedSequenceRange | null>(null)
  const measureRangeSelectedPartNamesRef = useRef<string[] | undefined>(
    undefined,
  )
  const soundfont = useAssetLoader('/fonts/GeneralUser_GS.sf2')
  const fonts = useFontsLoader()
  const wasm = useWasmLoader()
  const soundfontReady = soundfont.status === 'ready'
  const pdfFontsReady = fonts.status === 'ready'

  // The part focus editor's last good draft. While it is set, everything
  // that reads the score (preview, spans, playback, pitch description)
  // follows it instead of the untouched `source`.
  const [partFocusDraft, setPartFocusDraft] = useState<string | null>(null)

  const {
    partToggles,
    setDisabledParts,
    setSoloedParts,
    handlePartToggle,
    handleSoloToggle,
  } = usePartToggles(fileId)
  const { chosenPreset, setChosenPreset } = useChosenPreset(
    fileId,
    source,
    wasm.status === 'ready',
  )

  const {
    parts,
    visibility,
    sectionRanges,
    sequenceEntries,
    playNoteSelection,
    updatePartDeclaration,
    formatScore,
    shiftPartOctave,
    editRange,
    importFromFile,
    staffNotation,
    notifySelection,
    measureSpans,
    noteSpans,
    lyricSpans,
    measureAudioPlaying,
    measureAudioGenerating,
    selectedMeasureRange,
    playSelectedMeasures,
    playFromCurrentMeasure,
    stopMeasurePlayback,
    ...workerResults
  } = usePoentaroWorker(
    partFocusDraft ?? source,
    partToggles,
    chosenPreset,
    wasm.status === 'ready',
    store.active,
    soundfont.bytes,
    fonts.fonts,
    selectedSequenceRangeRef,
    measureRangeSelectedPartNamesRef,
  )
  usePartTogglePruning(parts, setDisabledParts, setSoloedParts)

  const handleSourceChange = useCallback(
    (value: string) => {
      setStore((prev) => backend.updateActiveContent(prev, value))
    },
    [setStore, backend],
  )
  const handleSelect = useCallback(
    (name: string) => {
      flushPendingSave()
      setStore((prev) => selectFile(prev, name))
    },
    [setStore, flushPendingSave],
  )
  const handleFormatScore = useCallback(() => {
    void formatScore(source).then(handleSourceChange)
  }, [formatScore, source, handleSourceChange])
  const { importingFile, handleImportFile } = useFileImport(
    store,
    backend,
    setStore,
    setFileOpError,
    importFromFile,
  )
  const {
    handlePartPillToggle,
    handleSoloPillToggle,
    presetMenuProps,
    staffNotation: presetStaffNotation,
  } = usePresetMenu({
    source,
    readOnly,
    wasmReady: wasm.status === 'ready',
    parts,
    visibility,
    staffNotation,
    setChosenPreset,
    handlePartToggle,
    handleSoloToggle,
    handleSourceChange,
  })
  const panels = useAppPanels(
    source,
    updatePartDeclaration,
    shiftPartOctave,
    handleSourceChange,
    visibility.presetName ?? null,
  )

  const {
    setSelectedLineRange,
    handleSectionJump,
    sectionJumpToolbarProps,
    selectedSequenceRange,
    sequenceJumpToolbarProps,
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
    handleEditSelection,
  } = useAppSelectionAndNavigation(
    sectionRanges,
    selectionEditorRef,
    notifySelection,
    sequenceEntries,
    measureSpans,
    selectedSequenceRangeRef,
    noteSpans,
    lyricSpans,
    playNoteSelection,
    editRange,
    handleSourceChange,
    measureRangeSelectedPartNamesRef,
  )

  useKeyboardShortcuts({
    measureAudioPlaying,
    measureAudioGenerating,
    soundfontReady,
    selectedMeasureRange,
    selectedSequenceRange,
    playSelectedMeasures,
    playFromCurrentMeasure,
    notePlaybackSelectionActive: selectedNoteRangePlaybackInfo !== null,
    playNoteSelection: handlePlayNoteSelection,
    stopMeasurePlayback,
    forceSave,
  })

  const noPartsSelected = computeNoPartsSelected(parts, visibility)
  return {
    ...workerResults,
    staffNotation: presetStaffNotation,
    notifySelection,
    measureSpans,
    noteSpans,
    lyricSpans,
    measureAudioPlaying,
    measureAudioGenerating,
    selectedMeasureRange,
    playSelectedMeasures,
    playFromCurrentMeasure,
    stopMeasurePlayback,
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
    setEditorCollapsed,
    setPartFocusDraft,
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
    soloedParts: visibility.soloedParts,
    hiddenParts: visibility.hiddenParts,
    partToggles,
    handlePartPillToggle,
    handleSoloToggle: handleSoloPillToggle,
    presetMenuProps,
    parts,
    selectedSequenceRange,
    sequenceJumpToolbarProps,
    handleSourceChange,
    handleSelect,
    handleFormatScore,
    handleEditSelection,
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
  }
}
