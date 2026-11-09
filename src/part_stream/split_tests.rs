use crate::error::Span;
use crate::part_stream::{apply_part_stream, PartStreamApplyResult, StreamDiagnosticSeverity};

const ONE_MEASURE: &str = r#"parts ["Melody" [M]: notes],

score """
    [M] 1 2 3 4
    """,
"#;

fn apply(stream: &str) -> PartStreamApplyResult {
    apply_part_stream(ONE_MEASURE, "M", stream)
}

fn error_spans(result: &PartStreamApplyResult) -> Vec<Span> {
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == StreamDiagnosticSeverity::Error)
        .map(|diagnostic| diagnostic.span)
        .collect()
}

fn error_messages(result: &PartStreamApplyResult) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.clone())
        .collect()
}

#[test]
fn a_triplet_fills_one_beat() {
    let result = apply("3:{1_1_1_} 2 3 4 5");
    assert_eq!(error_spans(&result), vec![]);
    assert_eq!(result.measure_starts, vec![0, 17]);
    assert_eq!(
        result.source.unwrap(),
        r#"parts ["Melody" [M]: notes],

score """
    [M] 3:{1_1_1_} 2 3 4

    [M] 5 0 0 0
    """,
"#
    );
}

#[test]
fn a_tuplet_crossing_a_barline_is_an_error() {
    let stream = "1 1 1_ 2:{1 1}";
    let result = apply(stream);
    assert_eq!(result.source, None);
    assert_eq!(error_spans(&result), vec![Span::new(10, 13)]);
    assert!(error_messages(&result)[0].contains("tuplet crosses the barline of measure 1"));
}

#[test]
fn a_cluster_crossing_a_barline_is_an_error_spanning_the_cluster() {
    let result = apply("1 2 3 4 - 5");
    assert_eq!(result.source, None);
    assert_eq!(error_spans(&result), vec![Span::new(6, 9)]);
    assert!(error_messages(&result)[0].contains("crosses the barline of measure 1"));
}

#[test]
fn an_opening_slur_moves_to_the_next_measure() {
    let result = apply("1 2 3 4 (5 6) 7 1");
    assert_eq!(result.measure_starts, vec![0, 8]);
    assert!(result
        .source
        .unwrap()
        .contains("[M] 1 2 3 4\n\n    [M] (5 6) 7 1\n"));
}

#[test]
fn a_closing_slur_and_a_tie_stay_with_the_measure_before() {
    let result = apply("1 2 3 (4 5) 6 7 1~ 1 0 0 0");
    assert_eq!(error_spans(&result), vec![]);
    let source = result.source.unwrap();
    assert!(
        source.contains("[M] 1 2 3 (4\n\n    [M] 5) 6 7 1~\n\n    [M] 1 0 0 0\n"),
        "{source}"
    );
}

#[test]
fn barlines_are_errors() {
    let result = apply("1 2 3 4 | 5 6 7 1");
    assert_eq!(result.source, None);
    assert_eq!(error_spans(&result), vec![Span::new(8, 9)]);
    assert!(error_messages(&result)[0].contains("barlines are not allowed"));
}

#[test]
fn inline_directives_are_errors() {
    let result = apply("1 2 bpm=90 3 4");
    assert_eq!(result.source, None);
    assert_eq!(error_spans(&result).len(), 1);
    assert!(error_messages(&result)[0].contains("directives are not allowed"));
}

#[test]
fn a_new_line_every_four_measures_does_not_affect_measure_starts() {
    let result = apply("1 2 3 4\n  5 6 7 1");
    assert_eq!(result.measure_starts, vec![0, 10]);
}

#[test]
fn an_unclosed_slur_is_an_error() {
    let result = apply("1 (2 3 4");
    assert_eq!(error_spans(&result), vec![Span::new(2, 3)]);
    assert_eq!(result.source, None);
}

#[test]
fn a_leading_extension_is_an_error() {
    let result = apply("- 1 2 3 4");
    assert_eq!(result.source, None);
    assert_eq!(error_spans(&result), vec![Span::new(0, 1)]);
    assert!(error_messages(&result)[0].contains("no note before it to extend"));
}

#[test]
fn a_stream_of_only_extensions_is_an_error_rather_than_deleting_the_part() {
    let result = apply("- -");
    assert_eq!(result.source, None);
    assert_eq!(error_spans(&result), vec![Span::new(0, 1), Span::new(2, 3)]);
}
