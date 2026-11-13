import {
  type Diagnostic,
  type FontBytesByFamily,
  poentaroWasm,
  type StaffClef,
} from '../poentaroWasm'
import { GM_INSTRUMENTS } from '../utils/gmInstruments'
import {
  binaryBufferFromResult,
  singleErrorDiagnostic,
} from './exportMessageHandlers'
import type { WorkerRequest, WorkerResponse } from './messages'
import { engraveMei } from './staffMessageHandlers'

type LoadedFonts = FontBytesByFamily | null

const FONTS_NOT_LOADED = 'Fonts are not yet loaded.'

/** PDF export while the preview shows staff notation: Rust writes the MEI,
 * Verovio engraves it, and Rust writes Verovio's pages as the PDF. Replies
 * with the same `pdf`/`pdfErr` messages as the numbered export. */
export async function handleGenerateStaffPdf(
  msg: Extract<WorkerRequest, { type: 'generatePdf' }>,
  clef: StaffClef,
  loadedFonts: LoadedFonts,
): Promise<void> {
  const fail = (diagnostics: Diagnostic[]) =>
    postMessage({
      type: 'pdfErr',
      id: msg.id,
      diagnostics,
    } satisfies WorkerResponse)
  if (!loadedFonts) return fail(singleErrorDiagnostic(FONTS_NOT_LOADED))
  const mei = poentaroWasm().renderMei(
    msg.source,
    msg.visibility,
    GM_INSTRUMENTS,
    clef,
  )
  if (mei.tag === 'err') return fail(mei.val.diagnostics)
  const engraved = await engraveMei(mei.val.mei, { tag: 'a4' })
  if (engraved.status === 'err') {
    return fail(singleErrorDiagnostic(engraved.message))
  }
  const result = poentaroWasm().generateStaffPdf(
    engraved.pages,
    msg.source,
    loadedFonts,
  )
  if (result.tag === 'err') return fail(result.val.diagnostics)
  const pdf = binaryBufferFromResult(result.val.pdf)
  postMessage({ type: 'pdf', id: msg.id, pdf } satisfies WorkerResponse, {
    transfer: [pdf],
  })
}

/** The staff counterpart of the per-part PDF ZIP export. */
export async function handleGenerateSplitStaffPdf(
  msg: Extract<WorkerRequest, { type: 'generateSplitPdf' }>,
  clef: StaffClef,
  loadedFonts: LoadedFonts,
): Promise<void> {
  const fail = (diagnostics: Diagnostic[]) =>
    postMessage({
      type: 'splitPdfErr',
      id: msg.id,
      diagnostics,
    } satisfies WorkerResponse)
  if (!loadedFonts) return fail(singleErrorDiagnostic(FONTS_NOT_LOADED))
  const meis = poentaroWasm().renderSplitMeis(
    msg.source,
    msg.baseName,
    msg.presetName,
    GM_INSTRUMENTS,
    clef,
  )
  if (meis.tag === 'err') return fail(meis.val.diagnostics)
  const parts = []
  for (const part of meis.val) {
    const engraved = await engraveMei(part.mei, { tag: 'a4' })
    if (engraved.status === 'err') {
      return fail(singleErrorDiagnostic(engraved.message))
    }
    parts.push({
      trackName: part.trackName,
      fileName: part.fileName,
      pages: engraved.pages,
    })
  }
  const result = poentaroWasm().generateSplitStaffPdfs(parts, loadedFonts)
  if (result.tag === 'err') return fail(result.val.diagnostics)
  const zip = binaryBufferFromResult(result.val.zip)
  postMessage({ type: 'splitPdf', id: msg.id, zip } satisfies WorkerResponse, {
    transfer: [zip],
  })
}
