import { tagSelector } from '../dataAttributes'
import type { ScoreOrder } from '../poentaroWasm'
import type { MeasureSelection } from '../types'
import type { StaffScore } from '../worker/messages'
import { staffMeasureElement } from './previewStaffMeasures'

/** The element the preview scrolls into view for a measure selection.
 * A `sequence` chain selection's `highlightRanges` can paint several
 * disjoint highlight rects (e.g. "Intro" and "C"), and the first one in DOM
 * order is wherever the chain's earliest entry sits in the document — not
 * necessarily `revealMeasureIndex`. So only the caret-only highlight (a
 * single contiguous range) is targeted by its rect; a chain selection
 * always scrolls to its reveal measure. In the staff view (`staffScore`
 * set) it's always the reveal measure's Verovio group. */
export function findRevealTarget(
  container: HTMLElement,
  selection: MeasureSelection,
  staffScore: StaffScore | null,
  scoreOrder: ScoreOrder,
): Element | null {
  const sequenceEntryIndex =
    scoreOrder === 'expanded'
      ? selection.sequenceSelection?.revealEntryIndex
      : undefined
  if (staffScore) {
    return (
      staffMeasureElement(
        container,
        staffScore.measures,
        selection.revealMeasureIndex,
        sequenceEntryIndex,
      ) ?? null
    )
  }
  if (sequenceEntryIndex !== undefined) {
    return container.querySelector(
      tagSelector('measure', {
        index: selection.revealMeasureIndex,
        sequenceEntryIndex,
      }),
    )
  }
  const highlightRect =
    selection.highlightRanges === undefined
      ? container.querySelector('[data-testid="measure-highlight"]')
      : null
  return (
    highlightRect ??
    container.querySelector(
      tagSelector('measure', { index: selection.revealMeasureIndex }),
    )
  )
}
