use crate::error::Span;
use crate::part_stream::{apply_part_stream, extract_part_stream, PartStreamApplyResult};

const MELODY_AND_BASS: &str = r#"parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    [M]   1  2 3 4   // intro
    [B] 1 1 1 1

    [M] 5 6 7 1 // second
    [B] 1 1 1 1
    """,
"#;

fn applied(result: PartStreamApplyResult) -> String {
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    result.source.unwrap()
}

#[test]
fn inserting_a_note_shifts_later_notes_and_appends_a_measure() {
    let result = apply_part_stream(MELODY_AND_BASS, "M", "0 1 2 3 4 5 6 7 1");
    assert_eq!(result.changed_measure_count, 3);
    assert_eq!(
        applied(result),
        r#"parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    [M]   0 1 2 3   // intro
    [B] 1 1 1 1

    [M] 4 5 6 7 // second
    [B] 1 1 1 1

    [M] 1 0 0 0
    """,
"#
    );
}

#[test]
fn deleting_notes_rest_fills_the_last_measure_and_removes_later_lines() {
    let source = r#"parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    [M] 1 2 3 4
    [B] 1 1 1 1

    [B] 1 1 1 1
    [M] 5 6 7 1

    [M] 1 1 1 1
    [B] 1 1 1 1
    """,
"#;
    let result = apply_part_stream(source, "M", "2 3 4");
    assert_eq!(result.changed_measure_count, 3);
    assert_eq!(
        applied(result),
        r#"parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    [M] 2 3 4 0
    [B] 1 1 1 1

    [B] 1 1 1 1

    [B] 1 1 1 1
    """,
"#
    );
}

#[test]
fn emptying_a_group_whose_only_line_is_this_part_rest_fills_it() {
    let source = r#"parts ["Melody" [M]: notes],

score """
    [M] 1 2 3 4

    [M] 5 6 7 1
    """,
"#;
    let result = apply_part_stream(source, "M", "1 2 3 4");
    assert_eq!(result.changed_measure_count, 1);
    assert!(applied(result).ends_with("[M] 1 2 3 4\n\n    [M] 0 0 0 0\n    \"\"\",\n"));
}

#[test]
fn implicit_padding_is_kept_while_the_measure_is_unchanged() {
    let source = r#"parts ["Melody" [M]: notes],

score """
    [M] 1 2

    [M] 3 4
    """,
"#;
    assert_eq!(
        extract_part_stream(source, "M").unwrap().text,
        "1 2 - -\n3 4 - -"
    );
    let result = apply_part_stream(source, "M", "1 2 - - 3 4 5 -");
    assert_eq!(result.changed_measure_count, 1);
    assert!(applied(result).ends_with("[M] 1 2\n\n    [M] 3 4 5 -\n    \"\"\",\n"));
}

#[test]
fn unwritten_measures_stay_unwritten_and_written_rests_stay_written() {
    let source = r#"parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    [M] 1 2 3 4
    [B] 1 1 1 1

    [B] 1 1 1 1

    [M] 0 0 0 0
    [B] 1 1 1 1
    """,
"#;
    let stream = extract_part_stream(source, "M").unwrap();
    assert_eq!(stream.text, "1 2 3 4\n0 0 0 0\n0 0 0 0");
    let unchanged = apply_part_stream(source, "M", &stream.text);
    assert_eq!(unchanged.changed_measure_count, 0);
    assert_eq!(unchanged.source.as_deref(), Some(source));

    let result = apply_part_stream(source, "M", "5 5 5 5 0 0 0 0 0 0 0 0");
    assert_eq!(result.changed_measure_count, 1);
    assert!(applied(result)
        .contains("[M] 5 5 5 5\n    [B] 1 1 1 1\n\n    [B] 1 1 1 1\n\n    [M] 0 0 0 0\n"));
}

#[test]
fn writing_into_an_unwritten_measure_inserts_a_line_in_declaration_order() {
    let source = r#"parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    [B] 1 1 1 1
    """,
"#;
    assert_eq!(
        applied(apply_part_stream(source, "M", "5 6 7 1")),
        r#"parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    [M] 5 6 7 1
    [B] 1 1 1 1
    """,
"#
    );
    assert_eq!(
        applied(apply_part_stream(source, "B", "2 2 2 2 3")),
        r#"parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    [B] 2 2 2 2

    [B] 3 0 0 0
    """,
"#
    );
}

