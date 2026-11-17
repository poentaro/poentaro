//! The guitar chord view: every chord symbol that chords-db has a voicing
//! for gets a chord diagram, chosen per part so the progression (in written
//! order) needs the least hand movement — see `least_movement_path`.
//! Everything else about the layout stays as in the numbered view.

use std::collections::HashMap;

use itertools::Itertools;

use super::chord_degrees::ChordDegrees;
use super::element_position::{apply_picks, positioned_elements, ElementPosition};
use super::guitar_voicing_path::least_movement_path;
use super::letter_names::block_tonics;
use super::types::{ColumnElement, CompileResult, ElementContent, MeasureBlock};
use crate::ast::grouped::Score;
use crate::pitch_description::spelling::SpelledPitch;
use crate::pitch_description::{guitar_voicings, GuitarVoicing};

/// A chord symbol and every voicing chords-db has for it.
struct ChordCandidates {
    position: ElementPosition,
    source_part_index: usize,
    voicings: Vec<&'static GuitarVoicing>,
}

/// `result`, compiled from `score`, with each chord symbol's
/// `guitar_voicing` set to its voicing on its part's least-movement path.
/// A chord chords-db has no voicing for keeps `None` and is skipped by the
/// path, as if it weren't there.
pub(super) fn place_guitar_voicings(result: CompileResult, score: &Score) -> CompileResult {
    let tonics = block_tonics(&result.blocks, score);
    let chosen: HashMap<ElementPosition, GuitarVoicing> = chord_candidates(&result.blocks, &tonics)
        .into_iter()
        .into_group_map_by(|candidates| candidates.source_part_index)
        .into_values()
        .flat_map(|part_chords| {
            let path = least_movement_path(
                &part_chords
                    .iter()
                    .map(|candidates| candidates.voicings.clone())
                    .collect_vec(),
            );
            part_chords
                .into_iter()
                .zip(path)
                .filter_map(|(candidates, index)| {
                    Some((
                        candidates.position,
                        (*candidates.voicings.get(index)?).clone(),
                    ))
                })
                .collect_vec()
        })
        .collect();
    apply_picks(result, chosen, with_voicing)
}

/// Every chord symbol with at least one chords-db voicing, in written order.
fn chord_candidates(blocks: &[MeasureBlock], tonics: &[SpelledPitch]) -> Vec<ChordCandidates> {
    positioned_elements(blocks)
        .filter_map(|positioned| match &positioned.element.content {
            ElementContent::ChordSymbol { degrees, .. } => Some(ChordCandidates {
                position: positioned.position,
                source_part_index: positioned.source_part_index,
                voicings: chord_voicings(degrees, *tonics.get(positioned.position.block)?),
            }),
            _ => None,
        })
        .filter(|candidates| !candidates.voicings.is_empty())
        .collect()
}

fn chord_voicings(degrees: &ChordDegrees, tonic: SpelledPitch) -> Vec<&'static GuitarVoicing> {
    guitar_voicings(
        &degrees.triad,
        degrees.extension.as_ref(),
        tonic.scale_degree(&degrees.degree, &degrees.accidental),
        degrees
            .bass
            .as_ref()
            .map(|bass| tonic.scale_degree(&bass.degree, &bass.accidental)),
    )
}

fn with_voicing(element: ColumnElement, voicing: GuitarVoicing) -> ColumnElement {
    match element.content {
        ElementContent::ChordSymbol {
            text,
            degrees,
            dotted,
            double_dotted,
            guitar_voicing: _,
        } => ColumnElement {
            content: ElementContent::ChordSymbol {
                text,
                degrees,
                dotted,
                double_dotted,
                guitar_voicing: Some(voicing),
            },
            ..element
        },
        _ => element,
    }
}

#[cfg(test)]
#[path = "tests_guitar_chords.rs"]
mod tests;
