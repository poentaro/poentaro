import {
  DATA_RANGE_ACTIVE_FLAG,
  rectVariantSelector,
  tagSelector,
} from '../dataAttributes'
import { type ClickableElementId, poentaroWasm } from '../poentaroWasm'
import type { LyricSpan, NoteSpan } from '../types'
import { clickableElementIdFromElement } from './clickableElementId'
import type { LyricCell, NoteCell } from './previewSelection'

/** Marks every `kind` label click-target rect whose notes (part label) or
 * syllables (lyric label) are all selected with `flag`, clearing it from the
 * rest. Wasm decides which labels count (`fully_selected_labels`). */
function applyLabelHighlights(
  container: HTMLElement,
  kind: 'part-label' | 'lyric-label',
  flag: string,
  selected: {
    noteSpans: NoteSpan[]
    lyricSpans: LyricSpan[]
    noteCells: NoteCell[]
    lyricCells: LyricCell[]
  },
): void {
  const labels = Array.from(
    container.querySelectorAll<SVGRectElement>(
      rectVariantSelector(`${kind}-click-target`),
    ),
  ).flatMap((rect) => {
    const group = rect.closest(tagSelector(kind))
    const id = group && clickableElementIdFromElement(group)
    return id ? [{ rect, id }] : []
  })
  if (labels.length === 0) return
  const fullySelected = poentaroWasm().fullySelectedLabels(
    selected.noteSpans,
    selected.lyricSpans,
    labels.map(({ id }): ClickableElementId => id),
    selected.noteCells,
    selected.lyricCells,
  )
  labels.forEach(({ rect }, index) => {
    if (fullySelected[index]) {
      rect.setAttribute(flag, '')
    } else {
      rect.removeAttribute(flag)
    }
  })
}

/** Re-applies the lyric-label fill from the given selected lyric cells: a
 * lyric label reads as selected once every syllable it covers (its lyric
 * part across its whole system) is selected. The lyric-side mirror of
 * `applyPersistedPartLabelHighlights`. */
export function applyPersistedLyricLabelHighlights(
  container: HTMLElement,
  lyricSpans: LyricSpan[],
  selectedLyricCells: LyricCell[],
): void {
  applyLabelHighlights(
    container,
    'lyric-label',
    DATA_RANGE_ACTIVE_FLAG.lyricLabel,
    {
      noteSpans: [],
      lyricSpans,
      noteCells: [],
      lyricCells: selectedLyricCells,
    },
  )
}

/** Re-applies the part-label fill from the given selected note cells: a
 * part label reads as selected once every note/rest it covers (its part
 * across its whole system) is selected, so the fill persists after the
 * second click (and survives an SVG DOM swap, same as
 * `applyPersistedNoteHighlights`) and also reflects a selection made some
 * other way (e.g. typing in the editor). */
export function applyPersistedPartLabelHighlights(
  container: HTMLElement,
  noteSpans: NoteSpan[],
  selectedNoteCells: NoteCell[],
): void {
  applyLabelHighlights(
    container,
    'part-label',
    DATA_RANGE_ACTIVE_FLAG.partLabel,
    {
      noteSpans,
      lyricSpans: [],
      noteCells: selectedNoteCells,
      lyricCells: [],
    },
  )
}
