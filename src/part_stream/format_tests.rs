use crate::part_stream::{
    extract_part_stream, format_part_stream, FormattedPartStream, StreamCursor,
};

const THREE_MEASURES: &str = r#"parts ["Melody" [M]: notes, "Words" [W]: lyrics [M], "Accompaniment" [A]: follow [M]],

score """
    [M] 1 2 3 4
    [W] a b c d

    [M] 5 6 7 1
    [W] e f g h

    [M] 3 4 5 6
    [A] 1 1 1 1
    """,
"#;

const CHORDS_AND_WORDS: &str = r#"parts ["Chords" [C]: chords, "Words" [W]: lyrics [C]],

score """
    [C] 1 - 4 -
    [W] (天) 空  好 (藍)

    [C] 0 5 - -
    [W] 我 （在）想 你
    """,
"#;

fn apply_edits(stream: &str, formatted: &FormattedPartStream) -> String {
    formatted
        .edits
        .iter()
        .rev()
        .fold(stream.to_string(), |mut text, edit| {
            text.replace_range(edit.span.start..edit.span.end, &edit.replacement);
            text
        })
}

fn formatted(source: &str, abbreviation: &str, stream: &str) -> String {
    apply_edits(
        stream,
        &format_part_stream(source, abbreviation, stream, &[]),
    )
}

/// Formats `stream`, where `|` marks the cursor, and returns the result with
/// the mapped cursor marked the same way.
fn formatted_with_cursor(source: &str, abbreviation: &str, marked: &str) -> String {
    let offset = marked.find('|').unwrap();
    let stream = marked.replacen('|', "", 1);
    let result = format_part_stream(source, abbreviation, &stream, &[StreamCursor { offset }]);
    let mut text = apply_edits(&stream, &result);
    text.insert(result.cursors[0].offset, '|');
    text
}

#[test]
fn a_note_that_overflows_its_measure_moves_to_the_next_line() {
    assert_eq!(
        formatted(THREE_MEASURES, "M", "0 1 2 3 4\n5 6 7 1\n3 4 5 6"),
        "0 1 2 3\n4 5 6 7\n1 3 4 5\n6"
    );
}

#[test]
fn deleting_a_note_pulls_every_later_measure_back() {
    assert_eq!(
        formatted(THREE_MEASURES, "M", "2 3 4\n5 6 7 1\n3 4 5 6"),
        "2 3 4 5\n6 7 1 3\n4 5 6"
    );
}

#[test]
fn notes_without_whitespace_are_cut_at_the_barline() {
    assert_eq!(formatted(THREE_MEASURES, "M", "12345"), "1234\n5");
}

#[test]
fn a_line_break_inside_a_measure_is_collapsed() {
    assert_eq!(
        formatted(THREE_MEASURES, "M", "1 2\n 3 4\n5 6 7 1\n3 4 5 6"),
        "1 2 3 4\n5 6 7 1\n3 4 5 6"
    );
}

#[test]
fn leading_whitespace_is_removed_and_trailing_whitespace_kept() {
    assert_eq!(
        formatted(THREE_MEASURES, "M", "\n  1 2 3 4 5 \n"),
        "1 2 3 4\n5 \n"
    );
}

#[test]
fn a_stream_with_an_error_is_left_as_typed() {
    let unclosed = format_part_stream(THREE_MEASURES, "M", "(1 2 3 4 5 6", &[]);
    assert_eq!(unclosed.edits, vec![]);
}

#[test]
fn deleting_a_note_pulls_a_held_note_back_across_the_barline() {
    let source = r#"parts ["Melody" [M]: notes],

score """
    [M] 1 2 3 4

    [M] 5 - 6 7

    [M] 1 2 3 4
    """,
"#;
    assert_eq!(
        formatted(source, "M", "2 3 4\n5 - 6 7\n1 2 3 4"),
        "2 3 4 5 -\n6 7 1 2\n3 4"
    );
}

#[test]
fn a_note_crossing_a_barline_stays_on_the_line_it_starts() {
    assert_eq!(
        formatted(THREE_MEASURES, "M", "1 2 3 4. 5 6 7"),
        "1 2 3 4.\n5 6 7"
    );
}

#[test]
fn a_tuplet_crossing_a_barline_does_not_block_re_barring() {
    assert_eq!(
        formatted(THREE_MEASURES, "M", "1 2\n3 1_ 2:{1 1} 4 5"),
        "1 2 3 1_ 2:{1\n1} 4 5"
    );
}

#[test]
fn formatting_is_idempotent() {
    let streams = [
        ("M", "0 1 2 3 4\n5 6 7 1\n3 4 5 6"),
        ("M", "12345 6 7\n\n1"),
        ("W", "z a b c d e f g h"),
        ("A", "@\n@ 1 1 1 1"),
    ];
    for (abbreviation, stream) in streams {
        let once = formatted(THREE_MEASURES, abbreviation, stream);
        let again = format_part_stream(THREE_MEASURES, abbreviation, &once, &[]);
        assert_eq!(again.edits, vec![], "{once:?}");
    }
}

