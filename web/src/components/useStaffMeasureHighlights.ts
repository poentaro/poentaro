import { type RefObject, useEffect } from 'react'
import type { StaffScore } from '../worker/messages'
import type { PreviewAnchorState } from './previewAnchorState'
import type { MeasureRange } from './previewSelection'
import {
  applyStaffMeasureHighlights,
  applyStaffMeasureHitTargets,
} from './previewStaffMeasures'

export interface SelectedMeasures extends MeasureRange {
  highlightRanges?: MeasureRange[]
}

/** The staff view's counterpart of the numbered measure highlight: paints the
 * editor's selected measures (a `sequence` chain's exact disjoint ranges
 * when it has them) over Verovio's pages, re-applied whenever those pages
 * are swapped in, along with each measure's hover target. The highlight is
 * skipped while a click-and-click gesture is anchored: its
 * hover preview owns the highlight until it commits or cancels (see
 * `hoverStaffMeasure`/`cancelAnchor`), and the anchoring click's own
 * debounced editor selection would otherwise paint over it. */
export function useStaffMeasureHighlights(
  previewPagesRef: RefObject<HTMLDivElement | null>,
  staffScore: StaffScore | null,
  selectedMeasureRange: SelectedMeasures | null,
  anchorStateRef: RefObject<PreviewAnchorState>,
): void {
  // Before the highlight effect below, so its rects size to the same box.
  useEffect(() => {
    const container = previewPagesRef.current
    if (container && staffScore) {
      applyStaffMeasureHitTargets(container, staffScore.measures)
    }
  }, [previewPagesRef, staffScore])
  useEffect(() => {
    const container = previewPagesRef.current
    if (!container || !staffScore || anchorStateRef.current) return
    const ranges = selectedMeasureRange
      ? (selectedMeasureRange.highlightRanges ?? [selectedMeasureRange])
      : []
    applyStaffMeasureHighlights(container, staffScore.measures, ranges)
  }, [previewPagesRef, staffScore, selectedMeasureRange, anchorStateRef])
}
