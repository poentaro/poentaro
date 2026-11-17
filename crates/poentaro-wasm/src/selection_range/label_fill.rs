use std::collections::HashSet;

use crate::lyric_selection_types::LyricCellIn;
use crate::note_selection_types::NoteCellIn;
use crate::types::{LyricSpanOut, NoteSpanOut};

use super::types::ClickableElementId;

/// Whether each label reads as selected: a part label once every note it
/// covers (its part across its own measures) is in `note_cells`, a lyric
/// label once every syllable it covers is in `lyric_cells`. A label covering
/// nothing, and any other kind of id, is never selected.
pub(crate) fn fully_selected_labels(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    labels: &[ClickableElementId],
    note_cells: &[NoteCellIn],
    lyric_cells: &[LyricCellIn],
) -> Vec<bool> {
    let note_cells: HashSet<&NoteCellIn> = note_cells.iter().collect();
    let lyric_cells: HashSet<&LyricCellIn> = lyric_cells.iter().collect();
    labels
        .iter()
        .map(|label| match *label {
            ClickableElementId::PartLabel {
                source_part_index,
                measure_index_start,
                measure_index_end,
            } => all_selected(
                note_spans
                    .iter()
                    .filter(|span| {
                        span.source_part_index == source_part_index
                            && (measure_index_start..=measure_index_end)
                                .contains(&span.measure_index)
                    })
                    .map(|span| {
                        note_cells.contains(&NoteCellIn {
                            source_part_index: span.source_part_index,
                            note_id: span.note_id,
                        })
                    }),
            ),
            ClickableElementId::LyricLabel {
                source_part_index,
                measure_index_start,
                measure_index_end,
            } => all_selected(
                lyric_spans
                    .iter()
                    .filter(|span| {
                        span.source_part_index == source_part_index
                            && (measure_index_start..=measure_index_end)
                                .contains(&span.measure_index)
                    })
                    .map(|span| {
                        lyric_cells.contains(&LyricCellIn {
                            source_part_index: span.source_part_index,
                            note_id: span.note_id,
                        })
                    }),
            ),
            _ => false,
        })
        .collect()
}

/// True when there is at least one covered cell and every one is selected.
fn all_selected(mut selected: impl Iterator<Item = bool>) -> bool {
    selected
        .next()
        .is_some_and(|first| first && selected.all(|is| is))
}
