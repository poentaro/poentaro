use super::is_chord_only_row;
use crate::compiler::types::{ElementContent, MeasureBlock, MeasureRow};
use crate::grid_layout::types::GridElement;
use std::collections::{HashMap, HashSet};

/// Which parts draw the optional bands atop their sub-rows in one system:
/// a note part's `tuplet_bracket` band when it has a tuplet bracket in the
/// system, and any part's guitar-diagram band when one of its chord symbols
/// carries a diagram in the system (the guitar chord view). Part indices are
/// consolidated ones (row position within the system's first block's
/// `rows`). Shared by every pass that needs to know which sub-rows a part
/// reserves in this particular system — layout height math
/// (`system_musical_height_pt`), row expansion (`expand_note_part`), and the
/// highlight/click/playback-cursor row-counting passes (`grid_layout::highlight`,
/// `grid_layout::playback_cursor`) all must agree on this or their row indices
/// drift apart.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SystemBands {
    pub(crate) tuplet_part_indices: HashSet<usize>,
    fermata_octaves: HashMap<usize, i8>,
    pub(crate) guitar_diagram_part_indices: HashSet<usize>,
}

impl SystemBands {
    /// `tuplet_part_indices` as given; guitar-diagram parts read off `system`.
    pub(crate) fn of(system: &[MeasureBlock], tuplet_part_indices: HashSet<usize>) -> Self {
        let part_count = system.first().map_or(0, |first| first.rows.len());
        SystemBands {
            tuplet_part_indices,
            fermata_octaves: (0..part_count)
                .filter_map(|idx| {
                    let octave = system
                        .iter()
                        .filter_map(|block| block.rows.get(idx))
                        .flat_map(|row| &row.elements)
                        .filter_map(|element| match element.content {
                            ElementContent::NoteHead {
                                fermata: true,
                                octave,
                                ..
                            } => Some(octave.max(0)),
                            ElementContent::Rest { fermata: true, .. } => Some(0),
                            _ => None,
                        })
                        .max()?;
                    Some((idx, octave))
                })
                .collect(),
            guitar_diagram_part_indices: (0..part_count)
                .filter(|&idx| {
                    system.iter().any(|block| {
                        block.rows.get(idx).is_some_and(|row| {
                            row.elements.iter().any(|element| {
                                matches!(
                                    element.content,
                                    ElementContent::ChordSymbol {
                                        guitar_voicing: Some(_),
                                        ..
                                    }
                                )
                            })
                        })
                    })
                })
                .collect(),
        }
    }

    pub(crate) fn fermata_clearance(&self, part_idx: usize, base: f32) -> f32 {
        self.fermata_octaves
            .get(&part_idx)
            .map_or(0.0, |&octave| base * (0.92 + 0.30 * f32::from(octave)))
    }

    pub(crate) fn has_tuplet_bracket(&self, part_idx: usize) -> bool {
        self.tuplet_part_indices.contains(&part_idx)
    }

    pub(crate) fn has_guitar_diagrams(&self, part_idx: usize) -> bool {
        self.guitar_diagram_part_indices.contains(&part_idx)
    }

    /// How many grid rows the non-lyric part `part_template` at `part_idx`
    /// expands to (see `expand_note_part`).
    pub(crate) fn sub_row_count(&self, part_idx: usize, part_template: &MeasureRow) -> usize {
        let musical = if is_chord_only_row(part_template) {
            4
        } else if self.has_tuplet_bracket(part_idx) {
            7
        } else {
            6
        };
        musical + usize::from(self.has_guitar_diagrams(part_idx))
    }
}

/// [`SystemBands`] for one system: its tuplet parts per `tuplet_bracket_map`
/// (keyed by `(abs_sys, source_part_index)`, see `resolve_tuplet_spans`).
pub(crate) fn system_bands(
    system: &[MeasureBlock],
    tuplet_bracket_map: &HashMap<(usize, usize), Vec<GridElement>>,
    abs_sys: usize,
) -> SystemBands {
    let tuplet_part_indices = system.first().map_or_else(HashSet::new, |first| {
        first
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| tuplet_bracket_map.contains_key(&(abs_sys, row.source_part_index)))
            .map(|(idx, _)| idx)
            .collect()
    });
    SystemBands::of(system, tuplet_part_indices)
}
