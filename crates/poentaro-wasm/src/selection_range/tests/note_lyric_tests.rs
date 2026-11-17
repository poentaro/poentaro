use super::test_helpers::{assert_range, lyric, lyric_cell, note, note_cell};

#[test]
fn lyric_of_the_notes_own_part_ranges_over_the_measure_rectangle() {
    assert_range(
        &note(0, 0),
        &lyric(1, 2),
        &[note_cell(0, 0), note_cell(0, 1), note_cell(0, 2)],
        &[lyric_cell(1, 0), lyric_cell(1, 1), lyric_cell(1, 2)],
    );
}

#[test]
fn sweeping_down_to_a_later_lyric_part_crosses_the_earlier_ones() {
    assert_range(
        &note(0, 0),
        &lyric(2, 1),
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
fn lyric_of_another_part_ranges_by_position_within_measure() {
    assert_range(
        &note(3, 3),
        &lyric(1, 1),
        &[note_cell(3, 3)],
        &[lyric_cell(1, 1), lyric_cell(2, 1)],
    );
}

#[test]
fn note_of_one_part_and_lyric_of_a_later_part_range_across_parts() {
    assert_range(
        &note(0, 1),
        &lyric(4, 3),
        &[note_cell(0, 1), note_cell(3, 3)],
        &[lyric_cell(1, 1), lyric_cell(2, 1), lyric_cell(4, 3)],
    );
}

#[test]
fn note_and_its_own_lyric_select_just_those_two_cells() {
    assert_range(
        &note(0, 0),
        &lyric(1, 0),
        &[note_cell(0, 0)],
        &[lyric_cell(1, 0)],
    );
}
