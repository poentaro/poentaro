use super::test_helpers::{assert_range, lyric_cell, measure, note_cell};

#[test]
fn single_measure_selects_every_part() {
    assert_range(
        &measure(1, 1),
        &measure(1, 1),
        &[note_cell(0, 1), note_cell(3, 3)],
        &[lyric_cell(1, 1), lyric_cell(2, 1), lyric_cell(4, 3)],
    );
}

#[test]
fn multi_measure_range_selects_everything_between() {
    assert_range(
        &measure(0, 0),
        &measure(2, 2),
        &[
            note_cell(0, 0),
            note_cell(0, 1),
            note_cell(0, 2),
            note_cell(3, 3),
        ],
        &[
            lyric_cell(1, 0),
            lyric_cell(1, 1),
            lyric_cell(1, 2),
            lyric_cell(2, 0),
            lyric_cell(2, 1),
            lyric_cell(4, 3),
        ],
    );
}
