//! The selection-range exports, called from `guest_impl`.

#![allow(clippy::needless_pass_by_value)]

use super::*;

// Same underlying `crate::selection_range::resolve_selection_range_response`
// the old `selection_range::resolve_selection_range` `#[wasm_bindgen]
// fn` calls — that function was already unconditional (not gated on
// `wasm-bindgen-boundary`), unlike this group's other findings, so no
// prerequisite-ungating fix was needed here, only widening
// `ClickableElementId`/`NoteCellOut`/`LyricCellOut`/
// `ResolveSelectionRangeResponse`'s reachability from `component.rs` (see
// `selection_range/mod.rs`'s own comment on its `pub(crate) use`).

pub(super) fn resolve_selection_range(
    note_spans: Vec<NoteSpan>,
    lyric_spans: Vec<LyricSpan>,
    anchor: ClickableElementId,
    current: ClickableElementId,
) -> ResolveSelectionRangeResponse {
    let note_spans: Vec<crate::types::NoteSpanOut> =
        note_spans.into_iter().map(note_span_from_wit).collect();
    let lyric_spans: Vec<crate::types::LyricSpanOut> =
        lyric_spans.into_iter().map(lyric_span_from_wit).collect();
    let anchor = clickable_element_id_from_wit(anchor);
    let current = clickable_element_id_from_wit(current);
    resolve_selection_range_response_to_wit(
        crate::selection_range::resolve_selection_range_response(
            &note_spans,
            &lyric_spans,
            &anchor,
            &current,
        ),
    )
}

pub(super) fn resolve_system_selection_range(
    note_spans: Vec<NoteSpan>,
    lyric_spans: Vec<LyricSpan>,
    systems: Vec<MeasureRangeIn>,
    anchor: ClickableElementId,
    current: ClickableElementId,
) -> ResolveSelectionRangeResponse {
    let note_spans: Vec<crate::types::NoteSpanOut> =
        note_spans.into_iter().map(note_span_from_wit).collect();
    let lyric_spans: Vec<crate::types::LyricSpanOut> =
        lyric_spans.into_iter().map(lyric_span_from_wit).collect();
    let systems: Vec<poentaro::grid_layout::MeasureRange> =
        systems.into_iter().map(measure_range_in_from_wit).collect();
    resolve_selection_range_response_to_wit(
        crate::selection_range::resolve_system_selection_range_response(
            &note_spans,
            &lyric_spans,
            &systems,
            &clickable_element_id_from_wit(anchor),
            &clickable_element_id_from_wit(current),
        ),
    )
}

pub(super) fn fully_selected_labels(
    note_spans: Vec<NoteSpan>,
    lyric_spans: Vec<LyricSpan>,
    labels: Vec<ClickableElementId>,
    note_cells: Vec<NoteCellIn>,
    lyric_cells: Vec<LyricCellIn>,
) -> Vec<bool> {
    let note_spans: Vec<crate::types::NoteSpanOut> =
        note_spans.into_iter().map(note_span_from_wit).collect();
    let lyric_spans: Vec<crate::types::LyricSpanOut> =
        lyric_spans.into_iter().map(lyric_span_from_wit).collect();
    let labels: Vec<crate::selection_range::ClickableElementId> = labels
        .into_iter()
        .map(clickable_element_id_from_wit)
        .collect();
    let note_cells: Vec<crate::note_selection_types::NoteCellIn> =
        note_cells.into_iter().map(note_cell_in_from_wit).collect();
    let lyric_cells: Vec<crate::lyric_selection_types::LyricCellIn> = lyric_cells
        .into_iter()
        .map(lyric_cell_in_from_wit)
        .collect();
    crate::selection_range::fully_selected_labels(
        &note_spans,
        &lyric_spans,
        &labels,
        &note_cells,
        &lyric_cells,
    )
}
