//! Real bodies for the `Guest` methods `guest_impl.rs` dispatches to. Kept
//! at the exact by-value parameter types the `Guest` trait requires (see
//! `mod.rs`'s doc comment), so `needless_pass_by_value` is relaxed here.
#![allow(clippy::needless_pass_by_value)]

use super::*;

pub(super) fn greet(name: String) -> String {
    format!("hello, {name}")
}

// Phase 3, group 1 of PLAN-wit-bindgen-migration.md: same underlying
// logic as `wasm_boundary::group_note_selection`/`group_lyric_selection`
// (`crate::responses::group_note_selection_response`/
// `group_lyric_selection_response`), just converting to/from the
// WIT-generated shapes instead of `JsValue`/`serde_wasm_bindgen`. Both
// the old `#[wasm_bindgen] fn` and this method coexist until Phase 6.

pub(super) fn group_note_selection(
    note_spans: Vec<NoteSpan>,
    selected_cells: Vec<NoteCellIn>,
) -> GroupNoteSelectionResponse {
    let note_spans: Vec<crate::types::NoteSpanOut> =
        note_spans.into_iter().map(note_span_from_wit).collect();
    let selected_cells: Vec<crate::note_selection_types::NoteCellIn> = selected_cells
        .into_iter()
        .map(note_cell_in_from_wit)
        .collect();
    group_note_selection_response_to_wit(crate::responses::group_note_selection_response(
        &note_spans,
        &selected_cells,
    ))
}

pub(super) fn group_lyric_selection(
    lyric_spans: Vec<LyricSpan>,
    selected_cells: Vec<LyricCellIn>,
) -> GroupLyricSelectionResponse {
    let lyric_spans: Vec<crate::types::LyricSpanOut> =
        lyric_spans.into_iter().map(lyric_span_from_wit).collect();
    let selected_cells: Vec<crate::lyric_selection_types::LyricCellIn> = selected_cells
        .into_iter()
        .map(lyric_cell_in_from_wit)
        .collect();
    group_lyric_selection_response_to_wit(crate::responses::group_lyric_selection_response(
        &lyric_spans,
        &selected_cells,
    ))
}

pub(super) fn select_notes_in_byte_ranges(
    note_spans: Vec<NoteSpan>,
    ranges: Vec<ByteRange>,
) -> NoteSelection {
    let note_spans: Vec<poentaro::NoteSourceSpan> = note_spans
        .into_iter()
        .map(note_source_span_from_wit)
        .collect();
    note_selection_to_wit(poentaro::select_notes_in_byte_ranges(
        &byte_ranges_from_wit(ranges),
        &note_spans,
    ))
}

pub(super) fn select_lyrics_in_byte_ranges(
    lyric_spans: Vec<LyricSpan>,
    ranges: Vec<ByteRange>,
) -> LyricSelection {
    let lyric_spans: Vec<poentaro::LyricSourceSpan> = lyric_spans
        .into_iter()
        .map(lyric_source_span_from_wit)
        .collect();
    lyric_selection_to_wit(poentaro::select_lyrics_in_byte_ranges(
        &byte_ranges_from_wit(ranges),
        &lyric_spans,
    ))
}

// Phase 3, group 2 of PLAN-wit-bindgen-migration.md: same underlying
// logic as `wasm_boundary::list_note_spans`/`list_lyric_spans`/
// `list_measure_spans` (`crate::responses::list_note_spans_response`/
// `list_lyric_spans_response`/`list_measure_spans_response`), just
// converting to/from the WIT-generated shapes instead of
// `serde_wasm_bindgen`. Both the old `#[wasm_bindgen] fn`s and these
// methods coexist until Phase 6.

pub(super) fn list_measure_spans(source: String) -> ListMeasureSpansResponse {
    list_measure_spans_response_to_wit(crate::responses::list_measure_spans_response(&source))
}

pub(super) fn list_note_spans(
    source: String,
    enabled_tracks: Option<Vec<String>>,
) -> ListNoteSpansResponse {
    list_note_spans_response_to_wit(crate::responses::list_note_spans_response(
        &source,
        enabled_tracks.as_deref(),
    ))
}

pub(super) fn list_lyric_spans(
    source: String,
    enabled_tracks: Option<Vec<String>>,
) -> ListLyricSpansResponse {
    list_lyric_spans_response_to_wit(crate::responses::list_lyric_spans_response(
        &source,
        enabled_tracks.as_deref(),
    ))
}

// Phase 3, group 3 of PLAN-wit-bindgen-migration.md: same underlying
// logic as `wasm_boundary::list_parts`/`list_symbols`/`rename_symbol`/
// `get_measure_index_at_offset` (`crate::part_declarations::list_parts_response`/
// `crate::symbols::list_symbols_response`/`rename_symbol_response`/
// `crate::responses::get_measure_at_offset_response`), just converting
// to/from the WIT-generated shapes instead of `JsValue`/
// `serde_wasm_bindgen`. Both the old `#[wasm_bindgen] fn`s and these
// methods coexist until Phase 6.

pub(super) fn list_parts(
    source: String,
    raw_instruments: Vec<InstrumentInfo>,
) -> ListPartsResponse {
    let instruments: Vec<poentaro::parser::parts_parser::InstrumentInfo> = raw_instruments
        .into_iter()
        .map(instrument_info_from_wit)
        .collect();
    list_parts_response_to_wit(crate::part_declarations::list_parts_response(
        &source,
        &instruments,
    ))
}