#[test]
fn an_extracted_stream_is_already_formatted() {
    for (source, abbreviation) in [
        (THREE_MEASURES, "M"),
        (THREE_MEASURES, "W"),
        (THREE_MEASURES, "A"),
        (CHORDS_AND_WORDS, "W"),
    ] {
        let stream = extract_part_stream(source, abbreviation).unwrap();
        let result = format_part_stream(source, abbreviation, &stream.text, &[]);
        assert_eq!(result.edits, vec![], "{abbreviation}: {:?}", stream.text);
    }
}

#[test]
fn a_cursor_before_a_cut_stays_and_one_after_it_follows_its_note() {
    assert_eq!(
        formatted_with_cursor(THREE_MEASURES, "M", "0| 1 2 3 4\n5 6 7 1\n3 4 5 6"),
        "0| 1 2 3\n4 5 6 7\n1 3 4 5\n6"
    );
    assert_eq!(
        formatted_with_cursor(THREE_MEASURES, "M", "0 1 2 3 4|\n5 6 7 1\n3 4 5 6"),
        "0 1 2 3\n4| 5 6 7\n1 3 4 5\n6"
    );
}

#[test]
fn a_space_that_fills_a_measure_moves_the_cursor_to_the_next_line() {
    assert_eq!(
        formatted_with_cursor(THREE_MEASURES, "M", "1 2 3 4 |5 6 7 1"),
        "1 2 3 4\n|5 6 7 1"
    );
}

#[test]
fn a_cursor_after_a_collapsed_line_break_stays_before_the_next_note() {
    assert_eq!(
        formatted_with_cursor(THREE_MEASURES, "M", "1 2\n|3 4\n5 6 7 1\n3 4 5 6"),
        "1 2 |3 4\n5 6 7 1\n3 4 5 6"
    );
}

#[test]
fn a_lyric_stream_is_re_cut_by_its_targets_slots() {
    assert_eq!(
        formatted(THREE_MEASURES, "W", "z a b c d\ne f g"),
        "z a b c\nd e f g"
    );
}

#[test]
fn each_line_of_a_lyric_stream_past_its_last_slot_is_a_measure() {
    assert_eq!(
        formatted(THREE_MEASURES, "W", "a b c d e f g h i j k l m n\no"),
        "a b c d\ne f g h\ni j k l\nm n\no"
    );
}

#[test]
fn placeholders_pushed_past_the_last_slot_by_typed_lyrics_are_deleted() {
    assert_eq!(
        formatted(THREE_MEASURES, "W", "a b c d\ne f g h _\n_\n_\n"),
        "a b c d\ne f g h\n_\n"
    );
}

#[test]
fn a_cursor_among_deleted_placeholders_moves_to_the_last_kept_syllable() {
    assert_eq!(
        formatted_with_cursor(THREE_MEASURES, "W", "a b c d e f g h _ _ _|"),
        "a b c d\ne f g h\n_|"
    );
}

#[test]
fn a_measure_without_lyric_slots_adds_no_blank_line() {
    let source = r#"parts ["Melody" [M]: notes, "Words" [W]: lyrics [M]],

score """
    [M] 1 2 3 4
    [W] a b c d

    [M] 0 0 0 0

    [M] 5 6 7 1
    [W] e f g h
    """,
"#;
    assert_eq!(
        formatted(source, "W", "a b c d e f g h"),
        "a b c d\ne f g h"
    );
}

#[test]
fn a_chord_lyric_stream_starts_a_line_after_each_barline() {
    assert_eq!(
        formatted(CHORDS_AND_WORDS, "W", "(天) 空 好\n(藍) | 我 （在）想 你"),
        "(天) 空 好 (藍) |\n我 （在）想 你"
    );
}

#[test]
fn a_follow_placeholder_keeps_its_own_line() {
    assert_eq!(
        formatted(THREE_MEASURES, "A", "@ @ 1 1 1 1"),
        "@\n@\n1 1 1 1"
    );
}

#[test]
fn notes_past_the_end_of_the_score_become_new_lines() {
    assert_eq!(
        formatted(THREE_MEASURES, "M", "1 2 3 4 5 6 7 1 3 4 5 6 1 2"),
        "1 2 3 4\n5 6 7 1\n3 4 5 6\n1 2"
    );
}

#[test]
fn a_follow_placeholder_past_the_end_of_the_score_is_left_as_typed() {
    let stream = "1 1 1 1 1 1 1 1 1 1 1 1 @";
    assert_eq!(formatted(THREE_MEASURES, "A", stream), stream);
}

#[test]
fn a_leading_extension_is_left_as_typed() {
    assert_eq!(formatted(THREE_MEASURES, "M", " - 1 2 3 4"), " - 1 2 3 4");
}
