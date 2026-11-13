import type { RefObject } from 'react'
import type { FontBytesByFamily } from '../poentaroWasm'
import type { PartToggleState } from '../types'
import { usePoentaroWorkerActions } from './usePoentaroWorkerActions'
import { usePoentaroWorkerState } from './usePoentaroWorkerState'
import type { PoentaroWorkerState } from './usePoentaroWorkerTypes'
import type { SelectedSequenceRange } from './useSequenceNavigation'

export type { PoentaroWorkerState } from './usePoentaroWorkerTypes'

export function usePoentaroWorker(
  source: string,
  partToggles: PartToggleState,
  /** The preset chosen in the preset menu; `null` is Default. */
  chosenPreset: string | null,
  wasmReady: boolean,
  activeFile: string,
  soundfontBytes: Uint8Array | null,
  fontBytes: FontBytesByFamily | null,
  /**
   * Owned by the caller (`useAppController`, which also feeds it to its own
   * `useSequenceNavigation` call) rather than this hook, since
   * `useMeasureAudioPlayback` below needs it before `useSequenceNavigation`
   * can run — that hook needs `notifySelection`, which this hook only
   * produces further down. See `useSequenceNavigation`'s matching parameter
   * doc comment.
   */
  selectedSequenceRangeRef: RefObject<SelectedSequenceRange | null>,
  /**
   * Owned by the caller (`useAppController`) for the same reason as
   * `selectedSequenceRangeRef` above — `useMeasureAudioPlayback` needs it
   * before `useAppSelectionAndNavigation`'s `useMeasureRangeSelection` (which
   * derives it) can run. See `useAppSelectionAndNavigation`'s matching
   * parameter doc comment.
   */
  measureRangeSelectedPartNamesRef: RefObject<string[] | undefined>,
  debounceMs = 300,
): PoentaroWorkerState {
  const state = usePoentaroWorkerState(
    source,
    activeFile,
    partToggles,
    chosenPreset,
    wasmReady,
  )
  const {
    parts,
    visibility,
    partDeclarations,
    partsLoading,
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
    splitPdfExporting,
    midiAvailable,
    midiExporting,
    splitMidiExporting,
    splitWavExporting,
    mp3Available,
    mp3Exporting,
    splitMp3Exporting,
    diagnostics,
    diagnosticViewZones,
    rendering,
    audioGenerating,
    selectedMeasureRange,
    highlightedDocuments,
    measureSpans,
    noteSpans,
    lyricSpans,
    sectionRanges,
    sequenceEntries,
  } = state

  const actions = usePoentaroWorkerActions({
    state,
    selectedSequenceRangeRef,
    measureRangeSelectedPartNamesRef,
    source,
    activeFile,
    soundfontBytes,
    fontBytes,
    debounceMs,
  })

  return {
    parts,
    visibility,
    partDeclarations,
    partsLoading,
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
    splitPdfExporting,
    midiAvailable,
    midiExporting,
    splitMidiExporting,
    splitWavExporting,
    mp3Available,
    mp3Exporting,
    splitMp3Exporting,
    diagnostics,
    diagnosticViewZones,
    rendering,
    audioGenerating,
    exportPdf: actions.exportPdf,
    exportSplitPdf: actions.exportSplitPdf,
    exportMidi: actions.exportMidi,
    exportSplitMidi: actions.exportSplitMidi,
    exportSplitWav: actions.exportSplitWav,
    exportMp3: actions.exportMp3,
    exportSplitMp3: actions.exportSplitMp3,
    generateFullAudio: actions.generateFullAudio,
    selectedMeasureRange,
    measureAudioGenerating: actions.measureAudioGenerating,
    measureAudioPlaying: actions.measureAudioPlaying,
    measureAudioNoteTimings: actions.measureAudioNoteTimings,
    measureAudioElement: actions.measureAudioElement,
    notifySelection: actions.notifySelection,
    playSelectedMeasures: actions.playSelectedMeasures,
    playFromCurrentMeasure: actions.playFromCurrentMeasure,
    playNoteSelection: actions.playNoteSelection,
    playAll: actions.playAll,
    metronomeEnabled: actions.metronomeEnabled,
    setMetronomeEnabled: actions.setMetronomeEnabled,
    stopMeasurePlayback: actions.stopMeasurePlayback,
    highlightedDocuments,
    measureSpans,
    noteSpans,
    lyricSpans,
    sectionRanges,
    sequenceEntries,
    previewInstrument: actions.previewInstrument,
    previewPercussion: actions.previewPercussion,
    stopPreviewInstrument: actions.stopPreviewInstrument,
    previewAudioPlaying: actions.previewAudioPlaying,
    updatePartDeclaration: actions.updatePartDeclaration,
    formatScore: actions.formatScore,
    shiftPartOctave: actions.shiftPartOctave,
    editRange: actions.editRange,
    describeSelection: actions.describeSelection,
    importFromFile: actions.importFromFile,
    staffNotation: actions.staffNotation,
    previewLayout: actions.previewLayout,
  }
}
