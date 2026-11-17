use super::toggle_range_slur;
use crate::source_edit::ByteRange;

/// One `ByteRange` covering the first occurrence of `needle` in `source`.
fn range_of(source: &str, needle: &str) -> ByteRange {
    let start = source.find(needle).unwrap();
    ByteRange {
        start_byte: start as u32,
        end_byte: (start + needle.len()) as u32,
    }
}

/// One `ByteRange` covering only the first `length` bytes of `needle`'s
/// first occurrence in `source` — for selecting a single note that needs
/// surrounding context to be found unambiguously.
fn range_of_prefix(source: &str, needle: &str, length: u32) -> ByteRange {
    let range = range_of(source, needle);
    ByteRange {
        start_byte: range.start_byte,
        end_byte: range.start_byte + length,
    }
}

const MELODY_BASS: &str = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] 1 2 3 4
    [B] 5 6 7 1
    """,
"#;

#[test]
fn slurs_the_selected_notes() {
    let result = toggle_range_slur(MELODY_BASS, &[range_of(MELODY_BASS, "2 3")]);
    assert!(
        result.source.contains("[M] 1 (2 3) 4"),
        "got:\n{}",
        result.source
    );
    assert!(
        result.source.contains("[B] 5 6 7 1"),
        "got:\n{}",
        result.source
    );
}

#[test]
fn slurring_then_toggling_again_restores_the_source() {
    let slurred = toggle_range_slur(MELODY_BASS, &[range_of(MELODY_BASS, "2 3")]);
    let unslurred = toggle_range_slur(&slurred.source, &slurred.ranges);
    assert_eq!(unslurred.source, MELODY_BASS);
}

#[test]
fn slurred_selection_is_remapped_onto_the_new_source() {
    let result = toggle_range_slur(MELODY_BASS, &[range_of(MELODY_BASS, "2 3")]);
    let [range] = result.ranges.as_slice() else {
        panic!("expected one range, got {:?}", result.ranges);
    };
    let selected = &result.source[range.start_byte as usize..range.end_byte as usize];
    assert_eq!(selected, "2 3)");
}

#[test]
fn selection_spanning_two_parts_slurs_each_part_separately() {
    let range = range_of(MELODY_BASS, "3 4\n    [B] 5 6");
    let result = toggle_range_slur(MELODY_BASS, &[range]);
    assert!(
        result.source.contains("[M] 1 2 (3 4)"),
        "got:\n{}",
        result.source
    );
    assert!(
        result.source.contains("[B] (5 6) 7 1"),
        "got:\n{}",
        result.source
    );
}

#[test]
fn part_with_a_single_selected_note_is_not_slurred() {
    let range = range_of(MELODY_BASS, "4\n    [B] 5 6");
    let result = toggle_range_slur(MELODY_BASS, &[range]);
    assert!(
        result.source.contains("[M] 1 2 3 4\n"),
        "got:\n{}",
        result.source
    );
    assert!(
        result.source.contains("[B] (5 6) 7 1"),
        "got:\n{}",
        result.source
    );
}

#[test]
fn single_selected_note_returns_source_unchanged() {
    let result = toggle_range_slur(MELODY_BASS, &[range_of_prefix(MELODY_BASS, "2 3 4", 1)]);
    assert_eq!(result.source, MELODY_BASS);
    assert!(result.ranges.is_empty());
}

#[test]
fn empty_ranges_list_returns_source_unchanged() {
    let result = toggle_range_slur(MELODY_BASS, &[]);
    assert_eq!(result.source, MELODY_BASS);
    assert!(result.ranges.is_empty());
}

const SLURRED: &str = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] (1 2) 3 4
    """,
"#;

#[test]
fn selection_touching_one_note_of_a_group_removes_the_whole_group() {
    let result = toggle_range_slur(SLURRED, &[range_of_prefix(SLURRED, "2) 3", 1)]);
    assert!(
        result.source.contains("[M] 1 2 3 4"),
        "got:\n{}",
        result.source
    );
}

#[test]
fn selection_touching_a_group_and_unslurred_notes_unslurs() {
    let result = toggle_range_slur(SLURRED, &[range_of(SLURRED, "2) 3 4")]);
    assert!(
        result.source.contains("[M] 1 2 3 4"),
        "got:\n{}",
        result.source
    );
}

#[test]
fn selection_outside_every_group_slurs() {
    let result = toggle_range_slur(SLURRED, &[range_of(SLURRED, "3 4")]);
    assert!(
        result.source.contains("[M] (1 2) (3 4)"),
        "got:\n{}",
        result.source
    );
}

const NESTED: &str = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] (3 (2 1)) 4
    """,
"#;

#[test]
fn selection_inside_nested_groups_removes_every_enclosing_group() {
    let result = toggle_range_slur(NESTED, &[range_of_prefix(NESTED, "1))", 1)]);
    assert!(
        result.source.contains("[M] 3 2 1 4"),
        "got:\n{}",
        result.source
    );
}

const TWO_MEASURES: &str = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes, "Harmony" [H]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] 1 2 3 4
    [H] 5 6 7 1

    [M] 5 6 7 1
    [H] 1 2 3 4
    """,
"#;

/// A part-label click selects the part's notes in every measure of the
/// system as disjoint ranges; the whole selection becomes one cross-measure
/// slur, leaving the other part's lines (between the ranges) untouched.
#[test]
fn disjoint_ranges_across_measures_become_one_cross_measure_slur() {
    let result = toggle_range_slur(
        TWO_MEASURES,
        &[
            range_of_prefix(TWO_MEASURES, "1 2 3 4\n    [H]", 7),
            range_of_prefix(TWO_MEASURES, "5 6 7 1\n    [H] 1", 7),
        ],
    );
    assert!(
        result.source.contains("[M] (1 2 3 4\n    [H] 5 6 7 1\n"),
        "got:\n{}",
        result.source
    );
    assert!(
        result.source.contains("[M] 5 6 7 1)\n    [H] 1 2 3 4\n"),
        "got:\n{}",
        result.source
    );
    assert_eq!(result.ranges.len(), 2);

    let unslurred = toggle_range_slur(&result.source, &result.ranges);
    assert_eq!(unslurred.source, TWO_MEASURES);
}
