import type { ByteRange, FontBytesByFamily } from '../poentaroWasm'
import { poentaroWasm, setWasmRoot } from '../poentaroWasm'
import type { EditorSelection, PartDeclaration } from '../types'
import { GM_INSTRUMENTS } from '../utils/gmInstruments'
import { instantiateWasmComponentFromModule } from '../wasmInit'
import {
  handleGenerateAudio,
  handleGenerateMeasureRangeAudio,
  handleGenerateMp3,
} from './audioMessageHandlers'
import {
  handleGenerateMidi,
  handleGeneratePdf,
  handleGenerateSplitMidi,
  handleGenerateSplitMp3,
  handleGenerateSplitPdf,
  handleGenerateSplitWav,
} from './exportMessageHandlers'
import { handleImportFromFile } from './importMessageHandlers'
import type { WorkerRequest, WorkerResponse } from './messages'
import { noteLabelsFor } from './noteLabels'
import {
  handlePreviewInstrument,
  handlePreviewPercussion,
} from './previewMessageHandlers'
import { staffClefFor } from './staffClef'
import {
  handleGenerateSplitStaffPdf,
  handleGenerateStaffPdf,
} from './staffExportHandlers'
import { handleRenderStaff } from './staffMessageHandlers'

export type { WorkerRequest, WorkerResponse } from './messages'

let resolveWasmModule: (module: WebAssembly.Module) => void
const wasmModulePromise = new Promise<WebAssembly.Module>((resolve) => {
  resolveWasmModule = resolve
})

let initPromise: Promise<void> | null = null

function ensureInit(): Promise<void> {
  if (!initPromise) {
    initPromise = wasmModulePromise
      .then((module) => instantiateWasmComponentFromModule(module))
      .then((root) => {
        setWasmRoot(root)
        postMessage({
          type: 'ready',
          audioAvailable: true,
          pdfAvailable: true,
          midiAvailable: true,
          mp3Available: true,
        } satisfies WorkerResponse)
      })
  }
  return initPromise
}

// Applies the layout fonts to the wasm module as soon as they've both
// arrived (from the `loadPdfFonts` message, reusing the bytes the app
// already fetches for PDF export) and the module is ready to receive them.
// Deliberately not awaited by any render: renders that happen before the
// fonts land just use the character-bucket fallback for that render, the
// same graceful degradation as a font fetch that fails outright. Blocking
// render on a network fetch would turn a slow or failed fetch into a stuck
// preview instead of a merely imprecise one.
function applyCoreFontsWhenReady(fonts: FontBytesByFamily): void {
  ensureInit().then(() => poentaroWasm().setLayoutFonts(fonts))
}

let loadedSoundfont: Uint8Array | null = null
let loadedFonts: FontBytesByFamily | null = null

function toByteRanges(ranges: EditorSelection[]): ByteRange[] {
  return ranges.map((range) => ({ startByte: range.start, endByte: range.end }))
}

function listDeclarationsFromSource(source: string): PartDeclaration[] {
  const result = poentaroWasm().listPartDeclarations(source, GM_INSTRUMENTS)
  return result.tag === 'ok' ? result.val.declarations : []
}

