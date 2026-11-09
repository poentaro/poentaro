use crate::error::Span;
use crate::part_stream::{
    apply_part_stream, extract_part_stream, list_streamable_parts, PartStreamApplyResult,
    StreamDiagnostic, StreamDiagnosticSeverity,
};

const TWO_MEASURES: &str = r#"parts ["Melody" [M]: notes, "Words" [W]: lyrics [M]],

score """
    [M] 1 2 3 4
    [W] a b c d

    [M] 5 6 7 1
    [W] e f g h
    """,
"#;

fn with_severity(
    result: &PartStreamApplyResult,
    severity: StreamDiagnosticSeverity,
) -> Vec<&StreamDiagnostic> {
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == severity)
        .collect()
}

fn applied(result: PartStreamApplyResult) -> String {
    assert!(
        with_severity(&result, StreamDiagnosticSeverity::Error).is_empty(),
        "{:?}",
        result.diagnostics
    );
    result.source.unwrap()
}

fn ineligible_reason(source: &str, abbreviation: &str) -> Option<String> {
    list_streamable_parts(source)
        .into_iter()
        .find(|part| part.abbreviation == abbreviation)
        .and_then(|part| part.ineligible_reason)
}

#[test]
fn extracts_cjk_and_latin_lyrics_by_the_targets_slots() {
    let source = r#"parts ["Melody" [M]: notes, "Words" [W]: lyrics [M]],

score """
    [M] 1 2 3 4
    [W] 你 好 世 界

    [M] 5 6 7 1~
    [W] twin-   kle - star

    [M] 1 2 3 4
    [W] up a bove

    [M] 0 0 0 0

    [M] 3 3 3 3
    """,
"#;
    let stream = extract_part_stream(source, "W").unwrap();
    assert_eq!(stream.text, "你 好 世 界\ntwin- kle - star\nup a bove\n_");
    assert_eq!(stream.measure_starts, vec![0, 16, 33, 42, 43]);
    let result = apply_part_stream(source, "W", &stream.text);
    assert_eq!(result.changed_measure_count, 0);
    assert_eq!(result.measure_starts, stream.measure_starts);
    assert_eq!(result.source.as_deref(), Some(source));
}

#[test]
fn cjk_syllables_without_spaces_are_split_by_slots() {
    let result = apply_part_stream(TWO_MEASURES, "W", "一二三四五六七八");
    assert_eq!(result.measure_starts, vec![0, 12]);
    assert_eq!(result.changed_measure_count, 2);
    let source = applied(result);
    assert!(source.contains("[W] 一二三四\n"), "{source}");
    assert!(source.contains("[W] 五六七八\n"), "{source}");
}

#[test]
fn inserting_a_syllable_shifts_later_syllables_into_the_next_measure() {
    let result = apply_part_stream(TWO_MEASURES, "W", "a b c d e f g\nh");
    assert_eq!(result.changed_measure_count, 0);
    let result = apply_part_stream(TWO_MEASURES, "W", "z a b c d e f g");
    assert_eq!(result.changed_measure_count, 2);
    let source = applied(result);
    assert!(
        source.contains("[W] z a b c\n\n    [M] 5 6 7 1\n    [W] d e f g\n"),
        "{source}"
    );
}

#[test]
fn held_dashes_count_as_syllables() {
    let result = apply_part_stream(TWO_MEASURES, "W", "a - b - c d - e");
    let source = applied(result);
    assert!(source.contains("[W] a - b -\n"), "{source}");
    assert!(source.contains("[W] c d - e\n"), "{source}");
}

#[test]
fn a_tie_across_a_barline_takes_no_syllable() {
    let source = r#"parts ["Melody" [M]: notes, "Words" [W]: lyrics [M]],

score """
    [M] 1 2 3 4~
    [W] a b c d

    [M] 4 5 6 7
    [W] e f g
    """,
"#;
    assert_eq!(
        extract_part_stream(source, "W").unwrap().text,
        "a b c d\ne f g"
    );
    let result = apply_part_stream(source, "W", "x a b c d e f");
    assert!(applied(result).contains("[W] x a b c\n\n    [M] 4 5 6 7\n    [W] d e f\n"));
}

#[test]
fn an_underscore_measure_has_no_lyrics() {
    let result = apply_part_stream(TWO_MEASURES, "W", "_ a b c d");
    assert_eq!(result.measure_starts, vec![0, 2]);
    let source = applied(result);
    assert!(
        source.contains("[W] _\n\n    [M] 5 6 7 1\n    [W] a b c d\n"),
        "{source}"
    );
}

