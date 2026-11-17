import type { WorkerResponse } from '../worker/poentaro.worker'
import type { WorkerMessageHandlerDeps } from './usePoentaroWorkerMessageHandler'
import {
  midiFilenameFromActiveFile,
  objectUrlForBytes,
  pdfFilenameFromActiveFile,
  zipFilenameFromActiveFile,
} from './workerHelpers'

/** A per-part ZIP's filename labels: the preset in force, then `kind`. */
function zipLabels(deps: WorkerMessageHandlerDeps, kind?: string) {
  return [deps.visibilityRef.current.presetName, kind].filter(
    (label): label is string => label != null,
  )
}

/** Handles the twelve "export finished"/"export failed" message pairs that
 * open the rename-before-download modal (`pdf`, `splitPdf`, `midi`,
 * `splitMidi`, `splitWav`, `splitMp3`, each with its own `...Err` variant).
 * Split out of `usePoentaroWorkerMessageHandler.ts` purely to keep that
 * file's dispatch loop under the line-count limit — these six pairs are
 * one repeated shape (check the id is still current, stop the "exporting"
 * spinner, then either open the download modal or surface diagnostics).
 * Returns `true` if `msg` was one of these twelve types (handled either
 * way, including a stale id), so the caller knows not to fall through to
 * its own dispatch. */
export function handleExportMessage(
  msg: WorkerResponse,
  deps: WorkerMessageHandlerDeps,
): boolean {
  if (msg.type === 'pdf') {
    if (msg.id !== deps.latestPdfIdRef.current) return true
    deps.setPdfExporting(false)
    deps.requestDownload(
      objectUrlForBytes(msg.pdf, 'application/pdf'),
      pdfFilenameFromActiveFile(
        deps.activeFileRef.current,
        deps.filenameLabelsRef.current,
      ),
      true,
    )
    return true
  }

  if (msg.type === 'pdfErr') {
    if (msg.id !== deps.latestPdfIdRef.current) return true
    deps.setPdfExporting(false)
    deps.setDiagnostics(msg.diagnostics)
    return true
  }

  if (msg.type === 'splitPdf') {
    if (msg.id !== deps.latestSplitPdfIdRef.current) return true
    deps.setSplitPdfExporting(false)
    deps.requestDownload(
      objectUrlForBytes(msg.zip, 'application/zip'),
      zipFilenameFromActiveFile(deps.activeFileRef.current, zipLabels(deps)),
      true,
    )
    return true
  }

  if (msg.type === 'splitPdfErr') {
    if (msg.id !== deps.latestSplitPdfIdRef.current) return true
    deps.setSplitPdfExporting(false)
    deps.setDiagnostics(msg.diagnostics)
    return true
  }

  if (msg.type === 'midi') {
    if (msg.id !== deps.latestMidiIdRef.current) return true
    deps.setMidiExporting(false)
    deps.requestDownload(
      objectUrlForBytes(msg.midi, 'audio/midi'),
      midiFilenameFromActiveFile(
        deps.activeFileRef.current,
        deps.filenameLabelsRef.current,
      ),
      true,
    )
    return true
  }

  if (msg.type === 'midiErr') {
    if (msg.id !== deps.latestMidiIdRef.current) return true
    deps.setMidiExporting(false)
    deps.setDiagnostics(msg.diagnostics)
    return true
  }

  if (msg.type === 'splitMidi') {
    if (msg.id !== deps.latestSplitMidiIdRef.current) return true
    deps.setSplitMidiExporting(false)
    deps.requestDownload(
      objectUrlForBytes(msg.zip, 'application/zip'),
      zipFilenameFromActiveFile(
        deps.activeFileRef.current,
        zipLabels(deps, 'MIDI parts'),
      ),
      true,
    )
    return true
  }

  if (msg.type === 'splitMidiErr') {
    if (msg.id !== deps.latestSplitMidiIdRef.current) return true
    deps.setSplitMidiExporting(false)
    deps.setDiagnostics(msg.diagnostics)
    return true
  }

  if (msg.type === 'splitWav') {
    if (msg.id !== deps.latestSplitWavIdRef.current) return true
    deps.setSplitWavExporting(false)
    deps.requestDownload(
      objectUrlForBytes(msg.zip, 'application/zip'),
      zipFilenameFromActiveFile(
        deps.activeFileRef.current,
        zipLabels(deps, 'WAV parts'),
      ),
      true,
    )
    return true
  }

  if (msg.type === 'splitWavErr') {
    if (msg.id !== deps.latestSplitWavIdRef.current) return true
    deps.setSplitWavExporting(false)
    deps.setDiagnostics(msg.diagnostics)
    return true
  }

  if (msg.type === 'splitMp3') {
    if (msg.id !== deps.latestSplitMp3IdRef.current) return true
    deps.setSplitMp3Exporting(false)
    deps.requestDownload(
      objectUrlForBytes(msg.zip, 'application/zip'),
      zipFilenameFromActiveFile(
        deps.activeFileRef.current,
        zipLabels(deps, 'MP3 parts'),
      ),
      true,
    )
    return true
  }

  if (msg.type === 'splitMp3Err') {
    if (msg.id !== deps.latestSplitMp3IdRef.current) return true
    deps.setSplitMp3Exporting(false)
    deps.setDiagnostics(msg.diagnostics)
    return true
  }

  return false
}
