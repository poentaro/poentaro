use serde::Serialize;

/// Identifies one clickable rendered element by exactly the `data-*` fields
/// TS already reads off it (`getNoteAtPoint`/`getLyricAtPoint`/
/// `getMeasureAtPoint`/`getPartLabelAtPoint`/`getLyricLabelAtPoint` in
/// `web/src/components/previewSelection.ts`/`previewLabelSelection.ts`) —
/// no new ID scheme, just a tagged union over what already exists. The
/// hand-written TS mirror lives in
/// `web/src/components/clickableElementId.ts`. Decoded from JS via
/// `serde_wasm_bindgen`, like [`crate::note_selection_types::NoteCellIn`];
/// `#[serde(rename_all = "camelCase")]` on the enum itself only renames the
/// `kind` tag values (`Note` → `"note"`, `PartLabel` → `"partLabel"`, ...) —
/// it does *not* cascade into each struct variant's own field names, so
/// every variant repeats the attribute to get its fields decoded as
/// `sourcePartIndex`/`noteId`/etc. Omitting it (verified against a
/// `serde_json` round-trip while chasing why `resolve_selection_range`
/// always fell through to `Err` for every combination, not just the
/// not-yet-ported ones) left every variant expecting snake_case keys no JS
/// caller ever sends, so decoding failed before `resolve_selection_range_
/// response`'s match arms ever ran.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum ClickableElementId {
    #[serde(rename_all = "camelCase")]
    Note {
        source_part_index: usize,
        note_id: usize,
    },
    #[serde(rename_all = "camelCase")]
    Lyric {
        source_part_index: usize,
        note_id: usize,
    },
    #[serde(rename_all = "camelCase")]
    Measure {
        measure_index_start: usize,
        measure_index_end: usize,
    },
    #[serde(rename_all = "camelCase")]
    PartLabel {
        source_part_index: usize,
        measure_index_start: usize,
        measure_index_end: usize,
    },
    #[serde(rename_all = "camelCase")]
    LyricLabel {
        source_part_index: usize,
        measure_index_start: usize,
        measure_index_end: usize,
    },
}

/// One resolved `(source_part_index, note_id)` cell — output mirror of
/// [`crate::note_selection_types::NoteCellIn`], matching TS's `NoteCell`
/// (`previewSelection.ts`).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct NoteCellOut {
    pub source_part_index: usize,
    pub note_id: usize,
}

/// One resolved `(source_part_index, note_id)` cell — output mirror
/// of [`crate::lyric_selection_types::LyricCellIn`], matching TS's
/// `LyricCell` (`previewSelection.ts`).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct LyricCellOut {
    pub source_part_index: usize,
    pub note_id: usize,
}

/// Result of `resolve_selection_range`. `Err` covers both a malformed
/// `JsValue` (matching the `unwrap_or_default` pattern already used for
/// `note_spans`/`selected_cells` elsewhere) and — for now — any
/// `(anchor, current)` combination not yet ported to ID-based resolution
/// (see `PLAN-clickable-element-id-selection.md`'s Phase 1 table): callers
/// must fall back to the existing pixel-marquee resolution for those.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ResolveSelectionRangeResponse {
    Ok {
        note_cells: Vec<NoteCellOut>,
        lyric_cells: Vec<LyricCellOut>,
    },
    Err,
}
