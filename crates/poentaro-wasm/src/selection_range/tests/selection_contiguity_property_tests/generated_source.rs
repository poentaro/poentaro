//! Writes a [`GeneratedScore`] as `.poentaro` source, recording when every
//! token sounds.

use std::collections::HashMap;

use itertools::Itertools;
use proptest::sample::Index;

use super::generated_score::{
    GeneratedEvent, GeneratedLyricPart, GeneratedMeasure, GeneratedNotesPart, GeneratedScore,
    Timing, HELD_SYLLABLE, MEASURE_LENGTH, NOTES_PART_NAMES, SYLLABLES,
};

/// A tie sits after a shortening suffix (`5_~`) but before extension dashes
/// (`5~---`), since a `_` glued after `~` would be a repeat atom instead.
fn event_token(event: &GeneratedEvent) -> String {
    let pitch = event.pitch.unwrap_or(0);
    let tie = if event.tied_to_next { "~" } else { "" };
    match event.duration {
        2 => format!("{pitch}_{tie}"),
        8 => format!("{pitch}{tie}-"),
        12 => format!("{pitch}{tie}--"),
        16 => format!("{pitch}{tie}---"),
        _ => format!("{pitch}{tie}"),
    }
}

fn lyric_part_name(index: usize) -> String {
    format!("L{index}")
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

/// The notes of one measure that take a syllable (a note that isn't the
/// continuation of a tie), each timed from its onset to the end of its tie
/// chain within the measure.
fn lyric_slots(
    measure: &GeneratedMeasure,
    entering_tied: bool,
    measure_onset: usize,
) -> Vec<Timing> {
    let continuations = measure.events.iter().scan(entering_tied, |tied, event| {
        let continuation = *tied;
        *tied = event.tied_to_next;
        Some(continuation)
    });
    let events = measure
        .events
        .iter()
        .zip(onsets(&measure.events, measure_onset))
        .zip(continuations)
        .collect_vec();
    events
        .iter()
        .enumerate()
        .filter(|(_, ((event, _), continuation))| event.pitch.is_some() && !continuation)
        .map(|(index, ((_, onset), _))| {
            let chain_end = events[index..]
                .iter()
                .take_while_inclusive(|((event, _), _)| event.tied_to_next)
                .last()
                .map_or(*onset, |((event, onset), _)| onset + event.duration);
            Timing {
                onset: *onset,
                end: chain_end,
            }
        })
        .collect()
}

/// The generated source, plus the timing of every token keyed by its start
/// byte — what the test checks the real spans' cells against.
pub(super) struct GeneratedSource {
    pub(super) text: String,
    pub(super) timing_by_start_byte: HashMap<usize, Timing>,
}

/// One token of a score line, and when it sounds (`None` for a token no
/// span points at, such as `_`).
struct Token {
    text: String,
    timing: Option<Timing>,
}

/// One score line of a measure: `[name] tokens…`.
struct Line {
    name: String,
    tokens: Vec<Token>,
}

fn onsets(events: &[GeneratedEvent], measure_onset: usize) -> impl Iterator<Item = usize> + '_ {
    events.iter().scan(measure_onset, |onset, event| {
        let start = *onset;
        *onset += event.duration;
        Some(start)
    })
}

/// The `parts` section: each lyric part right after its target, unless it
/// is declared last.
fn parts_section(score: &GeneratedScore) -> String {
    let notes_part_line = |index: usize| format!("\"{}\" : notes", NOTES_PART_NAMES[index]);
    let lyric_part_line = |index: usize, part: &GeneratedLyricPart| {
        format!(
            "\"{}\" : lyrics [{}]",
            lyric_part_name(index),
            NOTES_PART_NAMES[part.target]
        )
    };
    (0..score.notes_parts.len())
        .flat_map(|notes_index| {
            std::iter::once(notes_part_line(notes_index)).chain(
                score
                    .lyric_parts
                    .iter()
                    .enumerate()
                    .filter(move |(_, part)| part.target == notes_index && !part.declared_last)
                    .map(|(index, part)| lyric_part_line(index, part)),
            )
        })
        .chain(
            score
                .lyric_parts
                .iter()
                .enumerate()
                .filter(|(_, part)| part.declared_last)
                .map(|(index, part)| lyric_part_line(index, part)),
        )
        .join(", ")
}

