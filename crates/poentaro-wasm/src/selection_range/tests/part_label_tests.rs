use super::test_helpers::{assert_range, lyric_cell, note, note_cell, part_label};

#[test]
fn a_single_part_label_selects_only_its_notes() {
    assert_range(
        &part_label(0, 1, 1),
        &part_label(0, 1, 1),
        &[note_cell(0, 1)],
        &[],
    );
}

#[test]
fn part_labels_across_systems_widen_the_measure_range() {
    assert_range(
        &part_label(0, 0, 0),
        &part_label(0, 1, 1),
        &[note_cell(0, 0), note_cell(0, 1)],
        &[],
    );
}

#[test]
fn a_swept_range_covers_only_the_parts_between_the_labels() {
    // Part 4 (B's lyric part) lies past the swept range 0..=3, so it is
    // left out even though it sings along to part 3.
    assert_range(
        &part_label(0, 1, 1),
        &part_label(3, 1, 1),
        &[note_cell(0, 1), note_cell(3, 3)],
        &[lyric_cell(1, 1), lyric_cell(2, 1)],
    );
}

#[test]
fn note_and_label_of_the_same_part_select_no_lyrics() {
    assert_range(
        &note(0, 2),
        &part_label(0, 1, 1),
        &[note_cell(0, 1), note_cell(0, 2)],
        &[],
    );
}

#[test]
fn note_and_label_of_another_part_cover_the_parts_between() {
    assert_range(
        &note(0, 0),
        &part_label(3, 1, 1),
        &[note_cell(0, 0), note_cell(0, 1), note_cell(3, 3)],
        &[
            lyric_cell(1, 0),
            lyric_cell(1, 1),
            lyric_cell(2, 0),
            lyric_cell(2, 1),
        ],
    );
}
