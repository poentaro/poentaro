use super::test_helpers::{assert_range, assert_unresolved, lyric, lyric_cell};

#[test]
fn single_syllable() {
    assert_range(&lyric(1, 1), &lyric(1, 1), &[], &[lyric_cell(1, 1)]);
}

#[test]
fn same_lyric_part_ranges_by_note_id() {
    assert_range(
        &lyric(1, 0),
        &lyric(1, 2),
        &[],
        &[lyric_cell(1, 0), lyric_cell(1, 1), lyric_cell(1, 2)],
    );
}

#[test]
fn different_lyric_parts_range_by_part_and_measure() {
    assert_range(
        &lyric(1, 0),
        &lyric(2, 1),
        &[],
        &[
            lyric_cell(1, 0),
            lyric_cell(1, 1),
            lyric_cell(2, 0),
            lyric_cell(2, 1),
        ],
    );
}

#[test]
fn unknown_syllable_is_unresolved() {
    assert_unresolved(&lyric(1, 99), &lyric(2, 0));
}
