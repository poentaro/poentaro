use serde::Serialize;

/// Input mirror of `lyric_spans::LyricCell` — `component.rs` converts each
/// WIT-generated `LyricCellIn` record into this crate-internal shape
/// directly (a real, compile-time-typed conversion; no runtime decode/error
/// path, unlike the old `serde_wasm_bindgen::from_value` boundary this
/// replaced).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LyricCellIn {
    pub source_part_index: usize,
    pub note_id: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LyricSelectionRunOut {
    pub source_part_index: usize,
    pub part_abbreviation: Option<String>,
    pub measure_index: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum GroupLyricSelectionResponse {
    Ok {
        runs: Vec<LyricSelectionRunOut>,
    },
    /// Never actually constructed today — see
    /// `GroupNoteSelectionResponse::Err`'s doc comment (`note_selection_types.rs`)
    /// for why this is kept rather than removed.
    #[allow(dead_code)]
    Err,
}
