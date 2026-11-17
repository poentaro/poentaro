use crate::error::Span;
use crate::part_stream::{
    apply_part_stream, extract_part_stream, list_streamable_parts, PartStreamApplyResult,
    StreamDiagnosticSeverity,
};

const TWO_MEASURES: &str = r#"parts ["Chords" [C]: chords, "Words" [W]: lyrics [C]],

score """
    [C] 1 - 4 -
    [W] (天) 空  好 (藍)

    [C] 0 5 - -
    [W] 我 （在）想 你
    """,
"#;

fn spans_with_severity(
    result: &PartStreamApplyResult,
    severity: StreamDiagnosticSeverity,
) -> Vec<(Span, String)> {
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == severity)
        .map(|diagnostic| (diagnostic.span, diagnostic.message.clone()))
        .collect()
}

fn errors(result: &PartStreamApplyResult) -> Vec<(Span, String)> {
    spans_with_severity(result, StreamDiagnosticSeverity::Error)
}

fn applied(result: PartStreamApplyResult) -> String {
    assert_eq!(errors(&result), vec![]);
    result.source.unwrap()
}

#[test]
fn lyrics_under_a_chords_part_are_eligible() {
    let words = list_streamable_parts(TWO_MEASURES)
        .into_iter()
        .find(|part| part.abbreviation == "W")
        .unwrap();
    assert_eq!(words.ineligible_reason, None);
}

#[test]
fn extracts_one_measure_per_line_ended_by_a_barline() {
    let stream = extract_part_stream(TWO_MEASURES, "W").unwrap();
    assert_eq!(stream.text, "(天) 空 好 (藍) |\n我 （在）想 你");
    assert_eq!(stream.measure_starts, vec![0, 22]);
    let result = apply_part_stream(TWO_MEASURES, "W", &stream.text);
    assert_eq!(result.changed_measure_count, 0);
    assert_eq!(result.measure_starts, stream.measure_starts);
    assert_eq!(result.source.as_deref(), Some(TWO_MEASURES));
}

#[test]
fn a_measure_the_part_does_not_write_shows_as_underscore() {
    let source = r#"parts ["Chords" [C]: chords, "Words" [W]: lyrics [C]],

score """
    [C] 1 - - -
    [W] (a) b

    [C] 5 - - -
    """,
"#;
    let stream = extract_part_stream(source, "W").unwrap();
    assert_eq!(stream.text, "(a) b |\n_");
    let result = apply_part_stream(source, "W", &stream.text);
    assert_eq!(result.source.as_deref(), Some(source));
}

#[test]
fn moving_a_barline_moves_syllables_between_measures() {
    let result = apply_part_stream(TWO_MEASURES, "W", "(天) 空 好 | 藍 我 （在）想 你");
    assert_eq!(result.measure_starts, vec![0, 16]);
    assert_eq!(result.changed_measure_count, 2);
    assert_eq!(
        applied(result),
        r#"parts ["Chords" [C]: chords, "Words" [W]: lyrics [C]],

score """
    [C] 1 - 4 -
    [W] (天) 空 好

    [C] 0 5 - -
    [W] 藍 我 （在）想 你
    """,
"#
    );
}

#[test]
fn a_trailing_barline_does_not_open_another_measure() {
    let stream = "(天) 空 好 (藍) |\n我 （在）想 你 |\n";
    let result = apply_part_stream(TWO_MEASURES, "W", stream);
    assert_eq!(result.changed_measure_count, 0);
    assert_eq!(result.source.as_deref(), Some(TWO_MEASURES));
}

#[test]
fn more_brackets_than_chord_attacks_is_an_error_on_that_measure() {
    let result = apply_part_stream(TWO_MEASURES, "W", "(天) (空) (好) | 我 （在）想 你");
    assert_eq!(
        errors(&result),
        vec![(
            Span::new(0, 17),
            "lyrics under a chord part need at most one bracketed item per chord attack, \
             found 3 bracketed items for 2 chord attacks"
                .to_string()
        )]
    );
    assert_eq!(result.source, None);
}

#[test]
fn an_unbracketed_first_syllable_takes_a_beat_one_chord_attack() {
    let result = apply_part_stream(TWO_MEASURES, "W", "天 空 好 (藍) | 我 （在）想 你");
    assert_eq!(errors(&result), vec![]);
}