#[test]
fn a_time_signature_change_re_bars_by_the_new_capacity() {
    let source = r#"parts ["Melody" [M]: notes],

score """
    [M] 1 2 3 4

    time=3/4
    [M] 5 6 7
    """,
"#;
    let result = apply_part_stream(source, "M", "0 1 2 3 4 5 6 7");
    assert_eq!(result.measure_starts, vec![0, 8, 14]);
    assert_eq!(
        applied(result),
        r#"parts ["Melody" [M]: notes],

score """
    [M] 0 1 2 3

    time=3/4
    [M] 4 5 6

    [M] 7 0 0
    """,
"#
    );
}

#[test]
fn unchanged_measures_keep_comments_and_spacing() {
    let result = apply_part_stream(MELODY_AND_BASS, "M", "1 2 3 4 1 1 1 1");
    assert_eq!(result.changed_measure_count, 1);
    let source = applied(result);
    assert!(source.contains("[M]   1  2 3 4   // intro\n"), "{source}");
    assert!(source.contains("[M] 1 1 1 1 // second\n"), "{source}");
}

#[test]
fn applying_an_untouched_stream_re_bars_an_overflowing_measure() {
    let source = r#"parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    [M] 1 2 3 4 5
    [B] 1 1 1 1

    [M] 6 7 1 2
    [B] 1 1 1 1
    """,
"#;
    let stream = extract_part_stream(source, "M").unwrap().text;
    let result = apply_part_stream(source, "M", &stream);
    assert_eq!(result.changed_measure_count, 3);
    assert_eq!(
        applied(result),
        r#"parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    [M] 1 2 3 4
    [B] 1 1 1 1

    [M] 5 6 7 1
    [B] 1 1 1 1

    [M] 2 0 0 0
    """,
"#
    );
}

#[test]
fn applying_an_untouched_stream_re_bars_an_underflowing_measure() {
    let source = r#"parts ["Melody" [M]: notes],

score """
    [M] 1 2 3_

    [M] 4_ 5 6 7
    """,
"#;
    let stream = extract_part_stream(source, "M").unwrap().text;
    let result = apply_part_stream(source, "M", &stream);
    assert_eq!(result.changed_measure_count, 2);
    let source = applied(result);
    assert!(
        source.ends_with("[M] 1 2 3_ 4_ 5\n\n    [M] 6 7 0 0\n    \"\"\",\n"),
        "{source}"
    );
}

#[test]
fn measures_before_a_misbarred_one_are_left_alone() {
    let source = r#"parts ["Melody" [M]: notes],

score """
    [M]   1  2 3 4   // intro

    [M] 5 6 7 1 2
    """,
"#;
    let stream = extract_part_stream(source, "M").unwrap().text;
    let result = apply_part_stream(source, "M", &stream);
    assert_eq!(result.changed_measure_count, 2);
    let source = applied(result);
    assert!(source.contains("[M]   1  2 3 4   // intro\n"), "{source}");
    assert!(
        source.ends_with("[M] 5 6 7 1\n\n    [M] 2 0 0 0\n    \"\"\",\n"),
        "{source}"
    );
}

#[test]
fn a_note_crossing_a_barline_after_re_barring_is_an_error() {
    let source = r#"parts ["Melody" [M]: notes],

score """
    [M] 1 2 3 4_ 5 -
    """,
"#;
    let stream = extract_part_stream(source, "M").unwrap().text;
    let result = apply_part_stream(source, "M", &stream);
    assert_eq!(result.source, None);
    assert!(result.diagnostics[0]
        .message
        .contains("crosses the barline of measure 1"));
}

