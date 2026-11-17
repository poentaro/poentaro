import type { RefObject } from 'react'
import { useMemo } from 'react'
import { poentaroWasm } from '../poentaroWasm'
import type { EditorHandle, NoteSpan } from '../types'
import type { NoteCell, NoteSelectionRun } from '../utils/noteSpanSelection'
import { distinctPartAbbreviations } from '../utils/partAbbreviations'
import { ensureWasmInit } from '../wasmInit'
import {
  type ByteRange,
  useByteRangeSelectionCore,
} from './useByteRangeSelectionCore'

/** Calls the wasm `groupNoteSelection` export directly on the main
 * thread (bypassing the debounced render worker) — this is pure grouping
 * over an already-fetched flat `note_spans` array, so it doesn't need to
 * re-parse `source` and stays responsive on every selection-change tick. */
/** Exported for `useAppController`'s `handleMeasureRangeSelect`, which groups
 * a measure click's note cells itself so it can push them into Monaco in the
 * same combined selection as the measure's lyric cells (see
 * `useByteRangeSelectionCore`'s `applySelectionSilently`). */
export async function groupSelectedNotesIntoContiguousRuns(
  selectedCells: NoteCell[],
  noteSpans: NoteSpan[],
): Promise<NoteSelectionRun[]> {
  await ensureWasmInit()
  const response = poentaroWasm().groupNoteSelection(noteSpans, selectedCells)
  return response.tag === 'ok' ? response.val.runs : []
}

/** The notes whose source text `ranges` overlap, and their runs — so a
 * selection can be re-derived from its text after parts are hidden or shown
 * (see `useByteRangeSelectionCore`). */
async function selectNotesInByteRanges(
  ranges: ByteRange[],
  noteSpans: NoteSpan[],
): Promise<{ cells: NoteCell[]; runs: NoteSelectionRun[] }> {
  await ensureWasmInit()
  return poentaroWasm().selectNotesInByteRanges(
    noteSpans,
    ranges.map((range) => ({ startByte: range.start, endByte: range.end })),
  )
}

/** Exported alongside `groupSelectedNotesIntoContiguousRuns` for
 * `useAppController`'s `handleMeasureRangeSelect`. */
export function noteRunByteRange(run: NoteSelectionRun) {
  return { start: run.startByte, end: run.endByte }
}

export interface SelectedNoteRangePlaybackInfo {
  minMeasureIndex: number
  maxMeasureIndex: number
  selectedPartNames: string[]
}

/**
 * Turns a MuseScore-style note range-select (a set of `(source_part_index,
 * note_id)` cells hit-tested off the SVG, see `Preview.tsx`) into a Monaco
 * multicursor selection over the source text — one disjoint range per
 * `(part, measure)` the selection touched — and derives the info a "play
 * selection" action needs (see `useMeasureAudioPlayback.playNoteSelection`).
 */
export function useNoteSelection(
  noteSpans: NoteSpan[],
  editorRef: RefObject<EditorHandle | null>,
) {
  // Synced/shared views never mount an Editor, so there's no Monaco
  // selection to round-trip through `handleEditorSelectionChange` — but a
  // plain note tap still has its own precise blue highlight (painted
  // directly on the SVG by `resolveNoteSelection`, independent of any
  // editor), so no fallback is needed here: `useByteRangeSelectionCore`'s
  // default no-mounted-editor behavior (just recording `selectedCells`/
  // `runs`) is exactly right. This used to report the tap as a caret-only
  // `notifySelection` covering the whole enclosing measure — appropriate
  // back when this fallback was the *only* visual feedback (see
  // `useSectionNavigation`'s `selectSectionRange`), but left over after
  // per-note highlighting made it redundant, it painted a spurious
  // whole-measure amber background under every single-note tap in Synced/
  // shared view (see the mobile bug report this comment accompanies).
  // `useMeasureRangeSelection`'s own no-mounted-editor branch still does
  // this deliberately for an actual measure/bar-line selection.

  const {
    selectedCells: lastSelectedCells,
    runs: lastRuns,
    handleRangeSelect: handleNoteRangeSelect,
    handleEditorSelectionChange,
    applySelectionSilently: applyNoteSelectionSilently,
    clearSelection: clearNoteSelection,
  } = useByteRangeSelectionCore<NoteCell, NoteSpan, NoteSelectionRun>(
    noteSpans,
    editorRef,
    groupSelectedNotesIntoContiguousRuns,
    selectNotesInByteRanges,
    noteRunByteRange,
  )

  const selectedNoteRangePlaybackInfo =
    useMemo<SelectedNoteRangePlaybackInfo | null>(() => {
      if (lastRuns.length === 0) return null
      const measureIndices = lastRuns.map((run) => run.measureIndex)
      return {
        minMeasureIndex: Math.min(...measureIndices),
        maxMeasureIndex: Math.max(...measureIndices),
        // Each run carries its part's abbreviation straight from Rust (see
        // `note_spans::NoteSelectionRun::part_abbreviation`).
        selectedPartNames: distinctPartAbbreviations(lastRuns),
      }
    }, [lastRuns])

  return {
    handleNoteRangeSelect,
    handleEditorSelectionChange,
    selectedNoteRangePlaybackInfo,
    selectedNoteCells: lastSelectedCells,
    // Exposed alongside `selectedNoteCells` for `handleEditSelection`
    // (see `useAppSelectionAndNavigation.ts`), which needs both to re-apply
    // the same selection silently after an octave shift.
    selectedNoteRuns: lastRuns,
    applyNoteSelectionSilently,
    clearNoteSelection,
  }
}