fn notes_line(part: &GeneratedNotesPart, index: usize, measure_index: usize) -> Line {
    let events = &part.measures[measure_index].events;
    Line {
        name: NOTES_PART_NAMES[index].to_string(),
        tokens: events
            .iter()
            .zip(onsets(events, measure_index * MEASURE_LENGTH))
            .map(|(event, onset)| Token {
                text: event_token(event),
                timing: Some(Timing {
                    onset,
                    end: onset + event.duration,
                }),
            })
            .collect(),
    }
}

/// One syllable per lyric slot of the target's measure, drawn from `picks`,
/// or `_` when the measure has no slots.
fn lyric_line<'a>(
    score: &GeneratedScore,
    index: usize,
    measure_index: usize,
    picks: &mut impl Iterator<Item = &'a Index>,
) -> Line {
    let target = &score.notes_parts[score.lyric_parts[index].target];
    let entering_tied = measure_index
        .checked_sub(1)
        .and_then(|previous| target.measures[previous].events.last())
        .is_some_and(|event| event.tied_to_next);
    let slots = lyric_slots(
        &target.measures[measure_index],
        entering_tied,
        measure_index * MEASURE_LENGTH,
    );
    let tokens = if slots.is_empty() {
        vec![Token {
            text: "_".to_string(),
            timing: None,
        }]
    } else {
        slots
            .into_iter()
            .enumerate()
            .map(|(slot_index, timing)| {
                let pick = picks.next().map_or("la", |pick| *pick.get(&SYLLABLES));
                let syllable = if slot_index == 0 && pick == HELD_SYLLABLE {
                    "la"
                } else {
                    pick
                };
                Token {
                    text: syllable.to_string(),
                    timing: Some(timing),
                }
            })
            .collect()
    };
    Line {
        name: lyric_part_name(index),
        tokens,
    }
}

pub(super) fn to_source(score: &GeneratedScore) -> GeneratedSource {
    let mut text = format!(
        "metadata {{ merge duplicate measures across parts : {}, hide resting parts : {} }},\n\n\
         parts [{}],\n\n\
         score \"\"\"\n",
        yes_no(score.merge_duplicate_measures),
        yes_no(score.hide_resting_parts),
        parts_section(score),
    );
    let mut timing_by_start_byte = HashMap::new();
    let mut syllable_picks = score
        .lyric_parts
        .iter()
        .map(|part| part.syllable_picks.iter())
        .collect_vec();
    let measure_count = score
        .notes_parts
        .first()
        .map_or(0, |part| part.measures.len());
    for measure_index in 0..measure_count {
        let notes_lines = score
            .notes_parts
            .iter()
            .enumerate()
            .map(|(index, part)| notes_line(part, index, measure_index));
        let lyric_lines = syllable_picks
            .iter_mut()
            .enumerate()
            .map(|(index, picks)| lyric_line(score, index, measure_index, picks))
            .collect_vec();
        let mut lines = notes_lines.chain(lyric_lines).collect_vec();
        if score.reverse_lines {
            lines.reverse();
        }
        if measure_index > 0 {
            text.push('\n');
        }
        for line in lines {
            text.push_str(&format!("[{}]", line.name));
            for token in line.tokens {
                text.push(' ');
                if let Some(timing) = token.timing {
                    timing_by_start_byte.insert(text.len(), timing);
                }
                text.push_str(&token.text);
            }
            text.push('\n');
        }
    }
    text.push_str("\"\"\",\n");
    GeneratedSource {
        text,
        timing_by_start_byte,
    }
}
