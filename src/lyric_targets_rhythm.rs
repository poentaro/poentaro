//! A lyric part sung on several parts (`lyrics [S, A, T, B]`) puts each
//! syllable on the same note of every one of them, so they must offer the
//! same number of syllable slots in every measure.

use crate::ast::grouped::{GroupedPart, GroupedTrack};
use crate::error::RecoverableError;
use crate::lyric_slots::measure_lyric_slots;

struct TargetSlots {
    name: String,
    slots: usize,
}

/// An error when `lyric_part`'s targets differ in how many syllables they
/// take at `measure_index`; `None` when they agree or the measure has no
/// syllables.
pub(crate) fn lyric_targets_rhythm_error(
    grouped_tracks: &[GroupedTrack],
    lyric_part: &GroupedPart,
    measure_index: usize,
) -> Option<RecoverableError> {
    let measure = lyric_part.measures.get(measure_index)?;
    if lyric_part.lyric_targets.len() < 2 || measure.paired_lyrics.is_empty() {
        return None;
    }
    let targets = lyric_part
        .lyric_targets
        .iter()
        .filter_map(|&index| match grouped_tracks.get(index) {
            Some(GroupedTrack::Timed(target)) => target_slots(target, measure_index),
            None => None,
        })
        .collect::<Vec<_>>();
    let first = targets.first()?;
    let other = targets.iter().find(|target| target.slots != first.slots)?;
    Some(RecoverableError::lyrics_targets_differ(
        measure.source_span,
        &first.name,
        first.slots,
        &other.name,
        other.slots,
    ))
}

fn target_slots(target: &GroupedPart, measure_index: usize) -> Option<TargetSlots> {
    let entering_tied = measure_index
        .checked_sub(1)
        .and_then(|previous| target.measures.get(previous))
        .is_some_and(|previous| measure_lyric_slots(&previous.notes.events, false).exits_tied);
    let events = &target.measures.get(measure_index)?.notes.events;
    Some(TargetSlots {
        name: target.name.clone().unwrap_or_default(),
        slots: measure_lyric_slots(events, entering_tied).slots.len(),
    })
}

#[cfg(test)]
#[path = "lyric_targets_rhythm_tests.rs"]
mod tests;
