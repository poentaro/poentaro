import { tagFromElement, tagSelector } from '../dataAttributes'
import type { LyricCellIn, NoteCellIn, Tag } from '../poentaroWasm'
import { clickableElementIdFromElement } from './clickableElementId'

/** One rendered note/rest, keyed the same way as the `note` `Tag`'s
 * `data-part-index`/`data-note-id` SVG attributes. */
export type NoteCell = NoteCellIn

/**
 * Generic hit-test behind `getNoteAtPoint`/`getLyricAtPoint`/
 * `getPartLabelAtPoint`/`getLyricLabelAtPoint`: reads the element under
 * `(x, y)`, walks up to its nearest `tagType` group, and
 * resolves a `ClickableElementId` off it via `clickableElementIdFromElement`
 * — the point-based counterpart of that function's own delegated-event use
 * (`mouseover`'s `event.target.closest(...)` in `usePreviewClickSelection.ts`).
 */
export function getClickableElementIdAtPoint(
  x: number,
  y: number,
  tagType: Tag['tag'],
) {
  const el = document.elementFromPoint(x, y)
  const group = el?.closest(tagSelector(tagType))
  if (!group) return undefined
  return clickableElementIdFromElement(group)
}

export function getSectionLabelAtPoint(
  x: number,
  y: number,
): string | undefined {
  const el = document.elementFromPoint(x, y)
  if (!el) return undefined
  const group = el.closest(tagSelector('section-label'))
  if (!group) return undefined
  const tag = tagFromElement(group)
  return tag?.tag === 'section-label' ? tag.val.label : undefined
}

export interface MeasureRange {
  start: number
  end: number
}

/** Adapts a `ClickableElementId`'s `'measure'` variant to this module's own
 * `MeasureRange` shape, or `undefined` if `id` resolved to some other kind —
 * shared by every measure-flavored hit-test below. */
function measureRangeFromId(
  id: ReturnType<typeof clickableElementIdFromElement>,
): MeasureRange | undefined {
  if (id?.tag !== 'measure') return undefined
  return { start: id.val.measureIndexStart, end: id.val.measureIndexEnd }
}

/**
 * Resolves a click point that's over a bar line's own click target
 * (`[data-tag="bar-line"]`, see `Tag::BarLine`/
 * `AbsoluteContent::BarLineClickTarget`) to a measure range, purely from the
 * server-computed `data-measure-index-next`/`data-measure-index-prev`
 * identity carried on that group (see `clickableElementIdFromElement`'s
 * `'bar-line'` case) — no pixel geometry involved beyond the initial
 * `elementFromPoint`.
 *
 * Returns `undefined` when `x`/`y` isn't over a bar-line click target at
 * all, so callers can fall through to the generic point-based lookup.
 *
 * Exported so `previewClickHandler.ts` can check for a bar-line hit
 * *before* its Cmd/Ctrl gate: grabbing the divider itself is an unambiguous
 * request to select measures, unlike a click on a note/lyric/gutter pixel
 * (which is ambiguous enough to need the modifier) — see that file's
 * unconditional bar-line check.
 */
export function getBarLineMeasureAtPoint(
  x: number,
  y: number,
): MeasureRange | undefined {
  return measureRangeFromId(getClickableElementIdAtPoint(x, y, 'bar-line'))
}

/**
 * Resolves a click point that's over a measure's own bar number (drawn
 * in the directive row above the musical rows — `[data-tag="bar-number"]`,
 * see `BarNumberClickTarget`/`Tag::BarNumber`) to that measure's range.
 *
 * A bar number sits outside every note's own click target. Kept as its own
 * unconditional (no Cmd/Ctrl needed) check in `previewClickHandler.ts`,
 * mirroring `getBarLineMeasureAtPoint`'s bar-line-handle check: landing on
 * the bar number itself is an unambiguous request to select that measure,
 * checked ahead of the Cmd/Ctrl gate rather than left to fall through to the
 * gutter-miss measure fallback further down.
 */
