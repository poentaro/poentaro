//! Which parts can be stream-edited, and the grammar each one's stream uses.

use itertools::Itertools;

use crate::ast::parsed::{PartDecl, PartKind};
use crate::error::Span;

use super::chord_lyrics::{
    chord_attacks_per_measure, extract_chord_lyric_chunks, join_chord_lyric_chunks,
    split_chord_lyric_stream, MeasureChordAttacks,
};
use super::diagnostics::error_at;
use super::document::ScoreDocument;
use super::extract::extract_timed_chunks;
use super::lyrics::{extract_lyric_chunks, lyric_slot_counts, split_lyric_stream};
use super::split::{split_timed_stream, SplitStream};
use super::timing::{rest_fill, TimedLineKind};
use super::{
    join_chunks, PartStream, PartStreamError, StreamDiagnosticSeverity, FOLLOW_PLACEHOLDER,
    NO_LYRICS_MEASURE,
};

/// A notes / chords / percussion stream; a follow part's may also hold `@`.
#[derive(Debug, Clone, Copy)]
pub(super) struct TimedGrammar {
    pub(super) kind: TimedLineKind,
    pub(super) is_follow: bool,
}

pub(super) enum StreamGrammar {
    Timed {
        grammar: TimedGrammar,
    },
    /// A lyric part over a notes part: measure `m` takes `slot_counts[m]` syllables.
    Lyrics {
        slot_counts: Vec<usize>,
    },
    /// A lyric part over a chords part: measures are separated by `|`, and
    /// measure `m`'s bracketed items are tallied against `attacks[m]`.
    ChordLyrics {
        attacks: Vec<MeasureChordAttacks>,
    },
}

impl StreamGrammar {
    pub(super) fn split(&self, stream: &str, document: &ScoreDocument) -> SplitStream {
        match self {
            Self::Timed { grammar } => {
                let mut split = split_timed_stream(stream, *grammar, &document.capacities());
                let follows_past_the_end = split
                    .chunks
                    .iter()
                    .zip(&split.measure_starts)
                    .skip(document.measures.len())
                    .filter(|(chunk, _)| chunk.as_str() == FOLLOW_PLACEHOLDER)
                    .map(|(_, &start)| {
                        error_at(
                            Span::new(start, start + FOLLOW_PLACEHOLDER.len()),
                            "`@` can't add a measure past the end of the score",
                        )
                    })
                    .collect_vec();
                split.blocks_rebarring |= !follows_past_the_end.is_empty();
                split.diagnostics.extend(follows_past_the_end);
                split
            }
            Self::Lyrics { slot_counts } => split_lyric_stream(stream, slot_counts),
            Self::ChordLyrics { attacks } => split_chord_lyric_stream(stream, attacks),
        }
    }

    pub(super) fn join(&self, chunks: &[String]) -> PartStream {
        match self {
            Self::ChordLyrics { .. } => join_chord_lyric_chunks(chunks),
            Self::Timed { .. } | Self::Lyrics { .. } => join_chunks(chunks),
        }
    }

    pub(super) fn is_follow(&self) -> bool {
        matches!(self, Self::Timed { grammar } if grammar.is_follow)
    }

    /// What fills a measure the part must still write but has nothing for.
    pub(super) fn empty_measure(&self, capacity: u32) -> String {
        match self {
            Self::Timed { .. } => rest_fill(capacity),
            Self::Lyrics { .. } | Self::ChordLyrics { .. } => NO_LYRICS_MEASURE.to_string(),
        }
    }
}

/// A part's original measure chunks.
pub(super) struct ExtractedChunks {
    pub(super) chunks: Vec<String>,
    /// The first measure whose written beats don't match its time signature.
    /// Re-barring the stream moves notes across its barline, so the stream
    /// only round-trips up to it.
    pub(super) first_misbarred_measure: Option<usize>,
}

