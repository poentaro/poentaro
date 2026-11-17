import {
  type Diagnostic,
  type FontBytesByFamily,
  poentaroWasm,
} from '../poentaroWasm'
import { noteLabelsFor } from './noteLabels'
import type { WorkerRequest, WorkerResponse } from './poentaro.worker'
import { splitVocalsFor, type TrackVocals } from './singer'

type LoadedFonts = FontBytesByFamily | null

export function binaryBufferFromResult(bytes: Uint8Array): ArrayBuffer {
  return bytes.slice().buffer
}

export function singleErrorDiagnostic(message: string): Diagnostic[] {
  return [{ severity: 'error', message, span: { start: 0, end: 0 } }]
}

export function handleGeneratePdf(
  msg: Extract<WorkerRequest, { type: 'generatePdf' }>,
  loadedFonts: LoadedFonts,
): void {
  if (!loadedFonts) {
    postMessage({
      type: 'pdfErr',
      id: msg.id,
      diagnostics: singleErrorDiagnostic('Fonts are not yet loaded.'),
    } satisfies WorkerResponse)
    return
  }
  const result = poentaroWasm().generatePdf(
    msg.source,
    msg.visibility,
    loadedFonts,
    noteLabelsFor(msg.notation),
  )
  if (result.tag === 'ok') {
    const pdfBuffer = binaryBufferFromResult(result.val.pdf)
    postMessage(
      {
        type: 'pdf',
        id: msg.id,
        pdf: pdfBuffer,
      } satisfies WorkerResponse,
      { transfer: [pdfBuffer] },
    )
    return
  }

  postMessage({
    type: 'pdfErr',
    id: msg.id,
    diagnostics: result.val.diagnostics,
  } satisfies WorkerResponse)
}

export function handleGenerateSplitPdf(
  msg: Extract<WorkerRequest, { type: 'generateSplitPdf' }>,
  loadedFonts: LoadedFonts,
): void {
  if (!loadedFonts) {
    postMessage({
      type: 'splitPdfErr',
      id: msg.id,
      diagnostics: singleErrorDiagnostic('Fonts are not yet loaded.'),
    } satisfies WorkerResponse)
    return
  }
  const result = poentaroWasm().generateSplitPdfs(
    msg.source,
    msg.baseName,
    msg.presetName,
    loadedFonts,
    noteLabelsFor(msg.notation),
  )
  if (result.tag === 'ok') {
    const zipBuffer = binaryBufferFromResult(result.val.zip)
    postMessage(
      {
        type: 'splitPdf',
        id: msg.id,
        zip: zipBuffer,
      } satisfies WorkerResponse,
      { transfer: [zipBuffer] },
    )
    return
  }

  postMessage({
    type: 'splitPdfErr',
    id: msg.id,
    diagnostics: result.val.diagnostics,
  } satisfies WorkerResponse)
}

export function handleGenerateMidi(
  msg: Extract<WorkerRequest, { type: 'generateMidi' }>,
): void {
  const result = poentaroWasm().generateMidi(msg.source, msg.enabledTracks)
  if (result.tag === 'ok') {
    const midiBuffer = binaryBufferFromResult(result.val.midi)
    postMessage(
      {
        type: 'midi',
        id: msg.id,
        midi: midiBuffer,
      } satisfies WorkerResponse,
      { transfer: [midiBuffer] },
    )
    return
  }

  postMessage({
    type: 'midiErr',
    id: msg.id,
    diagnostics: result.val.diagnostics,
  } satisfies WorkerResponse)
}

export function handleGenerateSplitMidi(
  msg: Extract<WorkerRequest, { type: 'generateSplitMidi' }>,
): void {
  const result = poentaroWasm().generateSplitMidis(
    msg.source,
    msg.baseName,
    msg.presetName,
  )
  if (result.tag === 'ok') {
    const zipBuffer = binaryBufferFromResult(result.val.zip)
    postMessage(
      {
        type: 'splitMidi',
        id: msg.id,
        zip: zipBuffer,
      } satisfies WorkerResponse,
      { transfer: [zipBuffer] },
    )
    return
  }

  postMessage({
    type: 'splitMidiErr',
    id: msg.id,
    diagnostics: result.val.diagnostics,
  } satisfies WorkerResponse)
}

export async function handleGenerateSplitWav(
  msg: Extract<WorkerRequest, { type: 'generateSplitWav' }>,
  loadedSoundfont: Uint8Array | null,
): Promise<void> {
  if (!loadedSoundfont) {
    postMessage({
      type: 'splitWavErr',
      id: msg.id,
      diagnostics: singleErrorDiagnostic('Soundfont is not yet loaded.'),
    } satisfies WorkerResponse)
    return
  }
  let vocals: TrackVocals[]
  try {
    vocals = await splitVocalsFor(msg.source, msg.presetName)
  } catch (error) {
    postMessage({
      type: 'splitWavErr',
      id: msg.id,
      diagnostics: singleErrorDiagnostic(`Could not sing the lyrics: ${error}`),
    } satisfies WorkerResponse)
    return
  }
  const result = poentaroWasm().generateSplitWavs(
    msg.source,
    msg.baseName,
    msg.presetName,
    loadedSoundfont,
    vocals,
  )
  if (result.tag === 'ok') {
    const zipBuffer = binaryBufferFromResult(result.val.zip)
    postMessage(
      {
        type: 'splitWav',
        id: msg.id,
        zip: zipBuffer,
      } satisfies WorkerResponse,
      { transfer: [zipBuffer] },
    )
    return
  }

  postMessage({
    type: 'splitWavErr',
    id: msg.id,
    diagnostics: result.val.diagnostics,
  } satisfies WorkerResponse)
}

export async function handleGenerateSplitMp3(
  msg: Extract<WorkerRequest, { type: 'generateSplitMp3' }>,
  loadedSoundfont: Uint8Array | null,
): Promise<void> {
  if (!loadedSoundfont) {
    postMessage({
      type: 'splitMp3Err',
      id: msg.id,
      diagnostics: singleErrorDiagnostic('Soundfont is not yet loaded.'),
    } satisfies WorkerResponse)
    return
  }
  let vocals: TrackVocals[]
  try {
    vocals = await splitVocalsFor(msg.source, msg.presetName)
  } catch (error) {
    postMessage({
      type: 'splitMp3Err',
      id: msg.id,
      diagnostics: singleErrorDiagnostic(`Could not sing the lyrics: ${error}`),
    } satisfies WorkerResponse)
    return
  }
  const result = poentaroWasm().generateSplitMp3s(
    msg.source,
    msg.baseName,
    msg.presetName,
    loadedSoundfont,
    vocals,
  )
  if (result.tag === 'ok') {
    const zipBuffer = binaryBufferFromResult(result.val.zip)
    postMessage(
      {
        type: 'splitMp3',
        id: msg.id,
        zip: zipBuffer,
      } satisfies WorkerResponse,
      { transfer: [zipBuffer] },
    )
    return
  }

  postMessage({
    type: 'splitMp3Err',
    id: msg.id,
    diagnostics: result.val.diagnostics,
  } satisfies WorkerResponse)
}
