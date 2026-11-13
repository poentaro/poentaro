import type { RefObject } from 'react'
import type { FontBytesByFamily } from '../poentaroWasm'
import { useInstrumentPreview } from './useInstrumentPreview'
import { useMeasureAudioPlayback } from './useMeasureAudioPlayback'
import { usePoentaroWorkerAudioActions } from './usePoentaroWorkerAudioActions'
import { usePoentaroWorkerDescribeSelection } from './usePoentaroWorkerDescribeSelection'
import { usePoentaroWorkerEditRange } from './usePoentaroWorkerEditRange'
import { usePoentaroWorkerExports } from './usePoentaroWorkerExports'
import { usePoentaroWorkerFormat } from './usePoentaroWorkerFormat'
import { usePoentaroWorkerImport } from './usePoentaroWorkerImport'
import { usePoentaroWorkerLifecycle } from './usePoentaroWorkerLifecycle'
import { usePoentaroWorkerPartDeclaration } from './usePoentaroWorkerPartDeclaration'
import { usePoentaroWorkerRenderRequests } from './usePoentaroWorkerRenderRequests'
import { usePoentaroWorkerShiftOctave } from './usePoentaroWorkerShiftOctave'
import type { usePoentaroWorkerState } from './usePoentaroWorkerState'
import { usePreviewLayout } from './usePreviewLayout'
import { useStaffNotation } from './useStaffNotation'

interface UsePoentaroWorkerActionsParams {
  state: ReturnType<typeof usePoentaroWorkerState>
  selectedSequenceRangeRef: RefObject<{
    start: number
    end: number
    entryStartIndex: number
    entryEndIndex: number
  } | null>
  measureRangeSelectedPartNamesRef: RefObject<string[] | undefined>
  source: string
  activeFile: string
  soundfontBytes: Uint8Array | null
  fontBytes: FontBytesByFamily | null
  debounceMs: number
}

/**
 * Wires every `usePoentaroWorker` sub-hook that isn't plain state
 * (`usePoentaroWorkerState`) — audio/measure/instrument playback, worker
 * lifecycle callbacks, render requests, exports, and the one-off
 * part-declaration/format/shift-octave/import actions — and returns
 * everything `usePoentaroWorker` needs beyond `state` itself to assemble its
 * public return value.
 *
 * Split out of `usePoentaroWorker` purely to stay under the 400-line file
 * cap; the two only make sense read together, and reads `state`'s fields
 * directly (rather than destructuring them into locals) so nothing here
 * shadows or drifts from the single `usePoentaroWorkerState` call in the
 * caller.
 */
