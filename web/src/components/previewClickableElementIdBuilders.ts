import type { ClickableElementId } from '../poentaroWasm'
import type { LyricLabelHit, PartLabelHit } from './previewLabelSelection'
import {
  getLyricLabelAtPoint,
  getPartLabelAtPoint,
} from './previewLabelSelection'
import type { LyricCell, MeasureRange, NoteCell } from './previewSelection'
import { getLyricAtPoint, getNoteAtPoint } from './previewSelection'

/**
 * Builds the `ClickableElementId` (`'measure'` variant) `resolve_selection_range`
 * expects from a `MeasureRange`. Exported alongside its four siblings below
 * so `previewClickHandler.ts` can build a `PreviewAnchorState`'s `anchorId` at
 * anchor time, not just here at resolve time.
 */
export function measureClickableElementId(
  range: MeasureRange,
): Extract<ClickableElementId, { tag: 'measure' }> {
  return {
    tag: 'measure',
    val: { measureIndexStart: range.start, measureIndexEnd: range.end },
  }
}

/** The 'note'-mode analog of `measureClickableElementId` — builds the
 * `ClickableElementId` (`'note'` variant) from a `NoteCell`. */
export function noteClickableElementId(
  cell: NoteCell,
): Extract<ClickableElementId, { tag: 'note' }> {
  return {
    tag: 'note',
    val: { sourcePartIndex: cell.sourcePartIndex, noteId: cell.noteId },
  }
}

/** The 'lyric'-mode analog of `noteClickableElementId` — builds the
 * `ClickableElementId` (`'lyric'` variant) from a `LyricCell`. */
export function lyricClickableElementId(
  cell: LyricCell,
): Extract<ClickableElementId, { tag: 'lyric' }> {
  return {
    tag: 'lyric',
    val: {
      sourcePartIndex: cell.sourcePartIndex,
      noteId: cell.noteId,
    },
  }
}

/** The 'part-label'-mode analog of `noteClickableElementId` — builds the
 * `ClickableElementId` (`'part-label'` variant) from a `PartLabelHit`. */
export function partLabelClickableElementId(
  hit: PartLabelHit,
): Extract<ClickableElementId, { tag: 'part-label' }> {
  return { tag: 'part-label', val: hit }
}

/** The 'lyric-label'-mode analog of `partLabelClickableElementId` — builds
 * the `ClickableElementId` (`'lyric-label'` variant) from a `LyricLabelHit`. */
export function lyricLabelClickableElementId(
  hit: LyricLabelHit,
): Extract<ClickableElementId, { tag: 'lyric-label' }> {
  return { tag: 'lyric-label', val: hit }
}

/**
 * The `ClickableElementId` for whatever clickable target sits at `(x, y)`,
 * trying every type `resolve_selection_range` can resolve a range against —
 * `Note`, `Lyric`, `PartLabel`, then `LyricLabel` — and returning `undefined`
 * only when the point misses every one of them (a bar-line/gutter/
 * empty-space point). Every one of `'note'`/`'lyric'`/`'part-label'`/
 * `'lyric-label'` mode's `current` resolution goes through this now, not
 * just a same-type hit-test: with every `(anchor-type, current-type)`
 * combination among these four types ID-resolved (see
 * `resolve_selection_range_response`'s label-mixed arms in
 * `selection_range.rs`, added per
 * `PLAN-clickable-element-id-selection.md`'s Status section), a second click
 * landing on *any* of them — not just the anchor's own type — reaches wasm.
 * The order
 * mirrors `handleAnchorClick`'s own hit-test priority (label checks before
 * the plain note/lyric checks), except `Note`/`Lyric` are tried first here
 * since they're both mode's own most common `current` target and cheaper to
 * rule out than walking the label checks first.
 */
export function anyClickableElementIdAtPoint(
  x: number,
  y: number,
): ClickableElementId | undefined {
  const noteCell = getNoteAtPoint(x, y)
  if (noteCell) return noteClickableElementId(noteCell)
  const lyricCell = getLyricAtPoint(x, y)
  if (lyricCell) return lyricClickableElementId(lyricCell)
  const partLabelHit = getPartLabelAtPoint(x, y)
  if (partLabelHit) return partLabelClickableElementId(partLabelHit)
  const lyricLabelHit = getLyricLabelAtPoint(x, y)
  if (lyricLabelHit) return lyricLabelClickableElementId(lyricLabelHit)
  return undefined
}
