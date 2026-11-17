use serde::Serialize;

use crate::svg_types::SvgDocumentOut;

pub(crate) use crate::lyric_selection_types::{
    GroupLyricSelectionResponse, LyricCellIn, LyricSelectionRunOut,
};
pub(crate) use crate::note_selection_types::{
    GroupNoteSelectionResponse, NoteCellIn, NoteSelectionRunOut,
};

#[cfg(feature = "midi")]
pub(crate) use crate::types_export::{GenerateMidiResponse, GenerateSplitMidisResponse};
#[cfg(feature = "mp3")]
pub(crate) use crate::types_export::{GenerateMp3Response, GenerateSplitMp3sResponse};
#[cfg(feature = "pdf")]
pub(crate) use crate::types_export::{GeneratePdfResponse, GenerateSplitPdfsResponse};
#[cfg(feature = "wav")]
pub(crate) use crate::types_export::{
    GenerateSplitWavsResponse, GenerateWavResponse, NoteTimingOut, NoteTimingsResponse,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SpanOut {
    /// UTF-8 byte offset (inclusive).
    pub start: usize,
    /// UTF-8 byte offset (exclusive).
    pub end: usize,
}

/// One inclusive measure-index range from JS, deserialized via
/// `serde_wasm_bindgen::from_value` rather than a wasm-bindgen `Vec<T>` param
/// (which only works for `JsCast` types) — see [`crate::note_selection_types::NoteCellIn`]
/// for the same convention. Maps 1:1 onto [`poentaro::grid_layout::MeasureRange`].
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MeasureRangeIn {
    pub start: usize,
    pub end: usize,
}

impl From<MeasureRangeIn> for poentaro::grid_layout::MeasureRange {
    fn from(r: MeasureRangeIn) -> Self {
        poentaro::grid_layout::MeasureRange {
            start: r.start,
            end: r.end,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DiagnosticOut {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub span: SpanOut,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PartOut {
    pub abbreviation: String,
    pub display_name: String,
    pub sounds: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum RenderResponse {
    Ok {
        documents: Vec<SvgDocumentOut>,
        diagnostics: Vec<DiagnosticOut>,
        diagnostic_view_zones: Vec<DiagnosticViewZoneOut>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
        diagnostic_view_zones: Vec<DiagnosticViewZoneOut>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListPartsResponse {
    Ok {
        parts: Vec<PartOut>,
        declarations: Vec<poentaro::SourcePartDeclaration>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListPartDeclarationsResponse {
    Ok {
        declarations: Vec<poentaro::SourcePartDeclaration>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SymbolKindOut {
    Abbreviation,
    SectionLabel,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OccurrenceRoleOut {
    Declaration,
    Reference,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SymbolOccurrenceOut {
    pub span: SpanOut,
    /// The region a caret may rest in to trigger a rename of this occurrence;
    /// usually equal to `span` but wider for occurrences whose renamable text
    /// sits inside a larger token (e.g. a section label declaration's `span`
    /// covers just the quoted text in `label="C"`, while `hit_span` covers
    /// the whole token).
    pub hit_span: SpanOut,
    pub role: OccurrenceRoleOut,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SymbolOut {
    pub name: String,
    pub kind: SymbolKindOut,
    pub occurrences: Vec<SymbolOccurrenceOut>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ListSymbolsResponse {
    Ok { symbols: Vec<SymbolOut> },
    Err { diagnostics: Vec<DiagnosticOut> },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TextEditOut {
    pub span: SpanOut,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum RenameSymbolResponse {
    Ok { edits: Vec<TextEditOut> },
    Err { diagnostics: Vec<DiagnosticOut> },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum MeasureAtOffsetResponse {
    Ok { measure_index: usize },
    NotInMeasure,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MeasureSpanOut {
    /// Inclusive start of note content (for cursor/selection mapping).
    pub start: usize,
    /// Exclusive end of measure content in source.
    pub end: usize,
    /// Byte offset of the first source line in this measure group, for view zones.
    pub view_zone_start: usize,
    /// Section label from `label="..."` directive, if present on this measure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_label: Option<String>,
    /// 1-indexed first line of this measure (inclusive).
    pub start_line: usize,
    /// 1-indexed last line of this measure (inclusive).
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NoteSpanOut {
    /// Index into the compiled score's parts, matching the SVG's
    /// `data-part-index` attribute on a `Tag::Note` group.
    pub source_part_index: usize,
    /// The part's abbreviation as Rust resolved it after hidden parts were
    /// filtered out — see `note_spans::NoteSourceSpan::part_abbreviation`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_abbreviation: Option<String>,
    /// Matches the SVG's `data-note-id` attribute on a `Tag::Note` group.
    pub note_id: usize,
    /// Index into the score's measures, in source order.
    pub measure_index: usize,
    /// Inclusive start byte of this event's token in the original source.
    /// Absent for a rest, which has no single source token to map to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<usize>,
    /// Exclusive end byte of this event's token in the original source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<usize>,
    /// When this event sounds within its measure — see
    /// `note_spans::MeasureTiming`. Read only by Rust (selection ranges).
    pub timing: MeasureTimingOut,
}

/// See `note_spans::MeasureTiming`.
#[derive(Debug, Clone, Copy, Serialize, serde::Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MeasureTimingOut {
    /// Quarter-beats from the start of the measure to the onset.
    pub offset: u32,
    /// Length in quarter-beats.
    pub duration: u32,
}

impl From<poentaro::note_spans::MeasureTiming> for MeasureTimingOut {
    fn from(timing: poentaro::note_spans::MeasureTiming) -> Self {
        MeasureTimingOut {
            offset: timing.offset,
            duration: timing.duration,
        }
    }
}

impl From<MeasureTimingOut> for poentaro::note_spans::MeasureTiming {
    fn from(timing: MeasureTimingOut) -> Self {
        poentaro::note_spans::MeasureTiming {
            offset: timing.offset,
            duration: timing.duration,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ListNoteSpansResponse {
    Ok { spans: Vec<NoteSpanOut> },
    Err,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LyricSpanOut {
    /// Index into the compiled score's parts, matching the SVG's
    /// `data-part-index` attribute on a `Tag::Lyric` group.
    pub source_part_index: usize,
    /// See `NoteSpanOut::part_abbreviation`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_abbreviation: Option<String>,
    /// Matches the SVG's `data-note-id` attribute on a `Tag::Lyric` group.
    pub note_id: usize,
    /// Index into the score's measures, in source order.
    pub measure_index: usize,
    /// Inclusive start byte of this syllable's own token in the original source.
    pub start: usize,
    /// Exclusive end byte of this syllable's own token in the original source.
    pub end: usize,
    /// See `NoteSpanOut::timing`.
    pub timing: MeasureTimingOut,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ListLyricSpansResponse {
    Ok { spans: Vec<LyricSpanOut> },
    Err,
}

#[derive(Debug, Clone, Serialize)]
pub struct SectionRangeOut {
    pub first_line: usize,
    pub last_line: usize,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SequenceEntryOut {
    pub label: String,
    pub start_measure_index: usize,
    pub end_measure_index: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ListMeasureSpansResponse {
    Ok {
        spans: Vec<MeasureSpanOut>,
        section_ranges: Vec<SectionRangeOut>,
        sequence_entries: Vec<SequenceEntryOut>,
    },
    Err,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DiagnosticMessageOut {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DiagnosticViewZoneOut {
    pub severity: DiagnosticSeverity,
    /// 1-based line number; view zone is inserted after this line.
    pub after_line_number: usize,
    pub messages: Vec<DiagnosticMessageOut>,
}
