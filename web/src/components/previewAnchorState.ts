import type { ClickableElementId } from '../poentaroWasm'
import type { MeasureRange } from './previewSelection'

// One discriminated ref rather than a separate ref per mode: a single click
// can only ever anchor one mode (see `handlePreviewClick`'s idle-click
// branch), and independent refs risk several firing at once. This only
// governs which mode a click *anchors*: every mode resolves its range in
// wasm, which may return notes and lyric syllables alike.
//
// Selection is a click-and-click gesture, not a held-button drag: a first
// click anchors one of the modes below and eagerly highlights whatever it
// landed on; a plain `mousemove` (no button held) live-updates the
// anchor→hover range for mouse users; a second click resolves the range
// between the anchor and whatever that click landed on and commits it (see
// `handlePreviewClick`). This is what lets the same gesture work for touch:
// a tap synthesizes a `click` with no intervening movement, so two taps are
// just two clicks.
//
// Every mode keeps the `ClickableElementId` its anchoring click landed on
// (`anchorId`) and the one the gesture last resolved to (`currentId`, or a
// `current` measure range for the measure modes). A hover or second click
// that misses every click target keeps `currentId` as it was, so the range
// stays where the pointer last pointed. A click that misses every note/
// lyric click target but still lands inside a measure (a bar-line/gutter
// pixel) anchors 'measure' mode directly, the same as holding Cmd/Ctrl at
// the first click or landing exactly on a bar line's own divider (see
// `previewClickHandler.ts`).
export type PreviewAnchorState =
  | {
      mode: 'measure'
      anchor: MeasureRange
      current: MeasureRange
      anchorId: Extract<ClickableElementId, { tag: 'measure' }>
    }
  | {
      // A bar number's own click target anchors this instead of plain
      // 'measure' mode: resolution (`resolveBarNumberSystemSelection`)
      // selects every part of every system from the anchor's through the
      // second click's, rather than stopping at the exact measure the second
      // click landed in. See
      // `bar-number-click-and-click-selects-whole-systems.feature`.
      mode: 'bar-number-system'
      anchor: MeasureRange
      current: MeasureRange
      anchorId: Extract<ClickableElementId, { tag: 'measure' }>
    }
  | {
      mode: 'note'
      anchorId: Extract<ClickableElementId, { tag: 'note' }>
      currentId: ClickableElementId
    }
  | {
      mode: 'part-label'
      anchorId: Extract<ClickableElementId, { tag: 'part-label' }>
      currentId: ClickableElementId
    }
  | {
      // Cmd/Ctrl-click on a part label: the label-side mirror of the bar
      // number's 'bar-number-system' mode, selecting every part of every
      // system the gesture touches.
      mode: 'part-label-system'
      anchorId: Extract<ClickableElementId, { tag: 'part-label' }>
      currentId: ClickableElementId
    }
  | {
      // The lyric-label mirror of 'part-label' above: a click on a lyric
      // row's own label (e.g. "M:v1") anchors this instead.
      mode: 'lyric-label'
      anchorId: Extract<ClickableElementId, { tag: 'lyric-label' }>
      currentId: ClickableElementId
    }
  | {
      mode: 'lyric'
      anchorId: Extract<ClickableElementId, { tag: 'lyric' }>
      currentId: ClickableElementId
    }
  | null

/** The anchored modes whose ends are click targets rather than measures. */
export type ElementAnchorState = Exclude<
  NonNullable<PreviewAnchorState>,
  { mode: 'measure' | 'bar-number-system' }
>
