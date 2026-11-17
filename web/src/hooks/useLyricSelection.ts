import type { RefObject } from 'react'
import type { LyricCell } from '../components/previewSelection'
import { type LyricSelectionRun, poentaroWasm } from '../poentaroWasm'
import type { EditorHandle, LyricSpan } from '../types'
import { ensureWasmInit } from '../wasmInit'
import {
  type ByteRange,
  useByteRangeSelectionCore,
} from './useByteRangeSelectionCore'

/** One contiguous range-selected byte range within a single verse line of a
 * single part's single measure, as grouped by the wasm export
 * `groupLyricSelection` (`lyric_spans::group_selected_lyrics_into_contiguous_runs`
 * in Rust). */
export type { LyricSelectionRun }

/** Calls the wasm `groupLyricSelection` export directly on the main
 * thread (bypassing the debounced render worker) — this is pure grouping
 * over an already-fetched flat `lyric_spans` array, so it doesn't need to
 * re-parse `source` and stays responsive on every selection-change tick. */
/** Exported for `useAppController`'s `handleMeasureRangeSelect`, which groups
 * a measure click's lyric cells itself so it can push them into Monaco in
 * the same combined selection as the measure's note cells (see
 * `useByteRangeSelectionCore`'s `applySelectionSilently`). */
export async function groupSelectedLyricsIntoContiguousRuns(
  selectedCells: LyricCell[],
  lyricSpans: LyricSpan[],
): Promise<LyricSelectionRun[]> {
  await ensureWasmInit()
  const response = poentaroWasm().groupLyricSelection(lyricSpans, selectedCells)
  return response.tag === 'ok' ? response.val.runs : []
}

/** The lyric syllables whose source text `ranges` overlap, and their runs
 * (see `useByteRangeSelectionCore`). */
async function selectLyricsInByteRanges(
  ranges: ByteRange[],
  lyricSpans: LyricSpan[],
): Promise<{ cells: LyricCell[]; runs: LyricSelectionRun[] }> {
  await ensureWasmInit()
  return poentaroWasm().selectLyricsInByteRanges(
    lyricSpans,
    ranges.map((range) => ({ startByte: range.start, endByte: range.end })),
  )
}

/** Exported alongside `groupSelectedLyricsIntoContiguousRuns` for
 * `useAppController`'s `handleMeasureRangeSelect`. */
export function lyricRunByteRange(run: LyricSelectionRun) {
  return { start: run.startByte, end: run.endByte }
}

/**
 * Turns a click-and-click range-select over lyric syllables (a set of
 * `(source_part_index, note_id, verse)` cells hit-tested off the SVG, see
 * `Preview.tsx`) into a Monaco multicursor selection over the source text —
 * one disjoint range per `(part, verse, measure)` the selection touched.
 *
 * Deliberately independent of `useNoteSelection`: a lyric selection never
 * drives note highlighting and vice versa, so this hook keeps its own call to
 * `useByteRangeSelectionCore` (and thus its own state) rather than threading
 * lyric cells through `useNoteSelection`'s `selectedNoteCells`/`runs`.
 */
export function useLyricSelection(
  lyricSpans: LyricSpan[],
  editorRef: RefObject<EditorHandle | null>,
) {
  const {
    selectedCells: lastSelectedCells,
    handleRangeSelect: handleLyricRangeSelect,
    handleEditorSelectionChange,
    applySelectionSilently: applyLyricSelectionSilently,
    clearSelection: clearLyricSelection,
  } = useByteRangeSelectionCore<LyricCell, LyricSpan, LyricSelectionRun>(
    lyricSpans,
    editorRef,
    groupSelectedLyricsIntoContiguousRuns,
    selectLyricsInByteRanges,
    lyricRunByteRange,
  )

  return {
    handleLyricRangeSelect,
    handleEditorSelectionChange,
    selectedLyricCells: lastSelectedCells,
    applyLyricSelectionSilently,
    clearLyricSelection,
  }
}
