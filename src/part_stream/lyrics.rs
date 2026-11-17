//! Lyric parts bound to a notes part: measures are cut by the target's
//! **lyric slots** (its notes that aren't the continuation of a tie), never
//! by guessing from the lyric text. Past the last slot, each line is a measure.

use itertools::Itertools;

use crate::ast::parsed::{ScoreEvent, Syllable};
use crate::error::Span;
use crate::parser::score::interleaved_parser::padded_measure_events;
use crate::parser::score::timed_parser::GroupStack;
use crate::parser::score::token_parser::parse_notes_line;
use crate::utils::{count_lyric_slots_in_events, tokenize_lyrics, LyricTieState};

use super::diagnostics::{forbidden_character_errors, has_error, warning_at, AllowedMarkers};
use super::document::ScoreDocument;
use super::split::SplitStream;
use super::timing::normalize_whitespace;
use super::NO_LYRICS_MEASURE;

/// How many syllables each measure of a lyric part sung on the notes parts
/// at `target_indices` takes: the lyric slots of the part it pairs with
/// there, the bottommost with a note, else `bottommost` (see
/// `lyric_paired_target`).
pub(super) fn lyric_slot_counts(
    document: &ScoreDocument,
    target_indices: &[usize],
    bottommost: usize,
) -> Result<Vec<usize>, String> {
    let targets = target_indices
        .iter()
        .chain(std::iter::once(&bottommost))
        .unique()
        .map(|&index| {
            Ok(TargetSlots {
                index,
                measures: measure_slots(document, index)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok((0..document.measures.len())
        .map(|measure_index| {
            let slots_of = |target: &TargetSlots| target.measures.get(measure_index).copied();
            targets
                .iter()
                .filter(|target| slots_of(target).is_some_and(|slots| slots.has_note))
                .max_by_key(|target| target.index)
                .or_else(|| targets.iter().find(|target| target.index == bottommost))
                .and_then(slots_of)
                .map_or(0, |slots| slots.count)
        })
        .collect())
}

struct TargetSlots {
    index: usize,
    measures: Vec<MeasureSlots>,
}

#[derive(Clone, Copy)]
struct MeasureSlots {
    count: usize,
    has_note: bool,
}

/// Each measure's lyric slots of the notes part at `target_index`, with ties
/// carried across barlines the way the parser carries them.
fn measure_slots(
    document: &ScoreDocument,
    target_index: usize,
) -> Result<Vec<MeasureSlots>, String> {
    let mut stack = GroupStack::default();
    let mut tie_state = LyricTieState::default();
    document
        .measures
        .iter()
        .enumerate()
        .map(|(index, measure)| {
            let content = measure
                .resolved_contents
                .get(target_index)
                .map_or("", String::as_str);
            if content == NO_LYRICS_MEASURE {
                return Ok(MeasureSlots {
                    count: 0,
                    has_note: false,
                });
            }
            let parsed = parse_notes_line(content, 0, &mut stack).map_err(|error| {
                format!(
                    "its target's measure {} does not parse: {}",
                    index + 1,
                    error.message()
                )
            })?;
            let events = padded_measure_events(parsed.events, measure.capacity);
            Ok(MeasureSlots {
                count: count_lyric_slots_in_events(&events, &mut tie_state) as usize,
                has_note: events
                    .iter()
                    .any(|event| matches!(event.value, ScoreEvent::Note(_))),
            })
        })
        .collect()
}

/// One measure chunk per measure of the lyric part at `part_index`: nothing
/// for a measure whose target has no lyric slots, `_` for a measure without
/// lyrics, otherwise the written syllables. `Err` explains why the part
/// can't be stream-edited.
pub(super) fn extract_lyric_chunks(
    document: &ScoreDocument,
    part_index: usize,
    slot_counts: &[usize],
) -> Result<Vec<String>, String> {
    document
        .measures
        .iter()
        .zip(slot_counts)
        .enumerate()
        .map(|(index, (measure, &slot_count))| {
            let content = measure
                .written_line(part_index)
                .map_or(NO_LYRICS_MEASURE, |line| line.content.as_str());
            let syllable_count = if content == NO_LYRICS_MEASURE {
                0
            } else {
                tokenize_lyrics(content, 0).len()
            };
            match (slot_count, syllable_count) {
                (0, 0) => Ok(String::new()),
                (_, 0) if content == NO_LYRICS_MEASURE => Ok(NO_LYRICS_MEASURE.to_string()),
                (slots, syllables) if slots == syllables => Ok(normalize_whitespace(content)),
                (slots, syllables) => Err(format!(
                    "measure {} has {syllables} syllables for {slots} notes",
                    index + 1
                )),
            }
        })
        .collect()
}

/// Cuts a lyric stream into measures by `slot_counts`. Syllables left after
/// the last slot are appended as new measures, one per line (see
/// [`appended_measures`]), unless they are all `_`: placeholders pushed out
/// by syllables typed before them, which are dropped.
pub(super) fn split_lyric_stream(stream: &str, slot_counts: &[usize]) -> SplitStream {
    let mut diagnostics = forbidden_character_errors(stream, AllowedMarkers::default());
    let syllables = tokenize_lyrics(stream, 0);
    let mut chunks = Vec::new();
    let mut measure_starts = Vec::new();
    let mut consumed = 0;
    let mut previous_end = 0;
    for (index, &slot_count) in slot_counts.iter().enumerate() {
        let remaining = syllables.get(consumed..).unwrap_or_default();
        let Some(first) = remaining.first() else {
            if slot_count > 0 {
                break;
            }
            chunks.push(String::new());
            measure_starts.push(previous_end);
            continue;
        };
        let taken_count = match slot_count {
            0 => 0,
            _ if first.text == NO_LYRICS_MEASURE => 1,
            _ => slot_count.min(remaining.len()),
        };
        let taken = remaining.get(..taken_count).unwrap_or_default();
        let span = covering_span(taken).unwrap_or(Span::new(previous_end, previous_end));
        if !taken.is_empty() && first.text != NO_LYRICS_MEASURE && taken.len() < slot_count {
            diagnostics.push(warning_at(
                span,
                format!(
                    "measure {} has {} of its {slot_count} syllables; it will show a lyrics underflow warning",
                    index + 1,
                    taken.len()
                ),
            ));
        }
        chunks.push(normalize_whitespace(
            stream.get(span.start..span.end).unwrap_or_default(),
        ));
        measure_starts.push(span.start);
        consumed += taken.len();
        previous_end = span.end;
    }
    let leftovers = syllables.get(consumed..).unwrap_or_default();
    let dropped_placeholders = if leftovers
        .iter()
        .all(|syllable| syllable.text == NO_LYRICS_MEASURE)
    {
        leftovers
            .last()
            .map(|last| Span::new(previous_end, last.span.end))
    } else {
        for measure in appended_measures(stream, leftovers) {
            let span = covering_span(measure).unwrap_or(Span::new(previous_end, previous_end));
            chunks.push(normalize_whitespace(
                stream.get(span.start..span.end).unwrap_or_default(),
            ));
            measure_starts.push(span.start);
        }
        None
    };
    SplitStream {
        chunks,
        measure_starts,
        blocks_rebarring: has_error(&diagnostics),
        diagnostics,
        dropped_placeholders,
    }
}

/// Syllables past the last slot have no notes to cut them by, so each line
/// of them is one new measure, and `_` is a whole measure on its own.
fn appended_measures<'a>(stream: &str, leftovers: &'a [Syllable]) -> Vec<&'a [Syllable]> {
    let starts = std::iter::once(0)
        .chain(
            leftovers
                .iter()
                .tuple_windows()
                .positions(|(previous, current)| {
                    previous.text == NO_LYRICS_MEASURE
                        || current.text == NO_LYRICS_MEASURE
                        || stream
                            .get(previous.span.end..current.span.start)
                            .is_some_and(|between| between.contains('\n'))
                })
                .map(|index| index + 1),
        )
        .collect_vec();
    starts
        .iter()
        .zip(starts.iter().skip(1).copied().chain([leftovers.len()]))
        .filter_map(|(&start, end)| leftovers.get(start..end))
        .filter(|measure| !measure.is_empty())
        .collect()
}

fn covering_span(syllables: &[Syllable]) -> Option<Span> {
    Some(Span::new(
        syllables.first()?.span.start,
        syllables.last()?.span.end,
    ))
}

#[cfg(test)]
#[path = "lyrics_tests.rs"]
mod tests;
