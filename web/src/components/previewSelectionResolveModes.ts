import {
  type ClickableElementId,
  poentaroWasm,
  type ResolveSelectionRangeResponse,
} from '../poentaroWasm'
import type { LyricSpan, NoteSpan } from '../types'
import type {
  ElementAnchorState,
  PreviewAnchorState,
} from './previewAnchorState'
import {
  anyClickableElementIdAtPoint,
  measureClickableElementId,
} from './previewClickableElementIdBuilders'
import {
  applyPersistedLyricHighlights,
  applyPersistedNoteHighlights,
} from './previewRangeHighlights'
import {
  getMeasureAtPoint,
  type MeasureRange,
  renderedSystems,
} from './previewSelection'
import type { ResolvedSelection } from './previewSelectionResolver'

/** Everything a mode's own resolver needs beyond `anchorState` itself —
 * shared by the resolvers below and their label-mode siblings in
 * `previewSelectionResolveLabelModes.ts`. */
export interface ResolveModeArgs {
  container: HTMLDivElement | null
  point: { x: number; y: number } | undefined
  currentIdHint: ClickableElementId | undefined
  noteSpans: NoteSpan[]
  lyricSpans: LyricSpan[]
}

type MeasureAnchorState = Extract<
  NonNullable<PreviewAnchorState>,
  { mode: 'measure' | 'bar-number-system' }
>
type NoteAnchorState = Extract<ElementAnchorState, { mode: 'note' }>
type LyricAnchorState = Extract<ElementAnchorState, { mode: 'lyric' }>

const EMPTY_SELECTION: ResolvedSelection = { noteCells: [], lyricCells: [] }

/** The click target the gesture's current end resolves to: the anchor
 * itself when there's no `point` (the first click's self-commit and a
 * cancelled gesture's revert), otherwise whatever is at `point`, or the last
 * resolved end when `point` misses every click target. */
export function resolveCurrentId(
  anchorState: ElementAnchorState,
  point: { x: number; y: number } | undefined,
  currentIdHint: ClickableElementId | undefined,
): ClickableElementId {
  if (point === undefined) return anchorState.anchorId
  return (
    currentIdHint ??
    anyClickableElementIdAtPoint(point.x, point.y) ??
    anchorState.currentId
  )
}

/** The measure range the gesture's current end resolves to — the measure
 * modes' counterpart of `resolveCurrentId`. */
function resolveCurrentMeasures(
  anchorState: MeasureAnchorState,
  point: { x: number; y: number } | undefined,
): MeasureRange {
  if (point === undefined) return anchorState.anchor
  return getMeasureAtPoint(point.x, point.y) ?? anchorState.current
}

/** The cells a wasm range response names, or `undefined` for `Err` (an
 * endpoint wasn't in the spans). */
export function selectionFromResponse(
  response: ResolveSelectionRangeResponse,
): ResolvedSelection | undefined {
  return response.tag === 'ok'
    ? { noteCells: response.val.noteCells, lyricCells: response.val.lyricCells }
    : undefined
}

/** Paints `selection`'s notes and syllables as selected. */
export function paintSelection(
  container: HTMLDivElement | null,
  selection: ResolvedSelection,
): void {
  if (!container) return
  applyPersistedNoteHighlights(container, selection.noteCells)
  applyPersistedLyricHighlights(container, selection.lyricCells)
}

export function resolveMeasureSelection(
  anchorState: Extract<MeasureAnchorState, { mode: 'measure' }>,
  { container, point, currentIdHint, noteSpans, lyricSpans }: ResolveModeArgs,
): ResolvedSelection {
  const selection =
    selectionFromResponse(
      poentaroWasm().resolveSelectionRange(
        noteSpans,
        lyricSpans,
        anchorState.anchorId,
        currentIdHint ??
          measureClickableElementId(resolveCurrentMeasures(anchorState, point)),
      ),
    ) ?? EMPTY_SELECTION
  paintSelection(container, selection)
  return selection
}

/** Every part of every system from the anchor's through the current end's
 * (see `resolve_system_selection_range_response`): a bar number's
 * click-and-click and a Cmd/Ctrl-click on a part label. */
export function resolveSystemSelection(
  anchorId: ClickableElementId,
  currentId: ClickableElementId,
  { container, noteSpans, lyricSpans }: ResolveModeArgs,
): ResolvedSelection {
  const selection =
    selectionFromResponse(
      poentaroWasm().resolveSystemSelectionRange(
        noteSpans,
        lyricSpans,
        container ? renderedSystems(container) : [],
        anchorId,
        currentId,
      ),
    ) ?? EMPTY_SELECTION
  paintSelection(container, selection)
  return selection
}

export function resolveBarNumberSystemSelection(
  anchorState: Extract<MeasureAnchorState, { mode: 'bar-number-system' }>,
  args: ResolveModeArgs,
): ResolvedSelection {
  return resolveSystemSelection(
    anchorState.anchorId,
    measureClickableElementId(resolveCurrentMeasures(anchorState, args.point)),
    args,
  )
}

export function resolveNoteSelection(
  anchorState: NoteAnchorState,
  { container, point, currentIdHint, noteSpans, lyricSpans }: ResolveModeArgs,
): ResolvedSelection {
  const selection = selectionFromResponse(
    poentaroWasm().resolveSelectionRange(
      noteSpans,
      lyricSpans,
      anchorState.anchorId,
      resolveCurrentId(anchorState, point, currentIdHint),
    ),
  )
  // A range with no notes collapses to the anchoring note, so the gesture
  // always selects at least what the first click landed on.
  const resolved =
    selection && selection.noteCells.length > 0
      ? selection
      : { noteCells: [anchorState.anchorId.val], lyricCells: [] }
  paintSelection(container, resolved)
  return resolved
}

export function resolveLyricSelection(
  anchorState: LyricAnchorState,
  { container, point, currentIdHint, noteSpans, lyricSpans }: ResolveModeArgs,
): ResolvedSelection {
  const selection =
    selectionFromResponse(
      poentaroWasm().resolveSelectionRange(
        noteSpans,
        lyricSpans,
        anchorState.anchorId,
        resolveCurrentId(anchorState, point, currentIdHint),
      ),
    ) ?? EMPTY_SELECTION
  paintSelection(container, selection)
  return selection
}