impl ExtractedChunks {
    fn fitting(chunks: Vec<String>) -> Self {
        Self {
            chunks,
            first_misbarred_measure: None,
        }
    }
}

/// A part that passed every eligibility check, with its original chunks.
pub(super) struct EditablePart {
    pub(super) index: usize,
    pub(super) grammar: StreamGrammar,
    pub(super) chunks: Vec<String>,
}

pub(super) fn ineligible(reason: impl Into<String>) -> PartStreamError {
    PartStreamError::Ineligible {
        reason: reason.into(),
    }
}

/// Runs every eligibility check on the part at `index`, including a
/// self-check that its own extracted stream splits back into exactly the
/// same measure chunks without errors — the guarantee that applying an
/// untouched stream never changes the source. A misbarred measure is re-barred
/// (and every measure after it shifts), so the check stops right before it;
/// errors from there on show up in the stream editor instead.
pub(super) fn editable_part(
    document: &ScoreDocument,
    index: usize,
) -> Result<EditablePart, PartStreamError> {
    let declaration = document
        .declarations
        .get(index)
        .ok_or_else(|| ineligible("no such part"))?;
    let grammar = stream_grammar(document, declaration)?;
    let ExtractedChunks {
        chunks,
        first_misbarred_measure,
    } = match &grammar {
        StreamGrammar::Timed { grammar } => extract_timed_chunks(document, index, *grammar),
        StreamGrammar::Lyrics { slot_counts } => {
            extract_lyric_chunks(document, index, slot_counts).map(ExtractedChunks::fitting)
        }
        StreamGrammar::ChordLyrics { .. } => Ok(ExtractedChunks::fitting(
            extract_chord_lyric_chunks(document, index),
        )),
    }
    .map_err(ineligible)?;
    let stream = grammar.join(&chunks);
    let resplit = grammar.split(&stream.text, document);
    let checked_text_end = first_misbarred_measure
        .and_then(|measure| stream.measure_starts.get(measure))
        .copied()
        .unwrap_or(stream.text.len());
    if let Some(error) = resplit.diagnostics.iter().find(|diagnostic| {
        diagnostic.severity == StreamDiagnosticSeverity::Error
            && diagnostic.span.start < checked_text_end
    }) {
        return Err(ineligible(format!(
            "its stream would not parse: {}",
            error.message
        )));
    }
    let checked_measure_count =
        first_misbarred_measure.unwrap_or(chunks.len().max(resplit.chunks.len()));
    if let Some(measure) = (0..checked_measure_count)
        .find(|&measure| chunks.get(measure) != resplit.chunks.get(measure))
    {
        return Err(ineligible(format!(
            "measure {} would not keep its content when re-barred",
            measure + 1
        )));
    }
    Ok(EditablePart {
        index,
        grammar,
        chunks,
    })
}

fn stream_grammar(
    document: &ScoreDocument,
    declaration: &PartDecl,
) -> Result<StreamGrammar, PartStreamError> {
    let PartKind::Lyrics { target_part_index } = declaration.kind else {
        let kind = TimedLineKind::of(declaration.kind)
            .ok_or_else(|| ineligible("this kind of part cannot be edited as a stream"))?;
        return Ok(StreamGrammar::Timed {
            grammar: TimedGrammar {
                kind,
                is_follow: declaration.follow_target.is_some(),
            },
        });
    };
    let target_kind = document
        .declarations
        .get(target_part_index)
        .map(|target| target.kind);
    match target_kind {
        Some(PartKind::Notes) => {
            lyric_slot_counts(document, &declaration.lyric_targets, target_part_index)
                .map(|slot_counts| StreamGrammar::Lyrics { slot_counts })
                .map_err(ineligible)
        }
        Some(PartKind::Chords) => chord_attacks_per_measure(document, target_part_index)
            .map(|attacks| StreamGrammar::ChordLyrics { attacks })
            .map_err(ineligible),
        _ => Err(ineligible(
            "only lyrics under a notes or chords part can be edited as a stream",
        )),
    }
}