pub(super) fn list_symbols(
    source: String,
    raw_instruments: Vec<InstrumentInfo>,
) -> ListSymbolsResponse {
    let instruments: Vec<poentaro::parser::parts_parser::InstrumentInfo> = raw_instruments
        .into_iter()
        .map(instrument_info_from_wit)
        .collect();
    list_symbols_response_to_wit(crate::symbols::list_symbols_response(&source, &instruments))
}

pub(super) fn rename_symbol(
    source: String,
    kind: SymbolKind,
    old_name: String,
    new_name: String,
    raw_instruments: Vec<InstrumentInfo>,
) -> RenameSymbolResponse {
    let instruments: Vec<poentaro::parser::parts_parser::InstrumentInfo> = raw_instruments
        .into_iter()
        .map(instrument_info_from_wit)
        .collect();
    rename_symbol_response_to_wit(crate::symbols::rename_symbol_response(
        &source,
        symbol_kind_out_from_wit(kind),
        &old_name,
        &new_name,
        &instruments,
    ))
}

pub(super) fn get_measure_index_at_offset(
    source: String,
    byte_offset: u32,
) -> MeasureAtOffsetResponse {
    measure_at_offset_response_to_wit(&crate::responses::get_measure_at_offset_response(
        &source,
        byte_offset as usize,
    ))
}

// Phase 3, group 4 of PLAN-wit-bindgen-migration.md: same underlying
// logic as `wasm_boundary::render`/`render_with_highlight_range`
// (`crate::responses::render_response`/
// `render_with_highlight_range_response`), just converting to/from the
// WIT-generated shapes instead of `JsValue`/`serde_wasm_bindgen`. Both
// the old `#[wasm_bindgen] fn`s and these methods coexist until Phase 6.
// Named `render_svg`/`render_svg_with_highlight_range` (not
// `render`/`render_with_highlight_range` matching the old fn names,
// group 3's convention) — see `wit/world.wit`'s doc comment on
// `render-svg` for why: a single-word WIT export name literally
// collides with `#[wasm_bindgen] pub fn render`'s own export symbol on
// `wasm32-unknown-unknown`.

/// A chosen preset that no longer parses or exists resolves as Default.
pub(super) fn resolve_part_visibility(
    source: String,
    parts: Vec<Part>,
    toggles: PartToggleState,
    preset_name: Option<String>,
) -> ResolvedPartVisibility {
    let parts: Vec<poentaro::PartInfo> = parts.into_iter().map(part_info_from_wit).collect();
    let preset = preset_name.and_then(|name| {
        poentaro::chosen_preset_from_source(&source, "input.poentaro", &name)
            .ok()
            .flatten()
    });
    resolved_part_visibility_to_wit(poentaro::resolve_part_visibility(
        &parts,
        &part_toggle_state_from_wit(toggles),
        preset.as_ref(),
    ))
}

pub(super) fn render_svg(
    source: String,
    visibility: ResolvedPartVisibility,
    raw_instruments: Vec<InstrumentInfo>,
    note_labels: NoteLabels,
    page_layout: PageLayout,
) -> RenderResponse {
    let instruments: Vec<poentaro::parser::parts_parser::InstrumentInfo> = raw_instruments
        .into_iter()
        .map(instrument_info_from_wit)
        .collect();
    render_response_to_wit(crate::responses::render_response(
        &source,
        &resolved_part_visibility_from_wit(visibility),
        &instruments,
        note_labels_from_wit(note_labels),
        page_layout_from_wit(page_layout),
    ))
}

pub(super) fn render_svg_with_highlight_range(
    source: String,
    raw_measure_ranges: Vec<MeasureRangeIn>,
    visibility: ResolvedPartVisibility,
    raw_instruments: Vec<InstrumentInfo>,
    note_labels: NoteLabels,
    page_layout: PageLayout,
) -> RenderResponse {
    let instruments: Vec<poentaro::parser::parts_parser::InstrumentInfo> = raw_instruments
        .into_iter()
        .map(instrument_info_from_wit)
        .collect();
    let measure_ranges: Vec<poentaro::grid_layout::MeasureRange> = raw_measure_ranges
        .into_iter()
        .map(measure_range_in_from_wit)
        .collect();
    render_response_to_wit(crate::responses::render_with_highlight_range_response(
        &source,
        &measure_ranges,
        &resolved_part_visibility_from_wit(visibility),
        &instruments,
        note_labels_from_wit(note_labels),
        page_layout_from_wit(page_layout),
    ))
}

pub(super) fn render_mei(
    source: String,
    visibility: ResolvedPartVisibility,
    raw_instruments: Vec<InstrumentInfo>,
    clef: StaffClef,
) -> RenderMeiResponse {
    let instruments: Vec<poentaro::parser::parts_parser::InstrumentInfo> = raw_instruments
        .into_iter()
        .map(instrument_info_from_wit)
        .collect();
    if let Err(diagnostics) = crate::opened_document::open_checked(&source, &instruments) {
        return RenderMeiResponse::Err(diagnostics_error_to_wit(&diagnostics));
    }
    match poentaro::render_mei_from_source_with_visibility(
        &source,
        "input.poentaro",
        &resolved_part_visibility_from_wit(visibility),
        &instruments,
        clef_choice_from_wit(clef),
    ) {
        Ok(document) => RenderMeiResponse::Ok(RenderMeiSuccess {
            mei: document.mei,
            measures: document
                .measures
                .into_iter()
                .map(|measure| StaffMeasure {
                    element_id: measure.element_id,
                    measure_index: measure.measure_index as u32,
                })
                .collect(),
        }),
        Err(error) => RenderMeiResponse::Err(diagnostics_error_to_wit(&[
            crate::diagnostics::diagnostic_from_error(&error),
        ])),
    }
}
