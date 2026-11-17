import {
  rectVariantSelector,
  tagFromElement,
  tagSelector,
} from '../dataAttributes'
import type { Tag } from '../poentaroWasm'
import type { LyricCell, NoteCell } from './previewSelection'

/**
 * Shape driving `applyPersistedHighlights` — the generic core behind
 * `applyPersistedNoteHighlights` and its lyric counterpart. Deliberately
 * *not* used for part-label highlighting: that uses a different marking
 * mechanism (`setAttribute`/`removeAttribute` on the rect itself, rather
 * than a dataset boolean on its enclosing group) plus a system-scoping
 * filter (`anchorSystem`) with no note/lyric analog.
 */
interface RangeHighlightSpec<Cell> {
  /** CSS selector for each candidate hit-target rect, e.g.
   * `rect[data-variant="note-click-target"]`. */
  rectSelector: string
  /** `Tag['tag']` of the rect's enclosing group, e.g. `'note'` — the
   * selector itself is built from this via `tagSelector`. */
  tagType: Tag['tag']
  /** Narrows the enclosing group's own `Tag` (already known to be
   * `tagType`, via `tagFromElement`) into a `Cell`, or `undefined` if a
   * required field is missing. */
  parseCell: (tag: Tag) => Cell | undefined
  /** A string uniquely identifying `cell`, used to test set membership. */
  cellKey: (cell: Cell) => string
  /** `dataset` property (camelCase) toggled on the enclosing group to mark it
   * as range-selected, e.g. `'noteRangeSelected'`. */
  datasetFlag: string
}

/** Marks every hit-target rect's enclosing group whose cell is in `cells`
 * with `spec.datasetFlag`, clearing it from every other one. Used to keep
 * the selection visible after it resolves and to restore it whenever the SVG DOM is
 * swapped out from under it. Cheap and idempotent, so it's safe to re-run on
 * every relevant render. */
function applyPersistedHighlights<Cell>(
  container: HTMLElement,
  cells: Cell[],
  spec: RangeHighlightSpec<Cell>,
): void {
  const selectedKeys = new Set(cells.map(spec.cellKey))
  // Clear every group first, not only those holding a click target: a
  // patched SVG can leave the flag on a sibling group (e.g. a note's
  // playback-cursor group) that held a click target before.
  for (const group of Array.from(
    container.querySelectorAll<HTMLElement>(tagSelector(spec.tagType)),
  )) {
    delete group.dataset[spec.datasetFlag]
  }
  for (const rect of Array.from(
    container.querySelectorAll<SVGRectElement>(spec.rectSelector),
  )) {
    const group = rect.closest(tagSelector(spec.tagType)) as HTMLElement | null
    if (!group) continue
    const tag = tagFromElement(group)
    const cell = tag && spec.parseCell(tag)
    const selected = cell !== undefined && selectedKeys.has(spec.cellKey(cell))
    if (selected) group.dataset[spec.datasetFlag] = ''
  }
}

const noteRangeSpec: RangeHighlightSpec<NoteCell> = {
  rectSelector: rectVariantSelector('note-click-target'),
  tagType: 'note',
  parseCell: (tag) => {
    if (tag.tag !== 'note') return undefined
    return { sourcePartIndex: tag.val.sourcePartIndex, noteId: tag.val.noteId }
  },
  cellKey: (c) => `${c.sourcePartIndex}:${c.noteId}`,
  datasetFlag: 'noteRangeSelected',
}

const lyricRangeSpec: RangeHighlightSpec<LyricCell> = {
  rectSelector: rectVariantSelector('lyric-click-target'),
  tagType: 'lyric',
  parseCell: (tag) => {
    if (tag.tag !== 'lyric') return undefined
    return {
      sourcePartIndex: tag.val.sourcePartIndex,
      noteId: tag.val.noteId,
    }
  },
  cellKey: (c) => `${c.sourcePartIndex}:${c.noteId}`,
  datasetFlag: 'lyricRangeSelected',
}

/** Re-applies the note-range highlight from a fixed cell list — used to keep the selection visible after mouseup (and
 * to restore it whenever the SVG DOM is swapped out from under it, e.g. by a
 * highlighted re-render triggered by the Monaco selection the range-select
 * pushed).
 * Cheap and idempotent, so it's safe to re-run on every relevant render.
 * Scope is the click-target rect's enclosing group — a note also has a
 * sibling `Tag::Note` group for its (pointer-events: none) playback-cursor
 * rect, and marking that one too would be harmless (the CSS rule only paints
 * the click-target rect) but redundant. */
export function applyPersistedNoteHighlights(
  container: HTMLElement,
  cells: NoteCell[],
): void {
  applyPersistedHighlights(container, cells, noteRangeSpec)
}

/** Re-applies the lyric-range highlight from a fixed cell list — mirrors `applyPersistedNoteHighlights`, used to keep
 * the selection visible after mouseup and to restore it whenever the SVG DOM
 * is swapped out from under it. */
export function applyPersistedLyricHighlights(
  container: HTMLElement,
  cells: LyricCell[],
): void {
  applyPersistedHighlights(container, cells, lyricRangeSpec)
}
