use super::test_helpers::{assert_range, assert_unresolved, lyric, lyric_cell, lyric_label};

#[test]
fn a_single_label_selects_its_lyric_part_in_range() {
    assert_range(
        &lyric_label(1, 1, 1),
        &lyric_label(1, 1, 1),
        &[],
        &[lyric_cell(1, 1)],
    );
}

#[test]
fn labels_of_different_lyric_parts_range_over_both() {
    assert_range(
        &lyric_label(1, 0, 0),
        &lyric_label(2, 1, 1),
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
fn syllable_and_label_range_over_their_parts() {
    assert_range(
        &lyric(2, 0),
        &lyric_label(4, 1, 1),
        &[],
        &[lyric_cell(2, 0), lyric_cell(2, 1), lyric_cell(4, 3)],
    );
}

#[test]
fn unknown_syllable_with_label_is_unresolved() {
    assert_unresolved(&lyric(1, 99), &lyric_label(1, 0, 0));
}
