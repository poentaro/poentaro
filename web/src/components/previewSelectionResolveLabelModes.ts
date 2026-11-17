import { poentaroWasm } from '../poentaroWasm'
import type { ElementAnchorState } from './previewAnchorState'
import {
  applyPersistedLyricLabelHighlights,
  applyPersistedPartLabelHighlights,
} from './previewLabelRangeHighlights'
import {
  paintSelection,
  type ResolveModeArgs,
  resolveCurrentId,
  resolveSystemSelection,
  selectionFromResponse,
} from './previewSelectionResolveModes'
import type { ResolvedSelection } from './previewSelectionResolver'

type PartLabelAnchorState = Extract<ElementAnchorState, { mode: 'part-label' }>
type PartLabelSystemAnchorState = Extract<
  ElementAnchorState,
  { mode: 'part-label-system' }
>
type LyricLabelAnchorState = Extract<
  ElementAnchorState,
  { mode: 'lyric-label' }
>

/** Resolves a label-anchored range in wasm and paints it. */
function resolveLabelRange(
  anchorState: PartLabelAnchorState | LyricLabelAnchorState,
  { container, point, currentIdHint, noteSpans, lyricSpans }: ResolveModeArgs,
): ResolvedSelection {
  const selection = selectionFromResponse(
    poentaroWasm().resolveSelectionRange(
      noteSpans,
      lyricSpans,
      anchorState.anchorId,
      resolveCurrentId(anchorState, point, currentIdHint),
    ),
  ) ?? { noteCells: [], lyricCells: [] }
  paintSelection(container, selection)
  return selection
}

export function resolvePartLabelSelection(
  anchorState: PartLabelAnchorState,
  args: ResolveModeArgs,
): ResolvedSelection {
  const selection = resolveLabelRange(anchorState, args)
  // Paints the label fill right away rather than waiting for the selection
  // to round-trip back down as a prop, which would flash every selected
  // label's fill off for a frame.
  if (args.container) {
    applyPersistedPartLabelHighlights(
      args.container,
      args.noteSpans,
      selection.noteCells,
    )
  }
  return selection
}

/** Every part of every system the gesture touches — the Cmd/Ctrl-gated
 * sibling of `resolvePartLabelSelection`, whose range covers only the rows
 * between the two endpoints. */
export function resolvePartLabelSystemSelection(
  anchorState: PartLabelSystemAnchorState,
  args: ResolveModeArgs,
): ResolvedSelection {
  const selection = resolveSystemSelection(
    anchorState.anchorId,
    resolveCurrentId(anchorState, args.point, args.currentIdHint),
    args,
  )
  if (args.container) {
    applyPersistedPartLabelHighlights(
      args.container,
      args.noteSpans,
      selection.noteCells,
    )
  }
  return selection
}

export function resolveLyricLabelSelection(
  anchorState: LyricLabelAnchorState,
  args: ResolveModeArgs,
): ResolvedSelection {
  const selection = resolveLabelRange(anchorState, args)
  if (args.container) {
    applyPersistedLyricLabelHighlights(
      args.container,
      args.lyricSpans,
      selection.lyricCells,
    )
  }
  return selection
}
