use serde::Serialize;

use crate::types::DiagnosticOut;

#[cfg(feature = "wav")]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum GenerateWavResponse {
    Ok { wav: Vec<u8> },
    Err { diagnostics: Vec<DiagnosticOut> },
}

#[cfg(feature = "wav")]
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct NoteTimingOut {
    pub source_part_index: usize,
    pub note_id: usize,
    pub start_s: f64,
    pub end_s: f64,
}

#[cfg(feature = "wav")]
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum NoteTimingsResponse {
    Ok {
        /// Elapsed-seconds start/end of every sounding note/rest, keyed by
        /// `(source_part_index, note_id)` — matching the `data-part-index`/
        /// `data-note-id` attributes on each `data-tag="note"` group in the
        /// rendered SVG.
        timings: Vec<NoteTimingOut>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[cfg(feature = "pdf")]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum GeneratePdfResponse {
    Ok { pdf: Vec<u8> },
    Err { diagnostics: Vec<DiagnosticOut> },
}

#[cfg(feature = "pdf")]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum GenerateSplitPdfsResponse {
    Ok { zip: Vec<u8> },
    Err { diagnostics: Vec<DiagnosticOut> },
}

#[cfg(feature = "midi")]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum GenerateMidiResponse {
    Ok { midi: Vec<u8> },
    Err { diagnostics: Vec<DiagnosticOut> },
}

#[cfg(feature = "midi")]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum GenerateSplitMidisResponse {
    Ok { zip: Vec<u8> },
    Err { diagnostics: Vec<DiagnosticOut> },
}

#[cfg(feature = "wav")]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum GenerateSplitWavsResponse {
    Ok { zip: Vec<u8> },
    Err { diagnostics: Vec<DiagnosticOut> },
}

#[cfg(feature = "mp3")]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum GenerateMp3Response {
    Ok { mp3: Vec<u8> },
    Err { diagnostics: Vec<DiagnosticOut> },
}

#[cfg(feature = "mp3")]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum GenerateSplitMp3sResponse {
    Ok { zip: Vec<u8> },
    Err { diagnostics: Vec<DiagnosticOut> },
}
