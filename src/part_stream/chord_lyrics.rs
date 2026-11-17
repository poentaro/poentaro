//! Lyric parts bound to a chords part: their unbracketed syllables flow
//! freely between the bracketed items paired with chord attacks, so nothing
//! in the target decides how many syllables a measure takes. Their stream
//! marks every measure boundary with `|` instead.

use itertools::Itertools;

use crate::ast::parsed::{ScoreEvent, Syllable};
use crate::chord_lyric_placement::chord_lyric_tally_error;
use crate::error::{RecoverableError, Span};
use crate::parser::lyrics::bracketed::tokenize_bracketed_lyrics;
use crate::parser::score::interleaved_parser::padded_measure_events;
use crate::parser::score::timed_parser::GroupStack;
use crate::parser::score::token_parser::parse_chord_line;

use super::diagnostics::{
    error_at, forbidden_character_errors, has_error, warning_at, AllowedMarkers,
};
use super::document::ScoreDocument;
use super::split::SplitStream;
use super::timing::normalize_whitespace;
use super::{PartStream, StreamDiagnostic, NO_LYRICS_MEASURE};

const MEASURE_SEPARATOR: char = '|';

/// What one measure's bracketed items are tallied against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MeasureChordAttacks {
    count: usize,
    first_on_beat_one: bool,
}

/// The chord attacks of every measure of the chords part at `target_index`.
pub(super) fn chord_attacks_per_measure(
    document: &ScoreDocument,
    target_index: usize,
) -> Result<Vec<MeasureChordAttacks>, String> {
    let mut stack = GroupStack::default();
    document
        .measures
        .iter()
        .enumerate()
        .map(|(index, measure)| {
            let content = measure
                .resolved_contents
                .get(target_index)
                .map_or("", String::as_str);
            let parsed = parse_chord_line(content, 0, &mut stack).map_err(|error| {
                format!(
                    "its target's measure {} does not parse: {}",
                    index + 1,
                    error.message()
                )
            })?;
            let events = padded_measure_events(parsed.events, measure.capacity);
            Ok(MeasureChordAttacks {
                count: events
                    .iter()
                    .filter(|event| matches!(event.value, ScoreEvent::Chord(_)))
                    .count(),
                first_on_beat_one: events
                    .iter()
                    .find(|event| matches!(event.value, ScoreEvent::Chord(_) | ScoreEvent::Rest(_)))
                    .is_some_and(|event| matches!(event.value, ScoreEvent::Chord(_))),
            })
        })
        .collect()
}

/// One measure chunk per measure of the lyric part at `part_index`: the
/// written line, or `_` for a measure it doesn't write.
pub(super) fn extract_chord_lyric_chunks(
    document: &ScoreDocument,
    part_index: usize,
) -> Vec<String> {
    document
        .measures
        .iter()
        .map(|measure| {
            measure
                .written_line(part_index)
                .map_or(NO_LYRICS_MEASURE.to_string(), |line| {
                    normalize_whitespace(&line.content)
                })
        })
        .collect()
}

/// Joins measure chunks into a stream, one measure per line, each but the
/// last ended by `|`.
pub(super) fn join_chord_lyric_chunks(chunks: &[String]) -> PartStream {
    let separator = format!(" {MEASURE_SEPARATOR}\n");
    let measure_starts = chunks
        .iter()
        .scan(0, |offset, chunk| {
            let start = *offset;
            *offset += chunk.len() + separator.len();
            Some(start)
        })
        .collect();
    PartStream {
        text: chunks.join(&separator),
        measure_starts,
    }
}

/// One measure's text between two `|`.
struct MeasurePiece<'a> {
    /// Whitespace-trimmed.
    text: &'a str,
    /// Where `text` starts in the stream.
    start: usize,
    /// Where the untrimmed piece ends (at its closing `|`, or the stream's end).
    end: usize,
}

impl MeasurePiece<'_> {
    fn span(&self) -> Span {
        Span::new(self.start, self.start + self.text.len())
    }
}

/// Cuts the stream at every `|`. A trailing `|` doesn't open another measure.
fn measure_pieces(stream: &str) -> Vec<MeasurePiece<'_>> {
    let mut pieces = stream
        .split(MEASURE_SEPARATOR)
        .scan(0, |offset, raw| {
            let raw_start = *offset;
            *offset += raw.len() + MEASURE_SEPARATOR.len_utf8();
            let leading = raw.len() - raw.trim_start().len();
            Some(MeasurePiece {
                text: raw.trim(),
                start: raw_start + leading,
                end: raw_start + raw.len(),
            })
        })
        .collect_vec();
    if pieces.last().is_some_and(|piece| piece.text.is_empty()) {
        pieces.pop();
    }
    pieces
}

/// Cuts a chord-bound lyric stream into measures at its `|`s and checks each
/// measure's brackets against its chord attacks. A measure past the last one
/// is appended to the score, with no chord attacks to bracket.
pub(super) fn split_chord_lyric_stream(
    stream: &str,
    attacks: &[MeasureChordAttacks],
) -> SplitStream {
    let mut diagnostics = forbidden_character_errors(
        stream,
        AllowedMarkers {
            placeholder: false,
            barlines: true,
        },
    );
    let pieces = measure_pieces(stream);
    diagnostics.extend(pieces.iter().enumerate().flat_map(|(index, piece)| {
        measure_diagnostics(piece, attacks.get(index).copied().unwrap_or(PAST_THE_END))
    }));
    if pieces.len() < attacks.len() {
        let span = pieces
            .last()
            .map_or(Span::new(0, stream.len()), MeasurePiece::span);
        diagnostics.push(warning_at(
            span,
            format!(
                "the stream ends at measure {} of {}; the measures after it will have no lyrics",
                pieces.len(),
                attacks.len()
            ),
        ));
    }
    SplitStream {
        chunks: pieces
            .iter()
            .map(|piece| normalize_whitespace(piece.text))
            .collect(),
        measure_starts: pieces.iter().map(|piece| piece.start).collect(),
        blocks_rebarring: has_error(&diagnostics),
        diagnostics,
        dropped_placeholders: None,
    }
}

/// A measure appended past the end of the score: its chords part writes
/// nothing there, so the measure is all rests.
const PAST_THE_END: MeasureChordAttacks = MeasureChordAttacks {
    count: 0,
    first_on_beat_one: false,
};

fn measure_diagnostics(
    piece: &MeasurePiece,
    attacks: MeasureChordAttacks,
) -> Vec<StreamDiagnostic> {
    if piece.text.is_empty() {
        return vec![error_at(
            Span::new(piece.end, piece.end + MEASURE_SEPARATOR.len_utf8()),
            "this measure is empty; write `_` for a measure without lyrics",
        )];
    }
    if piece.text == NO_LYRICS_MEASURE {
        return Vec::new();
    }
    let lyrics = tokenize_bracketed_lyrics(piece.text, piece.start);
    let errors = if lyrics.errors.is_empty() {
        tally_error(&lyrics.syllables, attacks, piece.span())
            .into_iter()
            .collect()
    } else {
        lyrics.errors
    };
    errors
        .iter()
        .map(|error| error_at(error.span, error.message()))
        .collect()
}

/// The same tally the grouper runs on a lyric line under a chords part.
fn tally_error(
    syllables: &[Syllable],
    attacks: MeasureChordAttacks,
    span: Span,
) -> Option<RecoverableError> {
    chord_lyric_tally_error(syllables, attacks.count, attacks.first_on_beat_one, span)
}

#[cfg(test)]
#[path = "chord_lyrics_tests.rs"]
mod tests;
