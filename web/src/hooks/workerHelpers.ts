import { findIndex, findLastIndex } from 'remeda'
import { stripFileExtension } from '../fileExtension'
import { poentaroWasm } from '../poentaroWasm'
import type {
  MeasureSpan,
  PartInfo,
  PartToggleState,
  ResolvedPartVisibility,
} from '../types'

export function measureRangeInSpan(
  spans: MeasureSpan[],
  startLine: number,
  endLine: number,
): { start: number; end: number } | null {
  const overlaps = (span: MeasureSpan) =>
    span.startLine <= endLine && span.endLine >= startLine
  const start = findIndex(spans, overlaps)
  const end = findLastIndex(spans, overlaps)
  return start === -1 ? null : { start, end }
}

/**
 * Like `measureRangeInSpan`, but also resolves `revealLine` to its own
 * measure index and attaches it as `revealMeasureIndex` — the measure the
 * preview should scroll to for this selection, when it differs from
 * `start` (e.g. a `sequence` chain selection whose envelope start, in
 * document order, isn't the entry the user actually navigated to). Falls
 * back to `start` when `revealLine` doesn't resolve to a measure of its
 * own.
 *
 * `measureRanges`, when given, is folded in as `highlightRanges` — the
 * exact disjoint measure ranges to highlight in the SVG preview for a `#
 * sequence` chain selection, bypassing `measureRangeInSpan`'s single-span
 * derivation entirely (it can't represent "C and A but not B in between").
 */
export function measureRangeInSpanWithReveal(
  spans: MeasureSpan[],
  startLine: number,
  endLine: number,
  revealLine: number,
  measureRanges?: { start: number; end: number }[],
): {
  start: number
  end: number
  revealMeasureIndex: number
  highlightRanges?: { start: number; end: number }[]
} | null {
  const range = measureRangeInSpan(spans, startLine, endLine)
  if (!range) return null
  const revealMeasureIndex =
    measureRangeInSpan(spans, revealLine, revealLine)?.start ?? range.start
  return {
    ...range,
    revealMeasureIndex,
    ...(measureRanges ? { highlightRanges: measureRanges } : {}),
  }
}

/** What is drawn and what sounds when nothing is hidden or soloed, and what
 * is used while the score's parts are not known yet. */
export const ALL_PARTS_VISIBLE: ResolvedPartVisibility = {
  hiddenParts: [],
  soloedParts: [],
}

/** Resolves the toggle UI's raw hide/solo state into what is drawn and what
 * sounds. The rules live in Rust (`resolve_part_visibility`); this only
 * hands the state over, with the preset chosen in the preset menu. */
export function resolvePartVisibility(
  source: string,
  parts: PartInfo[],
  toggles: PartToggleState,
  presetName: string | null,
): ResolvedPartVisibility {
  if (parts.length === 0) return ALL_PARTS_VISIBLE
  return poentaroWasm().resolvePartVisibility(
    source,
    parts,
    toggles,
    presetName ?? undefined,
  )
}

/** True when the toggles leave no part drawn at all. */
export function noPartsSelected(
  parts: PartInfo[],
  visibility: ResolvedPartVisibility,
): boolean {
  return parts.length > 0 && visibility.renderedTracks?.length === 0
}

/** Wraps export bytes in an object URL of the given MIME type — the URL is
 * the caller's to revoke once it's no longer needed (see `requestDownload`/
 * `cancelPendingDownload`/`confirmPendingDownload` in
 * `usePoentaroWorkerState.ts`). */
export function objectUrlForBytes(bytes: ArrayBuffer, mimeType: string) {
  return URL.createObjectURL(new Blob([bytes], { type: mimeType }))
}

/** Fires a browser download for `url` under `filename` via a throwaway
 * anchor click — the one primitive every export format's download funnels
 * through. Does not revoke `url`; the caller owns its lifetime. */
export function triggerAnchorDownload(url: string, filename: string) {
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = filename
  anchor.click()
}

// Mark the preset or the enabled parts in the filename so exports of
// different presets or toggle combinations don't silently overwrite each
// other.
export function withLabelsSuffix(
  filename: string,
  labels: string[] | undefined,
): string {
  if (!labels || labels.length === 0) return filename
  const suffix = labels.join(', ')
  const dotIndex = filename.lastIndexOf('.')
  if (dotIndex === -1) return `${filename} (${suffix})`
  return `${filename.slice(0, dotIndex)} (${suffix})${filename.slice(dotIndex)}`
}

export function pdfFilenameFromActiveFile(
  activeFile: string,
  labels?: string[],
): string {
  const base = `${baseNameFromActiveFile(activeFile)}.pdf`
  return withLabelsSuffix(base, labels)
}

export function midiFilenameFromActiveFile(
  activeFile: string,
  labels?: string[],
): string {
  const base = `${baseNameFromActiveFile(activeFile)}.mid`
  return withLabelsSuffix(base, labels)
}

export function wavFilenameFromActiveFile(
  activeFile: string,
  labels?: string[],
): string {
  const base = `${baseNameFromActiveFile(activeFile)}.wav`
  return withLabelsSuffix(base, labels)
}

export function mp3FilenameFromActiveFile(
  activeFile: string,
  labels?: string[],
): string {
  const base = `${baseNameFromActiveFile(activeFile)}.mp3`
  return withLabelsSuffix(base, labels)
}

export function zipFilenameFromActiveFile(
  activeFile: string,
  labels: string[],
): string {
  const base = `${baseNameFromActiveFile(activeFile)}.zip`
  return withLabelsSuffix(base, labels)
}

/** Posts `payload` to `worker` only after the browser has had a chance to
 * paint — use this right after flipping an "…Exporting" flag to `true` so
 * the resulting loading toast is guaranteed at least one visible frame
 * before the worker's response can flip it back to `false`. Without this,
 * a fast enough response (e.g. encoding a tiny fixture, or a paint delayed
 * by CPU contention from other work) can let both state flips land in the
 * same frame, so the toast never actually renders. `requestAnimationFrame`
 * fires right before the next paint, so by the time it runs, any commit
 * from the earlier `setState` call has already had its chance to paint. */
export function postAfterPaint(worker: Worker, payload: unknown) {
  requestAnimationFrame(() => worker.postMessage(payload))
}

/** The last segment of a `/`-separated file path, i.e. the file name
 * without the folders it lives in. */
export function fileNameWithoutFolder(path: string): string {
  return path.slice(path.lastIndexOf('/') + 1)
}

/** The name every export of `activeFile` is downloaded under: its file name
 * without its folders or its extension. */
export function baseNameFromActiveFile(activeFile: string): string {
  return stripFileExtension(fileNameWithoutFolder(activeFile))
}
