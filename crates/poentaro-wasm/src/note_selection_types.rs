use serde::Serialize;

/// Input mirror of `note_spans::NoteCell` — `component.rs` converts each
/// WIT-generated `NoteCellIn` record into this crate-internal shape
/// directly (a real, compile-time-typed conversion; no runtime decode/error
/// path, unlike the old `serde_wasm_bindgen::from_value` boundary this
/// replaced).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NoteCellIn {
    pub source_part_index: usize,
    pub note_id: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NoteSelectionRunOut {
    pub source_part_index: usize,
    pub part_abbreviation: Option<String>,
    pub measure_index: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum GroupNoteSelectionResponse {
    Ok {
        runs: Vec<NoteSelectionRunOut>,
    },
    /// Never actually constructed today — grouping over already-typed,
    /// already-fetched spans/cells can't fail. Kept (not removed) because
    /// `component.rs`'s `Guest` impl matches on it exhaustively when
    /// converting to the WIT-generated `group-note-selection-response`
    /// variant, which itself keeps an `err` case for API-shape parity with
    /// its `group-lyric-selection-response` sibling and every other
    /// spans-based response in `wit/world.wit`.
    #[allow(dead_code)]
    Err,
}
