import type { MouseEvent } from 'react'
import type { StaffMeasure } from '../poentaroWasm'
import type { PreviewAnchorState } from './previewAnchorState'
import {
  anchorAndCommit,
  cancelAnchor,
  commitAnchoredGesture,
} from './previewGestureSteps'
import {
  type HandlePreviewClickArgs,
  measureClickableElementId,
} from './previewSelectionResolver'
import {
  applyPendingStaffMeasures,
  applyStaffMeasureHighlights,
  staffMeasureRangeAtPoint,
} from './previewStaffMeasures'

/**
 * The staff view's `mousedown` dispatch (`handlePreviewClick` hands off here
 * when `staffMeasures` is set): the same click-and-click gesture, but only
 * at measure granularity — Verovio's SVG has none of the numbered preview's
 * note/lyric/label click targets. A first click on a measure anchors
 * 'measure' mode, a second click on a measure commits
 * the range between them, and a click outside every measure cancels an
 * anchored gesture. Both commits go through the editor's selection exactly
 * like the numbered view's, which is what drives the staff highlight
 * (`useStaffMeasureHighlights`) and measure playback.
 */
export function handleStaffPreviewClick(
  e: MouseEvent<HTMLDivElement>,
  args: HandlePreviewClickArgs,
  measures: StaffMeasure[],
): void {
  const { anchorStateRef, previewPagesRef } = args
  const container = previewPagesRef.current
  const range =
    container &&
    staffMeasureRangeAtPoint(container, measures, e.clientX, e.clientY)
  const anchorState = anchorStateRef.current
  if (!range) {
    if (anchorState !== null) cancelAnchor(anchorStateRef, anchorState, args)
    return
  }
  e.preventDefault()
  const id = measureClickableElementId(range)
  if (anchorState?.mode === 'measure') {
    commitAnchoredGesture(anchorState, { x: e.clientX, y: e.clientY }, id, args)
    return
  }
  const newState: NonNullable<PreviewAnchorState> = {
    mode: 'measure',
    anchor: range,
    current: range,
    anchorId: id,
  }
  anchorAndCommit(anchorStateRef, newState, args)
  // The editor-driven highlight yields to the anchored gesture (see
  // `useStaffMeasureHighlights`), so paint the anchor here.
  if (container) applyStaffMeasureHighlights(container, measures, [range])
}

/** The staff view's hover preview between the two clicks: paints the span
 * from the anchored measure to the one under the pointer (in the pending
 * color, via the container's `data-pending-selection`). Off every measure,
 * the last hovered one is kept, like the numbered view's measure mode, and
 * nothing is repainted until the hovered measure changes. */
export function hoverStaffMeasure(
  container: HTMLElement,
  measures: StaffMeasure[],
  anchorState: NonNullable<PreviewAnchorState>,
  x: number,
  y: number,
): void {
  if (anchorState.mode !== 'measure') return
  const range = staffMeasureRangeAtPoint(container, measures, x, y)
  if (!range || range.start === anchorState.current.start) return
  anchorState.current = range
  applyPendingStaffMeasures(
    container,
    measures,
    anchorState.anchor,
    anchorState.current,
  )
}
