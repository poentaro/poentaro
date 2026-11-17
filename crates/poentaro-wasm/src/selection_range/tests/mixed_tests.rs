use super::test_helpers::{
    assert_range, lyric, lyric_cell, lyric_label, note, note_cell, part_label,
};

#[test]
fn note_to_lyric_label_crosses_the_lyric_parts_above_it() {
    assert_range(
        &note(0, 0),
        &lyric_label(2, 1, 1),
        &[note_cell(0, 0), note_cell(0, 1)],
        &[
            lyric_cell(1, 0),
            lyric_cell(1, 1),
            lyric_cell(2, 0),
            lyric_cell(2, 1),
        ],
    );
}

#[test]
fn lyric_to_part_label_ranges_over_the_parts_between() {
    assert_range(
        &lyric(1, 0),
        &part_label(3, 1, 1),
        &[note_cell(3, 3)],
        &[
            lyric_cell(1, 0),
            lyric_cell(1, 1),
            lyric_cell(2, 0),
            lyric_cell(2, 1),
        ],
    );
}

#[test]
fn part_label_to_lyric_label_ranges_over_both() {
    assert_range(
        &part_label(0, 0, 0),
        &lyric_label(4, 1, 1),
        &[note_cell(0, 0), note_cell(0, 1), note_cell(3, 3)],
        &[
            lyric_cell(1, 0),
            lyric_cell(1, 1),
            lyric_cell(2, 0),
            lyric_cell(2, 1),
            lyric_cell(4, 3),
        ],
    );
}
