import { tagFromElement, tagSelector } from '../dataAttributes'
import type { ClickableElementId } from '../poentaroWasm'

/**
 * The `ClickableElementId` for an already-resolved DOM element — the
 * delegated-event (`mouseover`/`mouseout` `closest()`) counterpart of the
 * point-based `getNoteAtPoint`/`getLyricAtPoint`/`getMeasureAtPoint`/
 * `getPartLabelAtPoint`/`getLyricLabelAtPoint` in `previewSelection.ts`/
 * `previewLabelSelection.ts`, which funnel their own "element → fields"
 * parsing through this same function rather than duplicating it. Reads off
 * `el`'s own `Tag` (via `tagFromElement`, `../dataAttributes.ts`), so `el`
 * must already be the `[data-tag="..."]` group itself (the caller's
 * `closest()`/`elementFromPoint` walk has already found it) — not just some
 * descendant of it.
 */
export function clickableElementIdFromElement(
  el: Element,
): ClickableElementId | undefined {
  const tag = tagFromElement(el)
  if (!tag) return undefined
  switch (tag.tag) {
    case 'note':
    case 'lyric':
    case 'part-label':
    case 'lyric-label':
      return tag
    case 'measure':
    case 'bar-number':
      return {
        tag: 'measure',
        val: { measureIndexStart: tag.val.index, measureIndexEnd: tag.val.end },
      }
    case 'bar-line': {
      // A bar line visually introduces the measure *after* it, so `next`
      // wins when present; a system's *last* bar line (its closing line,
      // with no following measure on the same row) has no `next` and falls
      // back to `prev` instead. `next`/`prev` each name a single
      // source-measure index, not a range, so this looks up the actual
      // `[data-tag="measure"]` element that index belongs to (matching on
      // `data-measure-index` for `next`, since that's a block's own leading
      // index, or `data-measure-index-end` for `prev`, its trailing index)
      // to recover the merged-multi-measure-rest range, if any, that
      // measure is actually part of. Ported unchanged from
      // `getBarLineMeasureAtPoint` (`previewSelection.ts`) — the one
      // `document.querySelector` this function needs, since a bar line's
      // own `Tag` only carries a neighboring measure *index*, not that
      // measure's own possibly-wider range.
      const { measureIndexNext, measureIndexPrev } = tag.val
      const measureEl =
        measureIndexNext !== undefined
          ? document.querySelector<HTMLElement>(
              tagSelector('measure', { index: measureIndexNext }),
            )
          : measureIndexPrev !== undefined
            ? document.querySelector<HTMLElement>(
                tagSelector('measure', { end: measureIndexPrev }),
              )
            : null
      const measureTag = measureEl && tagFromElement(measureEl)
      if (
        !measureTag ||
        (measureTag.tag !== 'measure' && measureTag.tag !== 'bar-number')
      )
        return undefined
      return {
        tag: 'measure',
        val: {
          measureIndexStart: measureTag.val.index,
          measureIndexEnd: measureTag.val.end,
        },
      }
    }
    case 'section-label':
    case 'guitar-diagram':
      return undefined
    default: {
      const exhaustiveCheck: never = tag
      throw new Error(
        `Unhandled Tag variant: ${JSON.stringify(exhaustiveCheck)}`,
      )
    }
  }
}
