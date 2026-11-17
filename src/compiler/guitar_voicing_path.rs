//! Picks one guitar voicing per chord so the whole progression needs the
//! least hand movement (see `least_movement_path`): each chord (in written
//! order) is a layer, each of its voicings a node, and each edge the
//! movement between two consecutive voicings. Each node also
//! costs a little for how far down chords-db's list it is, so a progression
//! of open chords stays open rather than moving up the neck to save a few
//! string changes.

use itertools::Itertools;

use super::least_movement_path::PathCosts;
use crate::pitch_description::GuitarVoicing;

/// Cost of one string changing fret, against the hand position's shift
/// measured in hundredths of a fret: moving the hand one fret along the neck
/// costs as much as four strings changing fret.
const CHANGED_STRING_COST: u32 = 25;

/// Cost of picking a voicing one place further down chords-db's list (most
/// common first) than its chord's first: half a fret of hand movement.
const LESS_COMMON_VOICING_COST: u32 = 50;

/// For each layer, the index of its voicing on the least-movement path.
/// Every layer must be non-empty, ordered most common first. Ties go to the
/// voicing earliest in its layer, so the result is stable.
pub(crate) fn least_movement_path(layers: &[Vec<&GuitarVoicing>]) -> Vec<usize> {
    super::least_movement_path::least_movement_path(
        layers,
        &PathCosts {
            node: commonness_cost,
            movement: |from: &&GuitarVoicing, to: &&GuitarVoicing| movement_cost(from, to),
        },
    )
}

fn commonness_cost(index_in_layer: usize) -> u32 {
    LESS_COMMON_VOICING_COST * index_in_layer as u32
}

/// How far the hand moves from voicing `from` to voicing `to`: the shift
/// of its position along the neck, plus a little for every string whose
/// fret changes. The same voicing costs 0.
pub(crate) fn movement_cost(from: &GuitarVoicing, to: &GuitarVoicing) -> u32 {
    let changed_strings = from
        .absolute_frets()
        .iter()
        .zip(to.absolute_frets())
        .filter(|(from, to)| **from != *to)
        .count() as u32;
    hand_position(from).abs_diff(hand_position(to)) + CHANGED_STRING_COST * changed_strings
}

/// The hand's position along the neck in hundredths of a fret: the mean of
/// the fretted strings' absolute frets, or 0 when every string is open or
/// muted.
fn hand_position(voicing: &GuitarVoicing) -> u32 {
    let fretted = voicing
        .absolute_frets()
        .into_iter()
        .flatten()
        .filter(|&fret| fret > 0)
        .map(u32::from)
        .collect_vec();
    match fretted.len() as u32 {
        0 => 0,
        count => fretted.iter().sum::<u32>() * 100 / count,
    }
}

#[cfg(test)]
#[path = "tests_guitar_voicing_path.rs"]
mod tests;
