import type { LyricLabelElementId, PartLabelElementId } from '../poentaroWasm'
import { getClickableElementIdAtPoint } from './previewSelection'

/** One rendered part-label click target, keyed the same way as
 * `Tag` `part-label`'s `data-part-index`/`data-measure-index-start`/
 * `data-measure-index-end` SVG attributes — see `getPartLabelAtPoint`. */
export type PartLabelHit = PartLabelElementId

/** The part-label click target under the given point, if any — reads the
 * invisible `PartLabelClickTarget` rect's enclosing `Tag` `part-label` group
 * (see `renderer::new_renderer::render_part_label_click_target`). */
export function getPartLabelAtPoint(
  x: number,
  y: number,
): PartLabelHit | undefined {
  const id = getClickableElementIdAtPoint(x, y, 'part-label')
  return id?.tag === 'part-label' ? id.val : undefined
}

/** One rendered lyric-label click target, keyed the same way as
 * `Tag::LyricLabel`'s `data-part-index`/
 * `data-measure-index-start`/`data-measure-index-end` SVG attributes — see
 * `getLyricLabelAtPoint`. The lyric-side mirror of `PartLabelHit`. */
export type LyricLabelHit = LyricLabelElementId

/** The lyric-label click target under the given point, if any — reads the
 * invisible `LyricLabelClickTarget` rect's enclosing `Tag::LyricLabel` group
 * (see `renderer::new_renderer::render_lyric_label_click_target`). The
 * lyric-side mirror of `getPartLabelAtPoint`. */
export function getLyricLabelAtPoint(
  x: number,
  y: number,
): LyricLabelHit | undefined {
  const id = getClickableElementIdAtPoint(x, y, 'lyric-label')
  return id?.tag === 'lyric-label' ? id.val : undefined
}