export function usePoentaroWorkerActions({
  state,
  selectedSequenceRangeRef,
  measureRangeSelectedPartNamesRef,
  source,
  activeFile,
  soundfontBytes,
  fontBytes,
  debounceMs,
}: UsePoentaroWorkerActionsParams) {
  const { setNextWavUrl, setNextMp3Url, generateFullAudio } =
    usePoentaroWorkerAudioActions({
      workerRef: state.workerRef,
      sourceRef: state.sourceRef,
      enabledTracksRef: state.enabledTracksRef,
      wavUrlRef: state.wavUrlRef,
      setWavUrl: state.setWavUrl,
      mp3UrlRef: state.mp3UrlRef,
      setMp3Url: state.setMp3Url,
      audioGenerating: state.audioGenerating,
      setAudioGenerating: state.setAudioGenerating,
      audioRequestIdRef: state.audioRequestIdRef,
      latestAudioIdRef: state.latestAudioIdRef,
    })

  const {
    measureAudioGenerating,
    setMeasureAudioGenerating,
    measureAudioPlaying,
    measureAudioNoteTimings,
    measureAudioElement,
    setNextMeasureWavUrl,
    stopMeasurePlayback,
    playSelectedMeasures,
    playFromCurrentMeasure,
    playNoteSelection,
    playAll,
    metronomeEnabled,
    setMetronomeEnabled,
    latestMeasureAudioIdRef,
    measureWavUrlRef,
  } = useMeasureAudioPlayback({
    workerRef: state.workerRef,
    sourceRef: state.sourceRef,
    enabledTracksRef: state.enabledTracksRef,
    renderedTracksRef: state.renderedTracksRef,
    selectedMeasureRange: state.selectedMeasureRange,
    selectedSequenceRangeRef,
    measureRangeSelectedPartNamesRef,
    totalMeasures: state.measureSpans.length,
    sequenceEntryCount: state.sequenceEntries.length,
  })

  const {
    previewAudioPlaying,
    setPreviewAudioPlaying,
    previewInstrument,
    previewPercussion,
    stopPreviewInstrument,
    latestPreviewAudioIdRef,
    currentPreviewAudioRef,
  } = useInstrumentPreview({ workerRef: state.workerRef })

  usePoentaroWorkerLifecycle({
    workerRef: state.workerRef,
    wavUrlRef: state.wavUrlRef,
    mp3UrlRef: state.mp3UrlRef,
    measureWavUrlRef,
    cursorOffsetTimerRef: state.cursorOffsetTimerRef,
    soundfontBytes,
    fontBytes,
    audioAvailableRef: state.audioAvailableRef,
    setAudioAvailable: state.setAudioAvailable,
    setPdfAvailable: state.setPdfAvailable,
    setMidiAvailable: state.setMidiAvailable,
    setMp3Available: state.setMp3Available,
    latestPartsIdRef: state.latestPartsIdRef,
    setPartsLoading: state.setPartsLoading,
    setParts: state.setParts,
    setPartDeclarations: state.setPartDeclarations,
    latestUpdatePartDeclarationIdRef: state.latestUpdatePartDeclarationIdRef,
    pendingPartDeclarationUpdatesRef: state.pendingPartDeclarationUpdatesRef,
    latestFormatScoreIdRef: state.latestFormatScoreIdRef,
    pendingFormatScoreRequestsRef: state.pendingFormatScoreRequestsRef,
    shiftPartOctaveTracker: state.shiftPartOctaveTracker,
    editRangeTracker: state.editRangeTracker,
    describeSelectionTracker: state.describeSelectionTracker,
    staffRenderTracker: state.staffRenderTracker,
    latestPdfIdRef: state.latestPdfIdRef,
    setPdfExporting: state.setPdfExporting,
    activeFileRef: state.activeFileRef,
    filenameLabelsRef: state.filenameLabelsRef,
    visibilityRef: state.visibilityRef,
    setDiagnostics: state.setDiagnostics,
    latestSplitPdfIdRef: state.latestSplitPdfIdRef,
    setSplitPdfExporting: state.setSplitPdfExporting,
    latestMidiIdRef: state.latestMidiIdRef,
    setMidiExporting: state.setMidiExporting,
    latestSplitMidiIdRef: state.latestSplitMidiIdRef,
    setSplitMidiExporting: state.setSplitMidiExporting,
    latestSplitWavIdRef: state.latestSplitWavIdRef,
    setSplitWavExporting: state.setSplitWavExporting,
    latestMp3IdRef: state.latestMp3IdRef,
    setMp3Exporting: state.setMp3Exporting,
    latestSplitMp3IdRef: state.latestSplitMp3IdRef,
    setSplitMp3Exporting: state.setSplitMp3Exporting,
    latestRenderIdRef: state.latestRenderIdRef,
    setRendering: state.setRendering,
    setDocuments: state.setDocuments,
    setDiagnosticViewZones: state.setDiagnosticViewZones,
    latestAudioIdRef: state.latestAudioIdRef,
    setAudioGenerating: state.setAudioGenerating,
    setNextWavUrl,
    setNextMp3Url,
    setNoteTimings: state.setNoteTimings,
    latestMeasureAudioIdRef,
    setMeasureAudioGenerating,
    setNextMeasureWavUrl,
    latestHighlightRenderIdRef: state.latestHighlightRenderIdRef,
    setHighlightedDocuments: state.setHighlightedDocuments,
    latestMeasureSpansIdRef: state.latestMeasureSpansIdRef,
    setMeasureSpans: state.setMeasureSpans,
    latestNoteSpansIdRef: state.latestNoteSpansIdRef,
    setNoteSpans: state.setNoteSpans,
    latestLyricSpansIdRef: state.latestLyricSpansIdRef,
    setLyricSpans: state.setLyricSpans,
    setSectionRanges: state.setSectionRanges,
    setSequenceEntries: state.setSequenceEntries,
    latestPreviewAudioIdRef,
    currentPreviewAudioRef,
    setPreviewAudioPlaying,
    pendingImportsRef: state.pendingImportsRef,
    requestDownload: state.requestDownload,
  })

  const previewLayout = usePreviewLayout()
  const staffNotation = useStaffNotation({
    workerRef: state.workerRef,
    source,
    visibility: state.visibility,
    pageLayout: previewLayout.pageLayout,
    staffRenderTracker: state.staffRenderTracker,
    debounceMs,
  })

  const { notifySelection } = usePoentaroWorkerRenderRequests({
    workerRef: state.workerRef,
    sourceRef: state.sourceRef,
    source,
    activeFile,
    debounceMs,
    cursorOffsetTimerRef: state.cursorOffsetTimerRef,
    visibility: state.visibility,
    notation: staffNotation.notation,
    pageLayout: previewLayout.pageLayout,
    renderedTracks: state.renderedTracks,
    setDocuments: state.setDocuments,
    setNextWavUrl,
    setNextMp3Url,
    setDiagnostics: state.setDiagnostics,
    setPartsLoading: state.setPartsLoading,
    partsRequestIdRef: state.partsRequestIdRef,
    latestPartsIdRef: state.latestPartsIdRef,
    setRendering: state.setRendering,
    renderRequestIdRef: state.renderRequestIdRef,
    latestRenderIdRef: state.latestRenderIdRef,
    selectedMeasureRange: state.selectedMeasureRange,
    setSelectedMeasureRange: state.setSelectedMeasureRange,
    setHighlightedDocuments: state.setHighlightedDocuments,
    highlightRenderRequestIdRef: state.highlightRenderRequestIdRef,
    latestHighlightRenderIdRef: state.latestHighlightRenderIdRef,
    measureSpans: state.measureSpans,
    measureSpansRef: state.measureSpansRef,
    measureSpansRequestIdRef: state.measureSpansRequestIdRef,
    latestMeasureSpansIdRef: state.latestMeasureSpansIdRef,
    noteSpansRequestIdRef: state.noteSpansRequestIdRef,
    latestNoteSpansIdRef: state.latestNoteSpansIdRef,
    lyricSpansRequestIdRef: state.lyricSpansRequestIdRef,
    latestLyricSpansIdRef: state.latestLyricSpansIdRef,
    lastSelectionRef: state.lastSelectionRef,
  })

  const {
    exportPdf,
    exportSplitPdf,
    exportMidi,
    exportSplitMidi,
    exportSplitWav,
    exportMp3,
    exportSplitMp3,
  } = usePoentaroWorkerExports({
    workerRef: state.workerRef,
    sourceRef: state.sourceRef,
    activeFileRef: state.activeFileRef,
    enabledTracksRef: state.enabledTracksRef,
    visibilityRef: state.visibilityRef,
    notation: staffNotation.notation,
    pdfExporting: state.pdfExporting,
    splitPdfExporting: state.splitPdfExporting,
    midiExporting: state.midiExporting,
    splitMidiExporting: state.splitMidiExporting,
    splitWavExporting: state.splitWavExporting,
    mp3Exporting: state.mp3Exporting,
    splitMp3Exporting: state.splitMp3Exporting,
    setPdfExporting: state.setPdfExporting,
    setSplitPdfExporting: state.setSplitPdfExporting,
    setMidiExporting: state.setMidiExporting,
    setSplitMidiExporting: state.setSplitMidiExporting,
    setSplitWavExporting: state.setSplitWavExporting,
    setMp3Exporting: state.setMp3Exporting,
    setSplitMp3Exporting: state.setSplitMp3Exporting,
    pdfRequestIdRef: state.pdfRequestIdRef,
    latestPdfIdRef: state.latestPdfIdRef,
    splitPdfRequestIdRef: state.splitPdfRequestIdRef,
    latestSplitPdfIdRef: state.latestSplitPdfIdRef,
    midiRequestIdRef: state.midiRequestIdRef,
    latestMidiIdRef: state.latestMidiIdRef,
    splitMidiRequestIdRef: state.splitMidiRequestIdRef,
    latestSplitMidiIdRef: state.latestSplitMidiIdRef,
    splitWavRequestIdRef: state.splitWavRequestIdRef,
    latestSplitWavIdRef: state.latestSplitWavIdRef,
    mp3RequestIdRef: state.mp3RequestIdRef,
    latestMp3IdRef: state.latestMp3IdRef,
    splitMp3RequestIdRef: state.splitMp3RequestIdRef,
    latestSplitMp3IdRef: state.latestSplitMp3IdRef,
  })

  const { updatePartDeclaration } = usePoentaroWorkerPartDeclaration({
    workerRef: state.workerRef,
    sourceRef: state.sourceRef,
    updatePartDeclarationRequestIdRef: state.updatePartDeclarationRequestIdRef,
    latestUpdatePartDeclarationIdRef: state.latestUpdatePartDeclarationIdRef,
    pendingPartDeclarationUpdatesRef: state.pendingPartDeclarationUpdatesRef,
  })

  const { formatScore } = usePoentaroWorkerFormat({
    workerRef: state.workerRef,
    formatScoreRequestIdRef: state.formatScoreRequestIdRef,
    latestFormatScoreIdRef: state.latestFormatScoreIdRef,
    pendingFormatScoreRequestsRef: state.pendingFormatScoreRequestsRef,
  })

  const { shiftPartOctave } = usePoentaroWorkerShiftOctave({
    workerRef: state.workerRef,
    sourceRef: state.sourceRef,
    shiftPartOctaveTracker: state.shiftPartOctaveTracker,
  })

  const { editRange } = usePoentaroWorkerEditRange({
    workerRef: state.workerRef,
    sourceRef: state.sourceRef,
    editRangeTracker: state.editRangeTracker,
  })

  const { describeSelection } = usePoentaroWorkerDescribeSelection({
    workerRef: state.workerRef,
    sourceRef: state.sourceRef,
    describeSelectionTracker: state.describeSelectionTracker,
  })

  const { importFromFile } = usePoentaroWorkerImport({
    workerRef: state.workerRef,
    importRequestIdRef: state.importRequestIdRef,
    pendingImportsRef: state.pendingImportsRef,
  })

  return {
    generateFullAudio,
    measureAudioGenerating,
    measureAudioPlaying,
    measureAudioNoteTimings,
    measureAudioElement,
    stopMeasurePlayback,
    playSelectedMeasures,
    playFromCurrentMeasure,
    playNoteSelection,
    playAll,
    metronomeEnabled,
    setMetronomeEnabled,
    notifySelection,
    exportPdf,
    exportSplitPdf,
    exportMidi,
    exportSplitMidi,
    exportSplitWav,
    exportMp3,
    exportSplitMp3,
    updatePartDeclaration,
    formatScore,
    shiftPartOctave,
    editRange,
    describeSelection,
    importFromFile,
    staffNotation,
    previewLayout,
    previewInstrument,
    previewPercussion,
    stopPreviewInstrument,
    previewAudioPlaying,
  }
}
