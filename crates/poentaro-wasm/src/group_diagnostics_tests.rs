use crate::diagnostics::group_diagnostics_into_view_zones;
use crate::types::{DiagnosticOut, DiagnosticSeverity, SpanOut};

fn make_diagnostic(severity: DiagnosticSeverity, message: &str, span_end: usize) -> DiagnosticOut {
    DiagnosticOut {
        severity,
        message: message.to_string(),
        span: SpanOut {
            start: 0,
            end: span_end,
        },
    }
}

#[test]
fn single_error_produces_one_error_zone() {
    // "line1\nline2\n" — byte offset 10 is on line 2
    let source = "line1\nline2\n";
    let diagnostics = vec![make_diagnostic(DiagnosticSeverity::Error, "oops", 10)];
    let zones = group_diagnostics_into_view_zones(source, &diagnostics);
    assert_eq!(zones.len(), 1);
    assert_eq!(zones[0].severity, DiagnosticSeverity::Error);
    assert_eq!(zones[0].after_line_number, 2);
    assert_eq!(zones[0].messages.len(), 1);
    assert_eq!(zones[0].messages[0].message, "oops");
}

#[test]
fn single_warning_produces_one_warning_zone() {
    let source = "line1\n";
    let diagnostics = vec![make_diagnostic(DiagnosticSeverity::Warning, "note", 4)];
    let zones = group_diagnostics_into_view_zones(source, &diagnostics);
    assert_eq!(zones.len(), 1);
    assert_eq!(zones[0].severity, DiagnosticSeverity::Warning);
    assert_eq!(zones[0].after_line_number, 1);
}

#[test]
fn two_errors_same_line_merge_into_one_zone() {
    let source = "line1\nline2\n";
    let diagnostics = vec![
        make_diagnostic(DiagnosticSeverity::Error, "first", 8),
        make_diagnostic(DiagnosticSeverity::Error, "second", 10),
    ];
    let zones = group_diagnostics_into_view_zones(source, &diagnostics);
    assert_eq!(zones.len(), 1);
    assert_eq!(zones[0].messages.len(), 2);
    assert_eq!(zones[0].messages[0].message, "first");
    assert_eq!(zones[0].messages[1].message, "second");
}

#[test]
fn error_and_warning_on_same_line_produce_two_zones_error_first() {
    let source = "line1\nline2\n";
    let diagnostics = vec![
        make_diagnostic(DiagnosticSeverity::Warning, "warn", 8),
        make_diagnostic(DiagnosticSeverity::Error, "err", 10),
    ];
    let zones = group_diagnostics_into_view_zones(source, &diagnostics);
    assert_eq!(zones.len(), 2);
    assert_eq!(zones[0].severity, DiagnosticSeverity::Error);
    assert_eq!(zones[1].severity, DiagnosticSeverity::Warning);
    assert_eq!(zones[0].after_line_number, 2);
    assert_eq!(zones[1].after_line_number, 2);
}

#[test]
fn zones_sorted_by_line_number_ascending() {
    let source = "a\nb\nc\n";
    let diagnostics = vec![
        make_diagnostic(DiagnosticSeverity::Error, "line3", 5),
        make_diagnostic(DiagnosticSeverity::Error, "line1", 1),
    ];
    let zones = group_diagnostics_into_view_zones(source, &diagnostics);
    assert_eq!(zones.len(), 2);
    assert!(zones[0].after_line_number < zones[1].after_line_number);
}

#[test]
fn empty_diagnostics_returns_empty_zones() {
    let zones = group_diagnostics_into_view_zones("source", &[]);
    assert!(zones.is_empty());
}
