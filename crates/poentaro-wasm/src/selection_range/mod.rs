mod helpers;
mod label_fill;
mod lyric_label;
mod lyric_lyric;
mod measure;
mod mixed;
mod note_lyric;
mod note_note;
mod part_label;
mod system;
mod types;

use crate::types::{LyricSpanOut, NoteSpanOut};
// `pub(crate)`, not plain `use`: `component.rs` (a sibling module of this
// one, both children of `lib.rs`) needs `ClickableElementId`/
// `ResolveSelectionRangeResponse`/`NoteCellOut`/`LyricCellOut` to convert the
// WIT-generated shapes to/from these crate types — a private `use` here
// wouldn't propagate through `crate::selection_range::` far enough for
// `component.rs` to reach them, since `mod types;` below is itself
// module-private.
pub(crate) use label_fill::fully_selected_labels;
pub(crate) use system::resolve_system_selection_range_response;
pub(crate) use types::{
    ClickableElementId, LyricCellOut, NoteCellOut, ResolveSelectionRangeResponse,
};

/// Resolves the click-and-click range between two clickable elements into
/// the note/lyric cells it covers — the ID-based replacement for
/// pixel-marquee range resolution (see
/// `PLAN-clickable-element-id-selection.md`). Pure grouping over
/// already-fetched `note_spans`/`lyric_spans`, exactly like
/// `group_note_selection`'s doc comment describes for why callers must call
/// this on the main thread with cached spans, not re-parse `source`.
///
/// Every pair of clickable elements resolves. A cross-part pair of notes or
/// syllables covers every row between the two endpoints' rows for the
/// stretch of time from the earlier endpoint's onset to the later one's end
/// (see `helpers::cross_row_range`), so it never leaves a hole. Label and
/// measure pairs cover whole measures. `Err` means an endpoint's note or
/// syllable isn't in the spans.
///
/// Lyric parts are ordinary parts (own `source_part_index`, own rendered
/// rows), so no arm special-cases lyrics (no "sings along to" lookups).
///
/// Every label-mixed pair (`Note ↔ PartLabel`, `Lyric ↔ LyricLabel`,
/// `Note ↔ LyricLabel`, `Lyric ↔ PartLabel`, `PartLabel ↔ LyricLabel`) is
/// implemented too, closing out every combination this plan's Phase 1/Phase
/// 2 tables ever scoped — see `PLAN-clickable-element-id-selection.md`'s
/// Status section for the full per-pair design writeup and each topic
/// module's own doc comments below for each rule.
///
/// Implementation is split by topic across sibling modules — `measure`,
/// `note_note`, `note_lyric`, `lyric_lyric`, `part_label`, `lyric_label`,
/// and `mixed` — each owning a disjoint slice of the `(anchor, current)`
/// variant space via its own `pub(crate) fn resolve(..) -> Option<..>`.
/// This function just tries each in turn, falling through topic by topic
/// until one returns `Some`, ending in `ResolveSelectionRangeResponse::Err`
/// for any pair no topic module claims.
pub(crate) fn resolve_selection_range_response(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> ResolveSelectionRangeResponse {
    measure::resolve(note_spans, lyric_spans, anchor, current)
        .or_else(|| note_note::resolve(note_spans, lyric_spans, anchor, current))
        .or_else(|| note_lyric::resolve(note_spans, lyric_spans, anchor, current))
        .or_else(|| lyric_lyric::resolve(note_spans, lyric_spans, anchor, current))
        .or_else(|| part_label::resolve(note_spans, lyric_spans, anchor, current))
        .or_else(|| lyric_label::resolve(note_spans, lyric_spans, anchor, current))
        .or_else(|| mixed::resolve(note_spans, lyric_spans, anchor, current))
        .unwrap_or(ResolveSelectionRangeResponse::Err)
}

#[cfg(test)]
mod tests;