#[test]
fn an_unbracketed_first_syllable_on_beat_one_counts_toward_the_chord_attacks() {
    let result = apply_part_stream(TWO_MEASURES, "W", "天 (空) (好) | 我 （在）想 你");
    assert_eq!(
        errors(&result),
        vec![(
            Span::new(0, 15),
            "lyrics under a chord part need at most one bracketed item per chord attack, \
             found 2 bracketed items for 2 chord attacks; \
             the unbracketed first syllable already takes the beat-1 chord"
                .to_string()
        )]
    );
}

#[test]
fn bracket_mistakes_are_spanned_in_the_stream() {
    let result = apply_part_stream(TWO_MEASURES, "W", "(天) 空 | 我 (在 想 你");
    assert_eq!(
        errors(&result),
        vec![(
            Span::new(16, 17),
            "lyric bracket is never closed".to_string()
        )]
    );
}

#[test]
fn an_empty_measure_is_an_error() {
    let result = apply_part_stream(TWO_MEASURES, "W", "(天) 空 | |");
    assert_eq!(
        errors(&result),
        vec![(
            Span::new(12, 13),
            "this measure is empty; write `_` for a measure without lyrics".to_string()
        )]
    );
}

#[test]
fn measures_past_the_end_of_the_score_are_appended() {
    let result = apply_part_stream(
        TWO_MEASURES,
        "W",
        "(天) 空 好 (藍) | 我 （在）想 你 | 天 空 | _ | 好 藍",
    );
    assert_eq!(result.changed_measure_count, 3);
    assert_eq!(
        applied(result),
        r#"parts ["Chords" [C]: chords, "Words" [W]: lyrics [C]],

score """
    [C] 1 - 4 -
    [W] (天) 空  好 (藍)

    [C] 0 5 - -
    [W] 我 （在）想 你

    [W] 天 空

    [W] _

    [W] 好 藍
    """,
"#
    );
}

#[test]
fn a_bracket_past_the_end_of_the_score_has_no_chord_attack() {
    let stream = "(天) 空 好 (藍) | 我 （在）想 你 | (天)";
    let result = apply_part_stream(TWO_MEASURES, "W", stream);
    let errors = errors(&result);
    let appended_start = stream.rfind('|').unwrap() + 2;
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].0, Span::new(appended_start, stream.len()));
    assert!(
        errors[0].1.contains("1 bracketed item for 0 chord attacks"),
        "{}",
        errors[0].1
    );
}

#[test]
fn a_shorter_stream_warns_and_drops_the_later_measures_lyrics() {
    let result = apply_part_stream(TWO_MEASURES, "W", "(天) 空 好 (藍)");
    assert_eq!(
        spans_with_severity(&result, StreamDiagnosticSeverity::Warning),
        vec![(
            Span::new(0, 19),
            "the stream ends at measure 1 of 2; the measures after it will have no lyrics"
                .to_string()
        )]
    );
    assert_eq!(
        applied(result),
        r#"parts ["Chords" [C]: chords, "Words" [W]: lyrics [C]],

score """
    [C] 1 - 4 -
    [W] (天) 空  好 (藍)

    [C] 0 5 - -
    """,
"#
    );
}

#[test]
fn underscore_writes_a_measure_without_lyrics() {
    let result = apply_part_stream(TWO_MEASURES, "W", "(天) 空 好 (藍) | _");
    assert!(applied(result).ends_with("[C] 0 5 - -\n    [W] _\n    \"\"\",\n"));
}

#[test]
fn follow_placeholders_are_still_errors() {
    let result = apply_part_stream(TWO_MEASURES, "W", "(天) @ | 我");
    assert_eq!(
        errors(&result),
        vec![(
            Span::new(6, 7),
            "`@` is only allowed in follow parts".to_string()
        )]
    );
}

#[test]
fn a_source_tally_error_makes_the_part_ineligible() {
    let source = TWO_MEASURES.replace("[W] (天) 空  好 (藍)", "[W] (天) (空) (好)");
    let reason = list_streamable_parts(&source)
        .into_iter()
        .find(|part| part.abbreviation == "W")
        .and_then(|part| part.ineligible_reason)
        .unwrap();
    assert!(
        reason.contains("3 bracketed items for 2 chord attacks"),
        "{reason}"
    );
}
