//! A lyric part sung on several parts (`lyrics [S, A, T, B]`) puts each
//! syllable at the same time in every one of them. The syllables are paired
//! with one part's notes (see `lyric_paired_target`), so every other part must start a note
//! at each word's time, and may start no note at a time no syllable is sung.
//! A held `-` carries the word before on, so a part may hold its note through
//! it; likewise a note slurred into needs no syllable of its own. A part
//! resting all measure sings none of it, so isn't checked.

use crate::ast::grouped::{GroupedPart, GroupedTrack, NoteEvent};
use crate::ast::parsed::PartKind;
use crate::error::RecoverableError;
use crate::lyric_paired_target::{enters_tied, paired_target_index, target_part};
use crate::lyric_slots::{measure_lyric_slots, MeasureTime};

struct TargetTimes {
    name: String,
    /// When the part's notes may take a syllable: every note that isn't the
    /// continuation of a tie.
    may_take: Vec<MeasureTime>,
    /// When they must: those notes, less the ones a slur carries a syllable on to.
    must_take: Vec<MeasureTime>,
}

/// An error when `lyric_part`'s targets don't take their syllables at the
/// same times at `measure_index`; `None` when they agree or the measure has
/// no syllables.
pub(crate) fn lyric_targets_rhythm_error(
    grouped_tracks: &[GroupedTrack],
    lyric_part: &GroupedPart,
    measure_index: usize,
) -> Option<RecoverableError> {
    let measure = lyric_part.measures.get(measure_index)?;
    let PartKind::Lyrics { target_part_index } = lyric_part.kind else {
        return None;
    };
    if lyric_part.lyric_targets.len() < 2 || measure.paired_lyrics.is_empty() {
        return None;
    }
    let times_of = |index: usize| target_times(target_part(grouped_tracks, index)?, measure_index);
    let paired = times_of(paired_target_index(
        grouped_tracks,
        &lyric_part.lyric_targets,
        target_part_index,
        measure_index,
    ))?;
    let syllable_times = &paired.may_take;
    let word_times = syllable_times
        .iter()
        .zip(&measure.paired_lyrics)
        .filter(|(_, syllable)| syllable.text != "-")
        .map(|(&time, _)| time)
        .collect::<Vec<_>>();
    let other = lyric_part
        .lyric_targets
        .iter()
        .filter_map(|&index| times_of(index))
        .filter(|target| !target.may_take.is_empty())
        .find(|target| {
            !target
                .must_take
                .iter()
                .all(|&time| contains(syllable_times, time))
                || !word_times
                    .iter()
                    .all(|&time| contains(&target.may_take, time))
        })?;
    Some(RecoverableError::lyrics_targets_differ(
        measure.source_span,
        &paired.name,
        &other.name,
    ))
}

fn contains(times: &[MeasureTime], time: MeasureTime) -> bool {
    times.iter().any(|&other| other.same_as(time))
}

fn target_times(target: &GroupedPart, measure_index: usize) -> Option<TargetTimes> {
    let entering_slurred = measure_index
        .checked_sub(1)
        .and_then(|previous| target.measures.get(previous))
        .and_then(|previous| previous.notes.events.last())
        .is_some_and(slurs_into_next);
    let events = &target.measures.get(measure_index)?.notes.events;
    let lyric_slots = measure_lyric_slots(events, enters_tied(target, measure_index));
    let slurred_into = slurred_into(events, entering_slurred);
    let time = |column| MeasureTime {
        column,
        length: lyric_slots.end_column,
    };
    Some(TargetTimes {
        name: target.name.clone().unwrap_or_default(),
        may_take: lyric_slots
            .slots
            .iter()
            .map(|slot| time(slot.column))
            .collect(),
        must_take: lyric_slots
            .slots
            .iter()
            .filter(|slot| !slurred_into.get(slot.event_index).copied().unwrap_or(false))
            .map(|slot| time(slot.column))
            .collect(),
    })
}

/// For each event, whether the note before it is slurred into it.
fn slurred_into(events: &[NoteEvent], entering_slurred: bool) -> Vec<bool> {
    events
        .iter()
        .scan(entering_slurred, |slurred, event| {
            let into = *slurred;
            *slurred = slurs_into_next(event);
            Some(into)
        })
        .collect()
}

fn slurs_into_next(event: &NoteEvent) -> bool {
    matches!(event, NoteEvent::Note(note) if note.slur)
}

#[cfg(test)]
#[path = "lyric_targets_rhythm_tests.rs"]
mod tests;