export function getBarNumberMeasureAtPoint(
  x: number,
  y: number,
): MeasureRange | undefined {
  return measureRangeFromId(getClickableElementIdAtPoint(x, y, 'bar-number'))
}

/**
 * Elements a click/hover point can resolve a measure range from:
 * `[data-tag="measure"]` (a measure's own musical-row body) and
 * `[data-tag="bar-number"]` (that same measure's own bar number, drawn in
 * the directive row above — see `BarNumberClickTarget`/`Tag::BarNumber`,
 * kept as a separate, smaller tag rather than folded into `[data-tag="measure"]`
 * so several e2e tests' `[data-tag="measure"]` DOM-order/count assumptions
 * stay unaffected).
 */
const MEASURE_RANGE_SELECTOR = [
  tagSelector('measure'),
  tagSelector('bar-number'),
].join(', ')

/**
 * The inclusive measure range of every rendered system, read off the part
 * labels: every part label in a system carries that system's
 * `measureIndexStart`/`measureIndexEnd` (one label per part per system).
 * Feeds `resolveSystemSelectionRange`.
 */
export function renderedSystems(container: HTMLElement): MeasureRange[] {
  const systems = new Map<string, MeasureRange>()
  for (const label of Array.from(
    container.querySelectorAll(tagSelector('part-label')),
  )) {
    const tag = tagFromElement(label)
    if (tag?.tag !== 'part-label') continue
    const { measureIndexStart: start, measureIndexEnd: end } = tag.val
    systems.set(`${start}:${end}`, { start, end })
  }
  return [...systems.values()]
}

export function getMeasureAtPoint(
  x: number,
  y: number,
): MeasureRange | undefined {
  const barLineRange = getBarLineMeasureAtPoint(x, y)
  if (barLineRange) return barLineRange

  // A measure's click-target rect is a sibling of the note/lyric click
  // targets drawn over the same region (see `render_measure_click_target`/
  // `render_note_click_target`), not their ancestor — so a point over a
  // note or lyric returns *that* element from `elementFromPoint`, and
  // `closest` never reaches the measure group underneath it. Scanning the
  // full `elementsFromPoint` stack instead finds the measure regardless of
  // what's painted on top of it at this point.
  for (const el of document.elementsFromPoint(x, y)) {
    const group = el.closest<HTMLElement>(MEASURE_RANGE_SELECTOR)
    if (group) {
      const range = measureRangeFromId(clickableElementIdFromElement(group))
      if (range) return range
    }
  }
  return undefined
}

/** The note/rest under the given point, if any — reads the invisible
 * `NoteClickTarget` rect's enclosing `Tag::Note` group (see
 * `renderer::new_renderer::render_note_click_target`), which sits on top of
 * the `pointer-events: none` playback cursor rect for the same note. */
export function getNoteAtPoint(x: number, y: number): NoteCell | undefined {
  const id = getClickableElementIdAtPoint(x, y, 'note')
  return id?.tag === 'note' ? id.val : undefined
}

/** One rendered lyric syllable, keyed the same way as `Tag::Lyric`'s
 * `data-part-index`/`data-note-id`/`data-verse` SVG attributes. Structurally
 * identical to `NoteCell` but kept as its own type — a lyric cell and a note
 * cell sharing the same underlying note number are not interchangeable,
 * they're just keyed by the same note for convenience (see
 * `lyric_spans::LyricCell`). */
export type LyricCell = LyricCellIn

/** The lyric syllable under the given point, if any — reads the invisible
 * `LyricClickTarget` rect's enclosing `Tag::Lyric` group (see
 * `renderer::new_renderer::render_lyric_click_target`), which paints on top
 * of the wider `NoteClickTarget` rect that geometrically covers the same
 * lyric row, so a click that lands on the syllable's own rect always
 * resolves here rather than to `getNoteAtPoint`. */
export function getLyricAtPoint(x: number, y: number): LyricCell | undefined {
  const id = getClickableElementIdAtPoint(x, y, 'lyric')
  return id?.tag === 'lyric' ? id.val : undefined
}
