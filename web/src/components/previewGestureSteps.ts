import type { RefObject } from 'react'
import type { ClickableElementId } from '../poentaroWasm'
import type { PreviewAnchorState } from './previewAnchorState'
import {
  fireCommit,
  type HandlePreviewClickArgs,
  resolveSelection,
} from './previewSelectionResolver'
import { applyStaffMeasureHighlights } from './previewStaffMeasures'

/** Anchors `anchorStateRef` to `newState` and immediately self-commits its
 * single-target resolution (matching a plain click's long-standing
 * instant-select behavior — see the single-click e2e specs) — but, unlike
 * the old held-button drag model, leaves `anchorStateRef` anchored rather than
 * resetting to idle: a second click can still land elsewhere and widen this
 * into a real range (see `handleCommitClick`). */
export function anchorAndCommit(
  anchorStateRef: RefObject<PreviewAnchorState>,
  newState: NonNullable<PreviewAnchorState>,
  args: HandlePreviewClickArgs,
): void {
  anchorStateRef.current = newState
  // Arms the one-shot reveal suppression (see `HandlePreviewClickArgs`'s
  // `suppressNextRevealRef` doc comment) — set here, ahead of `fireCommit`,
  // so it's already armed by the time this self-commit's Monaco selection
  // round-trip debounces into `Preview.tsx`'s scroll-to-selection effect.
  args.suppressNextRevealRef.current = true
  // Flips on the pending-second-click banner/color (see
  // `HandlePreviewClickArgs`'s `onPendingSecondClickChange` doc comment)
  // ahead of the self-commit below, same ordering rationale as
  // `suppressNextRevealRef` above.
  args.onPendingSecondClickChange?.(true)
  fireCommit(resolveSelection(newState, undefined, undefined, args), args)
}

/** Resets `anchorStateRef` to idle and re-applies the highlight `anchorState`'s
 * anchoring click already committed — used both when a second click misses
 * every recognizable target and when the gesture is cancelled via Escape
 * (see `usePreviewClickSelection`). No callback fires: the anchoring click's
 * own commit already did, and nothing about that selection has changed. */
export function cancelAnchor(
  anchorStateRef: RefObject<PreviewAnchorState>,
  anchorState: NonNullable<PreviewAnchorState>,
  args: HandlePreviewClickArgs,
): void {
  const container = args.previewPagesRef.current
  if (args.staffMeasures) {
    // The staff view's hover preview painted over the anchor's highlight
    // directly (see `hoverStaffMeasure`), so put the anchor's back.
    if (container && anchorState.mode === 'measure') {
      applyStaffMeasureHighlights(container, args.staffMeasures, [
        anchorState.anchor,
      ])
    }
  } else {
    resolveSelection(anchorState, undefined, undefined, args)
  }
  anchorStateRef.current = null
  args.onPendingSecondClickChange?.(false)
}

/** Resolves the range between `anchorState`'s anchor and the second click
 * at `point` (or `currentIdHint`, when the caller already knows what was
 * clicked — see `resolveSelection`), commits it, and returns
 * `anchorStateRef` to idle. */
export function commitAnchoredGesture(
  anchorState: NonNullable<PreviewAnchorState>,
  point: { x: number; y: number },
  currentIdHint: ClickableElementId | undefined,
  args: HandlePreviewClickArgs,
): void {
  // Arms the one-shot reveal suppression (see `HandlePreviewClickArgs`'s
  // `suppressNextRevealRef` doc comment and `anchorAndCommit`'s identical
  // arming above) — without this, this second click's own self-committed
  // Monaco selection round-trips back through `Preview.tsx`'s
  // scroll-to-selection effect and re-scrolls the SVG to the range this
  // click just made, fighting whatever scroll position the user is already
  // looking at.
  args.suppressNextRevealRef.current = true
  fireCommit(resolveSelection(anchorState, point, currentIdHint, args), args)
  args.anchorStateRef.current = null
  args.onPendingSecondClickChange?.(false)
}
