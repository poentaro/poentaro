//! Part stream editing: edit **one part** of the `score` as a single
//! continuous stream of musical elements (no measure groups, no barlines, no
//! directives), then apply it back into the measure-grouped source.
//!
//! - [`extract_part_stream`] turns the part's per-measure lines into one
//!   stream of **measure chunks** (implicit trailing padding materialized as
//!   explicit `-`, unwritten measures shown as rests, as `@` for a follow
//!   part, or as `_` for a lyric part; a chords part's lyric stream ends
//!   each measure with `|`).
//! - [`apply_part_stream`] re-bars an edited stream by each measure's beat
//!   capacity (a lyric stream: by its target's lyric slots, or at its `|`s
//!   under a chords part) and rewrites only
//!   the measures whose chunk changed; every other line (other parts,
//!   directives, comments) stays byte-for-byte.
//! - [`list_streamable_parts`] lists every declared part with the reason it
//!   can't be stream-edited, if any.

mod apply;
mod chord_lyrics;
mod diagnostics;
mod document;
mod editable;
mod extract;
mod focus;
mod format;
mod lyrics;
mod selection;
mod split;
#[cfg(test)]
mod stream_generator;
mod timing;

use crate::ast::parsed::{PartDecl, PartKind};
use crate::error::Span;
use document::ScoreDocument;
use editable::editable_part;
pub use focus::{part_focus_measures, FocusMeasure};
pub use format::{format_part_stream, FormattedPartStream, StreamCursor, StreamTextEdit};
pub use selection::{part_stream_ranges_to_source, source_ranges_to_part_stream};

/// A follow part's measure that it doesn't write, so it copies its target.
const FOLLOW_PLACEHOLDER: &str = "@";
/// A lyric part's measure without lyrics.
const NO_LYRICS_MEASURE: &str = "_";

/// What kind of line a streamable part writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamablePartKind {
    Notes,
    Chords,
    Percussion,
    Follow,
    Lyrics,
}

/// One declared part, as listed in the part picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamablePart {
    pub abbreviation: String,
    pub display_name: String,
    pub kind: StreamablePartKind,
    /// `None` when the part can be stream-edited; otherwise why not.
    pub ineligible_reason: Option<String>,
}

/// A part's measures flattened into one stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartStream {
    pub text: String,
    /// Byte offset in `text` where each measure's chunk starts (index = measure index).
    pub measure_starts: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartStreamError {
    /// `parts` / `score` could not be resolved.
    SourceDoesNotParse { message: String },
    /// No part with this abbreviation is declared.
    UnknownPart { abbreviation: String },
    /// The part exists but cannot be stream-edited.
    Ineligible { reason: String },
}

impl PartStreamError {
    /// A human-readable explanation, for the part picker and the stream editor.
    pub fn describe(&self) -> String {
        match self {
            Self::SourceDoesNotParse { message } => format!("the score does not parse: {message}"),
            Self::UnknownPart { abbreviation } => format!("no part [{abbreviation}] is declared"),
            Self::Ineligible { reason } => reason.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamDiagnosticSeverity {
    Error,
    Warning,
}

/// A problem in an edited stream, spanned in stream coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamDiagnostic {
    pub span: Span,
    pub message: String,
    pub severity: StreamDiagnosticSeverity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartStreamApplyResult {
    /// `None` while any error-severity diagnostic exists.
    pub source: Option<String>,
    pub diagnostics: Vec<StreamDiagnostic>,
    /// Where each measure starts in the *edited* stream (drives the measure markers in the UI).
    pub measure_starts: Vec<usize>,
    pub changed_measure_count: usize,
}

/// Every declared part, in declaration order, with the reason it can't be
/// stream-edited (if any).
pub fn list_streamable_parts(source: &str) -> Vec<StreamablePart> {
    let document = ScoreDocument::load(source);
    let declarations = match &document {
        Ok(document) => document.declarations.clone(),
        Err(_) => document::declarations_of(source),
    };
    declarations
        .iter()
        .enumerate()
        .map(|(index, declaration)| StreamablePart {
            abbreviation: declaration.abbreviation.clone(),
            display_name: declaration.display_name.clone(),
            kind: streamable_kind(declaration),
            ineligible_reason: document
                .as_ref()
                .map_err(Clone::clone)
                .and_then(|document| editable_part(document, index))
                .err()
                .map(|error| error.describe()),
        })
        .collect()
}

/// The stream of `abbreviation`'s measures, one measure chunk per measure.
pub fn extract_part_stream(
    source: &str,
    abbreviation: &str,
) -> Result<PartStream, PartStreamError> {
    let document = ScoreDocument::load(source)?;
    let index = find_part(&document, abbreviation)?;
    let part = editable_part(&document, index)?;
    Ok(part.grammar.join(&part.chunks))
}

/// Re-bars `stream` into `abbreviation`'s measures and rewrites the measures
/// that changed. Pure: call it on every edit to get diagnostics and measure
/// markers, and use `source` only when the user applies.
pub fn apply_part_stream(source: &str, abbreviation: &str, stream: &str) -> PartStreamApplyResult {
    apply::apply(source, abbreviation, stream)
}

fn streamable_kind(declaration: &PartDecl) -> StreamablePartKind {
    match (declaration.kind, &declaration.follow_target) {
        (PartKind::Lyrics { .. }, _) => StreamablePartKind::Lyrics,
        (_, Some(_)) => StreamablePartKind::Follow,
        (PartKind::Notes, None) => StreamablePartKind::Notes,
        (PartKind::Chords, None) => StreamablePartKind::Chords,
        (PartKind::Percussion, None) => StreamablePartKind::Percussion,
    }
}

fn find_part(document: &ScoreDocument, abbreviation: &str) -> Result<usize, PartStreamError> {
    document
        .declarations
        .iter()
        .position(|declaration| declaration.abbreviation == abbreviation)
        .ok_or_else(|| PartStreamError::UnknownPart {
            abbreviation: abbreviation.to_string(),
        })
}

/// Joins measure chunks into a stream, one measure per line. An empty chunk
/// (a lyric measure with no slots) adds no separator: it starts right where
/// the previous chunk ends.
fn join_chunks(chunks: &[String]) -> PartStream {
    let mut text = String::new();
    let mut measure_starts = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        if !text.is_empty() && !chunk.is_empty() {
            text.push('\n');
        }
        measure_starts.push(text.len());
        text.push_str(chunk);
    }
    PartStream {
        text,
        measure_starts,
    }
}
