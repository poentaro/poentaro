//! Which of a lyric part's targets (`lyrics [Vo, A2]`) a measure's syllables
//! pair with: the bottommost one with a note in that measure, so a part that
//! rests while another sings hands the words to the one singing. A measure
//! where none has a note pairs with the bottommost target, under which the
//! lyric row is drawn.

use crate::ast::grouped::{GroupedPart, GroupedTrack, NoteEvent};
use crate::lyric_slots::measure_lyric_slots;

/// The declaration index of the part `lyric_targets` pair with at
/// `measure_index`; `target_part_index` (the bottommost) when none has a note.
pub(crate) fn paired_target_index(
    grouped_tracks: &[GroupedTrack],
    lyric_targets: &[usize],
    target_part_index: usize,
    measure_index: usize,
) -> usize {
    lyric_targets
        .iter()
        .copied()
        .filter(|&index| {
            target_part(grouped_tracks, index).is_some_and(|target| {
                target.measures.get(measure_index).is_some_and(|measure| {
                    measure
                        .notes
                        .events
                        .iter()
                        .any(|event| matches!(event, NoteEvent::Note(_)))
                })
            })
        })
        .max()
        .unwrap_or(target_part_index)
}

pub(crate) fn target_part(grouped_tracks: &[GroupedTrack], index: usize) -> Option<&GroupedPart> {
    match grouped_tracks.get(index) {
        Some(GroupedTrack::Timed(target)) => Some(target),
        None => None,
    }
}

/// Whether `target`'s last note before `measure_index` is tied into it.
pub(crate) fn enters_tied(target: &GroupedPart, measure_index: usize) -> bool {
    measure_index
        .checked_sub(1)
        .and_then(|previous| target.measures.get(previous))
        .is_some_and(|previous| measure_lyric_slots(&previous.notes.events, false).exits_tied)
}
