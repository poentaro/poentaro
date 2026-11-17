use crate::lyric_selection_types::LyricCellIn;
use crate::note_selection_types::NoteCellIn;
use crate::selection_range::fully_selected_labels;

use super::test_helpers::{fixture, lyric_label, measure, part_label};

fn note_cell_in(source_part_index: usize, note_id: usize) -> NoteCellIn {
    NoteCellIn {
        source_part_index,
        note_id,
    }
}

fn lyric_cell_in(source_part_index: usize, note_id: usize) -> LyricCellIn {
    LyricCellIn {
        source_part_index,
        note_id,
    }
}

#[test]
fn a_part_label_is_selected_once_every_note_it_covers_is() {
    let (note_spans, lyric_spans) = fixture();
    let labels = [
        part_label(0, 1, 2),
        part_label(0, 0, 2),
        part_label(3, 1, 2),
    ];
    let selected = [note_cell_in(0, 1), note_cell_in(0, 2)];
    assert_eq!(
        fully_selected_labels(&note_spans, &lyric_spans, &labels, &selected, &[]),
        [true, false, false]
    );
}

#[test]
fn a_lyric_label_is_selected_once_every_syllable_it_covers_is() {
    let (note_spans, lyric_spans) = fixture();
    let labels = [lyric_label(2, 0, 1), lyric_label(1, 0, 2)];
    let selected = [
        lyric_cell_in(2, 0),
        lyric_cell_in(2, 1),
        lyric_cell_in(1, 0),
    ];
    assert_eq!(
        fully_selected_labels(&note_spans, &lyric_spans, &labels, &[], &selected),
        [true, false]
    );
}

#[test]
fn a_label_covering_nothing_or_another_kind_of_id_is_not_selected() {
    let (note_spans, lyric_spans) = fixture();
    let labels = [part_label(3, 0, 0), measure(0, 0)];
    assert_eq!(
        fully_selected_labels(&note_spans, &lyric_spans, &labels, &[], &[]),
        [false, false]
    );
}
