//! A lyric part sung on several parts (`lyrics [S, A, T, B]`) puts each
//! syllable at the same time in every one of them. The syllables are paired
//! with the bottommost part's notes, so every other part must start a note
//! at each of those times, and may start no other note that would want a
//! syllable of its own. A note slurred into from the note before it carries
//! the syllable on, so it needs none.

use crate::ast::grouped::{GroupedPart, GroupedTrack, NoteEvent};
use crate::ast::parsed::PartKind;
use crate::error::RecoverableError;
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
    let times_of = |index: usize| match grouped_tracks.get(index) {
        Some(GroupedTrack::Timed(target)) => target_times(target, measure_index),
        None => None,
    };
    let paired = times_of(target_part_index)?;
    let syllable_times = &paired.may_take;
    let other = lyric_part
        .lyric_targets
        .iter()
        .filter_map(|&index| times_of(index))
        .find(|target| {
            !target
                .must_take
                .iter()
                .all(|&time| contains(syllable_times, time))
                || !syllable_times
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
    let previous = measure_index
        .checked_sub(1)
        .and_then(|previous| target.measures.get(previous))
        .map(|previous| previous.notes.events.as_slice());
    let entering_tied =
        previous.is_some_and(|events| measure_lyric_slots(events, false).exits_tied);
    let entering_slurred = previous
        .and_then(|events| events.last())
        .is_some_and(slurs_into_next);
    let events = &target.measures.get(measure_index)?.notes.events;
    let lyric_slots = measure_lyric_slots(events, entering_tied);
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