self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
  const msg = event.data

  if (msg.type === 'wasmModule') {
    resolveWasmModule(msg.module)
    return
  }

  if (msg.type === 'loadSoundfont') {
    loadedSoundfont = new Uint8Array(msg.soundfont)
    return
  }

  if (msg.type === 'loadPdfFonts') {
    loadedFonts = msg.fonts
    applyCoreFontsWhenReady(msg.fonts)
    return
  }

  await ensureInit()

  if (msg.type === 'listParts') {
    const result = poentaroWasm().listParts(msg.source, GM_INSTRUMENTS)
    if (result.tag === 'ok') {
      postMessage({
        type: 'parts',
        id: msg.id,
        parts: result.val.parts,
        declarations: result.val.declarations,
      } satisfies WorkerResponse)
      return
    }

    postMessage({
      type: 'parts',
      id: msg.id,
      parts: [],
      declarations: [],
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'updatePartDeclaration') {
    const newSource = poentaroWasm().updatePartDeclaration(
      msg.source,
      msg.abbreviation,
      msg.settings,
    )
    postMessage({
      type: 'partDeclarationUpdated',
      id: msg.id,
      source: newSource,
      declarations: listDeclarationsFromSource(newSource),
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'formatScore') {
    postMessage({
      type: 'scoreFormatted',
      id: msg.id,
      source: poentaroWasm().formatScore(msg.source),
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'shiftPartOctave') {
    postMessage({
      type: 'partOctaveShifted',
      id: msg.id,
      source: poentaroWasm().shiftPartOctave(
        msg.source,
        msg.abbreviation,
        msg.delta,
      ),
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'editRange') {
    const { operation } = msg
    const ranges = toByteRanges(msg.ranges)
    const result =
      operation.kind === 'shiftOctave'
        ? poentaroWasm().shiftRangeOctave(msg.source, ranges, operation.delta)
        : operation.kind === 'toggleSlur'
          ? poentaroWasm().toggleRangeSlur(msg.source, ranges)
          : poentaroWasm().toggleRangeTie(msg.source, ranges)
    postMessage({
      type: 'rangeEdited',
      id: msg.id,
      source: result.source,
      ranges: result.ranges.map((range) => ({
        start: range.startByte,
        end: range.endByte,
      })),
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'describeSelection') {
    postMessage({
      type: 'selectionDescribed',
      id: msg.id,
      description:
        poentaroWasm().describeSelection(
          msg.source,
          toByteRanges(msg.ranges),
        ) ?? null,
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'generatePdf') {
    const clef = staffClefFor(msg.notation)
    if (clef) void handleGenerateStaffPdf(msg, clef, loadedFonts)
    else handleGeneratePdf(msg, loadedFonts)
    return
  }

  if (msg.type === 'generateSplitPdf') {
    const clef = staffClefFor(msg.notation)
    if (clef) {
      void handleGenerateSplitStaffPdf(msg, clef, loadedFonts)
    } else handleGenerateSplitPdf(msg, loadedFonts)
    return
  }

  if (msg.type === 'generateMidi') {
    handleGenerateMidi(msg)
    return
  }

  if (msg.type === 'generateSplitMidi') {
    handleGenerateSplitMidi(msg)
    return
  }

  if (msg.type === 'generateSplitWav') {
    handleGenerateSplitWav(msg, loadedSoundfont)
    return
  }

  if (msg.type === 'generateMp3') {
    handleGenerateMp3(msg, loadedSoundfont)
    return
  }

  if (msg.type === 'generateSplitMp3') {
    handleGenerateSplitMp3(msg, loadedSoundfont)
    return
  }

  if (msg.type === 'generateAudio') {
    handleGenerateAudio(msg, loadedSoundfont)
    return
  }

  if (msg.type === 'generateMeasureRangeAudio') {
    handleGenerateMeasureRangeAudio(msg, loadedSoundfont)
    return
  }

  if (msg.type === 'previewInstrument') {
    handlePreviewInstrument(msg, loadedSoundfont)
    return
  }

  if (msg.type === 'previewPercussion') {
    handlePreviewPercussion(msg, loadedSoundfont)
    return
  }

  if (msg.type === 'renderWithHighlightRange') {
    const result = poentaroWasm().renderSvgWithHighlightRange(
      msg.source,
      msg.ranges,
      msg.visibility,
      GM_INSTRUMENTS,
      noteLabelsFor(msg.notation),
    )
    if (result.tag === 'ok') {
      postMessage({
        type: 'highlightRangeOk',
        id: msg.id,
        documents: result.val.documents,
      } satisfies WorkerResponse)
      return
    }
    postMessage({
      type: 'highlightRangeErr',
      id: msg.id,
      diagnostics: result.val.diagnostics,
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'renderStaff') {
    void handleRenderStaff(msg)
    return
  }

  if (msg.type === 'importFromFile') {
    handleImportFromFile(msg)
    return
  }

  if (msg.type === 'listMeasureSpans') {
    const result = poentaroWasm().listMeasureSpans(msg.source)
    postMessage({
      type: 'measureSpans',
      id: msg.id,
      status: result.tag,
      spans: result.tag === 'ok' ? result.val.spans : [],
      sectionRanges: result.tag === 'ok' ? result.val.sectionRanges : [],
      sequenceEntries: result.tag === 'ok' ? result.val.sequenceEntries : [],
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'listNoteSpans') {
    const result = poentaroWasm().listNoteSpans(msg.source, msg.enabledTracks)
    postMessage({
      type: 'noteSpans',
      id: msg.id,
      status: result.tag,
      spans: result.tag === 'ok' ? result.val.spans : [],
    } satisfies WorkerResponse)
    return
  }

  if (msg.type === 'listLyricSpans') {
    const result = poentaroWasm().listLyricSpans(msg.source, msg.enabledTracks)
    postMessage({
      type: 'lyricSpans',
      id: msg.id,
      status: result.tag,
      spans: result.tag === 'ok' ? result.val.spans : [],
    } satisfies WorkerResponse)
    return
  }

  if (msg.type !== 'render') return

  const result = poentaroWasm().renderSvg(
    msg.source,
    msg.visibility,
    GM_INSTRUMENTS,
    noteLabelsFor(msg.notation),
  )
  if (result.tag === 'ok') {
    postMessage({
      type: 'ok',
      id: msg.id,
      documents: result.val.documents,
      diagnostics: result.val.diagnostics,
      diagnosticViewZones: result.val.diagnosticViewZones,
    } satisfies WorkerResponse)
    return
  }

  postMessage({
    type: 'err',
    id: msg.id,
    diagnostics: result.val.diagnostics,
    diagnosticViewZones: result.val.diagnosticViewZones,
  } satisfies WorkerResponse)
}