#[test]
fn an_ineligible_part_is_refused() {
    let result = apply_part_stream(MELODY_AND_BASS, "X", "1 2 3 4");
    assert_eq!(result.source, None);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].message.contains("no part [X]"));
}

const FOLLOWING: &str = r#"parts ["Soprano" [S]: notes, "Alto" [A]: follow [S]],

score """
    [S] 1 2 3 4

    [S] 5 6 7 1
    [A] 3 4 5 6

    [S] 1 1 1 1
    """,
"#;

#[test]
fn a_follow_part_shows_unwritten_measures_as_placeholders_and_round_trips() {
    let stream = extract_part_stream(FOLLOWING, "A").unwrap();
    assert_eq!(stream.text, "@\n3 4 5 6\n@");
    assert_eq!(stream.measure_starts, vec![0, 2, 10]);
    let result = apply_part_stream(FOLLOWING, "A", &stream.text);
    assert_eq!(result.changed_measure_count, 0);
    assert_eq!(result.measure_starts, stream.measure_starts);
    assert_eq!(result.source.as_deref(), Some(FOLLOWING));
}

#[test]
fn a_placeholder_removes_the_follow_parts_line() {
    let result = apply_part_stream(FOLLOWING, "A", "@ @ @");
    assert_eq!(result.changed_measure_count, 1);
    assert!(applied(result).contains("[S] 5 6 7 1\n\n    [S] 1 1 1 1\n"));
}

#[test]
fn writing_over_a_placeholder_inserts_the_follow_parts_line() {
    let result = apply_part_stream(FOLLOWING, "A", "@ 3 4 5 6 2 2 2 2");
    assert_eq!(result.changed_measure_count, 1);
    assert!(applied(result).ends_with("[S] 1 1 1 1\n    [A] 2 2 2 2\n    \"\"\",\n"));
}

#[test]
fn a_shorter_follow_stream_writes_explicit_rests() {
    let result = apply_part_stream(FOLLOWING, "A", "@ 3 4");
    assert_eq!(result.changed_measure_count, 2);
    let source = applied(result);
    assert!(
        source.ends_with(
            "[S] 5 6 7 1\n    [A] 3 4 0 0\n\n    [S] 1 1 1 1\n    [A] 0 0 0 0\n    \"\"\",\n"
        ),
        "{source}"
    );
}

#[test]
fn a_placeholder_partway_through_a_measure_is_an_error() {
    let result = apply_part_stream(FOLLOWING, "A", "@ 3 4 @ 5 6 @");
    assert_eq!(result.source, None);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].span, Span::new(6, 7));
    assert!(result.diagnostics[0]
        .message
        .contains("partway through measure 2"));
}

#[test]
fn a_placeholder_cannot_add_a_measure() {
    let result = apply_part_stream(FOLLOWING, "A", "@ 3 4 5 6 @ @");
    assert_eq!(result.source, None);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].span, Span::new(12, 13));
}

#[test]
fn a_placeholder_on_a_groups_only_line_writes_rests() {
    let source = r#"parts ["Soprano" [S]: notes, "Alto" [A]: follow [S]],

score """
    [S] 1 2 3 4

    [A] 5 6 7 1
    """,
"#;
    let result = apply_part_stream(source, "A", "@ @");
    assert!(applied(result).ends_with("[S] 1 2 3 4\n\n    [A] 0 0 0 0\n    \"\"\",\n"));
}

#[test]
fn a_placeholder_in_a_non_follow_part_is_an_error() {
    let result = apply_part_stream(FOLLOWING, "S", "1 2 3 4 @");
    assert_eq!(result.source, None);
    assert_eq!(result.diagnostics[0].span, Span::new(8, 9));
    assert!(result.diagnostics[0]
        .message
        .contains("only allowed in follow parts"));
}