#[test]
fn a_measure_without_slots_gets_an_empty_chunk() {
    let source = r#"parts ["Melody" [M]: notes, "Words" [W]: lyrics [M]],

score """
    [M] 1 2 3 4
    [W] a b c d

    [M] 0 0 0 0
    [W] _

    [M] 5 6 7 1
    [W] e f g h
    """,
"#;
    let stream = extract_part_stream(source, "W").unwrap();
    assert_eq!(stream.text, "a b c d\ne f g h");
    assert_eq!(stream.measure_starts, vec![0, 7, 8]);
    let result = apply_part_stream(source, "W", "a b c x e f g");
    assert_eq!(result.changed_measure_count, 2);
    let source = applied(result);
    assert!(
        source.contains("[W] a b c x\n\n    [M] 0 0 0 0\n    [W] _\n\n"),
        "{source}"
    );
    assert!(source.ends_with("[W] e f g\n    \"\"\",\n"), "{source}");
}

#[test]
fn each_line_past_the_last_slot_is_appended_as_a_measure() {
    let result = apply_part_stream(TWO_MEASURES, "W", "a b c d\ne f g h i j\nk\n_\nl m");
    assert_eq!(result.changed_measure_count, 4);
    assert_eq!(result.measure_starts, vec![0, 8, 16, 20, 22, 24]);
    assert_eq!(
        applied(result),
        r#"parts ["Melody" [M]: notes, "Words" [W]: lyrics [M]],

score """
    [M] 1 2 3 4
    [W] a b c d

    [M] 5 6 7 1
    [W] e f g h

    [W] i j

    [W] k

    [W] _

    [W] l m
    """,
"#
    );
}

#[test]
fn placeholders_pushed_past_the_last_slot_are_dropped() {
    let source = r#"parts ["Melody" [M]: notes, "Words" [W]: lyrics [M]],

score """
    [M] 1 2 3 4

    [M] 5 6 7 1
    """,
"#;
    let result = apply_part_stream(source, "W", "a b c d _ _");
    assert_eq!(result.diagnostics, vec![]);
    assert_eq!(
        applied(result),
        r#"parts ["Melody" [M]: notes, "Words" [W]: lyrics [M]],

score """
    [M] 1 2 3 4
    [W] a b c d

    [M] 5 6 7 1
    """,
"#
    );
}

#[test]
fn a_placeholder_past_the_last_slot_is_a_whole_measure_when_syllables_follow_it() {
    let source = applied(apply_part_stream(TWO_MEASURES, "W", "a b c d e f g h _ i"));
    assert!(
        source.ends_with("[W] e f g h\n\n    [W] _\n\n    [W] i\n    \"\"\",\n"),
        "{source}"
    );
}

#[test]
fn a_short_last_measure_is_only_a_warning() {
    let result = apply_part_stream(TWO_MEASURES, "W", "b c d e f g h");
    let warnings = with_severity(&result, StreamDiagnosticSeverity::Warning);
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].span, Span::new(8, 13));
    assert!(warnings[0].message.contains("3 of its 4 syllables"));
    let source = applied(result);
    assert!(
        source.contains("[W] b c d e\n\n    [M] 5 6 7 1\n    [W] f g h\n"),
        "{source}"
    );
}

#[test]
fn a_stream_ending_at_a_barline_removes_later_lyric_lines() {
    let source = applied(apply_part_stream(TWO_MEASURES, "W", "a b c d"));
    assert!(
        source.ends_with("[W] a b c d\n\n    [M] 5 6 7 1\n    \"\"\",\n"),
        "{source}"
    );
}

#[test]
fn lyrics_under_a_follow_part_use_its_copied_notes() {
    let source = r#"parts ["Soprano" [S]: notes, "Alto" [A]: follow [S], "Words" [W]: lyrics [A]],

score """
    [S] 1 2 3 4
    [W] a b c d

    [S] 1 2 3 4
    [A] 5 - - -
    [W] e
    """,
"#;
    assert_eq!(ineligible_reason(source, "W"), None);
    assert_eq!(extract_part_stream(source, "W").unwrap().text, "a b c d\ne");
}

#[test]
fn a_lyric_tally_mismatch_makes_the_part_ineligible_except_for_underscore() {
    let mismatched = TWO_MEASURES.replace("[W] e f g h", "[W] e f g");
    let reason = ineligible_reason(&mismatched, "W").unwrap();
    assert!(reason.contains("measure 2"), "{reason}");
    assert!(reason.contains("3 syllables for 4 notes"), "{reason}");

    let skipped = TWO_MEASURES.replace("[W] e f g h", "[W] _");
    assert_eq!(ineligible_reason(&skipped, "W"), None);
    assert_eq!(
        extract_part_stream(&skipped, "W").unwrap().text,
        "a b c d\n_"
    );
}
