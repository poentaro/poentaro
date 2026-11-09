use super::toggle_range_tie;
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

#[test]
fn ties_selected_same_pitch_neighbours() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] 1 3 3 3
    [B] 5 6 7 1
    """,
"#;
    let result = toggle_range_tie(source, &[range_of(source, "3 3 3")]);
    assert!(
        result.source.contains("[M] 1 3~ 3~ 3\n"),
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
fn different_pitches_are_not_tied() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] 1 2 3 4
    [B] 5 6 7 1
    """,
"#;
    let result = toggle_range_tie(source, &[range_of(source, "1 2 3 4")]);
    assert_eq!(result.source, source);
    assert!(result.ranges.is_empty());
}

#[test]
fn different_octaves_are_not_tied() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] 1 1' 3 4
    [B] 5 6 7 1
    """,
"#;
    let result = toggle_range_tie(source, &[range_of(source, "1 1'")]);
    assert_eq!(result.source, source);
}

#[test]
fn a_rest_between_same_pitches_blocks_the_tie() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] 3 0 3 4
    [B] 5 6 7 1
    """,
"#;
    let result = toggle_range_tie(source, &[range_of(source, "3 0 3")]);
    assert_eq!(result.source, source);
}

#[test]
fn tie_goes_after_every_duration_suffix() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] 3'_ 3'_ 3--
    [B] 5 6 7 1
    """,
"#;
    let result = toggle_range_tie(source, &[range_of(source, "3'_ 3'_")]);
    assert!(
        result.source.contains("[M] 3'_~ 3'_ 3--"),
        "got:\n{}",
        result.source
    );
}

#[test]
fn single_selected_note_is_tied_into_the_next_note() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] 1 2 2 4
    [B] 5 6 7 1
    """,
"#;
    let result = toggle_range_tie(source, &[range_of_prefix(source, "2 2 4", 1)]);
    assert!(
        result.source.contains("[M] 1 2~ 2 4"),
        "got:\n{}",
        result.source
    );
}

#[test]
fn multi_note_selection_does_not_tie_out_of_itself() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] 1 1 2 2
    [B] 5 6 7 1
    """,
"#;
    let result = toggle_range_tie(source, &[range_of(source, "1 1 2")]);
    assert!(
        result.source.contains("[M] 1~ 1 2 2"),
        "got:\n{}",
        result.source
    );
}

#[test]
fn ties_across_a_measure_boundary() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] 1 2 3 4
    [B] 5 6 7 1

    [M] 4 3 2 1
    [B] 1 7 6 5
    """,
"#;
    let result = toggle_range_tie(
        source,
        &[
            range_of_prefix(source, "4\n    [B] 5", 1),
            range_of_prefix(source, "4 3 2 1", 1),
        ],
    );
    assert!(
        result.source.contains("[M] 1 2 3 4~\n"),
        "got:\n{}",
        result.source
    );
    assert!(
        result.source.contains("[M] 4 3 2 1\n"),
        "got:\n{}",
        result.source
    );
}

#[test]
fn tying_then_toggling_again_restores_the_source() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] 3 3 3 4
    [B] 5 5 7 1
    """,
"#;
    let range = range_of(source, "3 3 3 4\n    [B] 5 5");
    let tied = toggle_range_tie(source, &[range]);
    assert!(
        tied.source.contains("[M] 3~ 3~ 3 4\n    [B] 5~ 5 7 1"),
        "got:\n{}",
        tied.source
    );
    let untied = toggle_range_tie(&tied.source, &tied.ranges);
    assert_eq!(untied.source, source);
}

#[test]
fn tied_selection_is_remapped_onto_the_new_source() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] 1 3 3 4
    [B] 5 6 7 1
    """,
"#;
    let result = toggle_range_tie(source, &[range_of(source, "3 3")]);
    let [range] = result.ranges.as_slice() else {
        panic!("expected one range, got {:?}", result.ranges);
    };
    let selected = &result.source[range.start_byte as usize..range.end_byte as usize];
    assert_eq!(selected, "3~ 3");
}

#[test]
fn selection_touching_any_tie_unties_instead_of_tying() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] 3~ 3 4 4
    [B] 5 6 7 1
    """,
"#;
    let result = toggle_range_tie(source, &[range_of(source, "3~ 3 4 4")]);
    assert!(
        result.source.contains("[M] 3 3 4 4\n"),
        "got:\n{}",
        result.source
    );
}

#[test]
fn untie_keeps_a_glued_repeat_atom_separate() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] 5~_ 5_ 5 5 5
    [B] 5 6 7 1
    """,
"#;
    let result = toggle_range_tie(source, &[range_of_prefix(source, "5~_", 1)]);
    assert!(
        result.source.contains("[M] 5 _ 5_ 5 5 5\n"),
        "got:\n{}",
        result.source
    );
}

#[test]
fn ties_same_chords() {
    let source = r#"metadata { title "t", author "a" },

parts ["Chords" [C]: chords],

score """
    time=4/4 key=C4 bpm=120
    [C] 1m 1m 4 5
    """,
"#;
    let result = toggle_range_tie(source, &[range_of(source, "1m 1m 4")]);
    assert!(
        result.source.contains("[C] 1m~ 1m 4 5"),
        "got:\n{}",
        result.source
    );
}

#[test]
fn empty_ranges_list_returns_source_unchanged() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    time=4/4 key=C4 bpm=120
    [M] 3 3 3 4
    [B] 5 6 7 1
    """,
"#;
    let result = toggle_range_tie(source, &[]);
    assert_eq!(result.source, source);
    assert!(result.ranges.is_empty());
}
