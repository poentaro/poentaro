use super::*;

#[test]
fn display_shows_message() {
    let e = IrrecoverableError::new(IrrecoverableErrorKind::InternalInvariant {
        span: Span::new(10, 20),
        detail: "something went wrong".to_string(),
    });
    assert_eq!(format!("{e}"), "error: something went wrong");
}

#[test]
fn with_path_attaches_path() {
    let e = IrrecoverableError::new(IrrecoverableErrorKind::InternalInvariant {
        span: Span::new(0, 1),
        detail: "oops".to_string(),
    })
    .with_path("/tmp/test.poentaro");
    assert_eq!(e.path.unwrap().to_str().unwrap(), "/tmp/test.poentaro");
}

#[test]
fn without_path_is_none() {
    let e = IrrecoverableError::new(IrrecoverableErrorKind::InternalInvariant {
        span: Span::new(0, 1),
        detail: "oops".to_string(),
    });
    assert!(e.path.is_none());
}

#[test]
fn recoverable_error_measure_directives_missing_has_correct_kind() {
    let e = RecoverableError::measure_directives_missing(Span::new(0, 0));
    assert!(matches!(
        e.kind,
        RecoverableErrorKind::MeasureDirectivesMissing
    ));
}

#[test]
fn recoverable_error_source_span_missing_has_correct_kind_and_index() {
    let e = RecoverableError::source_span_missing(Span::new(0, 0), 3);
    assert!(matches!(
        e.kind,
        RecoverableErrorKind::SourceSpanMissing { index: 3 }
    ));
}

#[test]
fn recoverable_error_timed_part_measure_missing_has_correct_kind() {
    let e = RecoverableError::timed_part_measure_missing(Span::new(0, 0));
    assert!(matches!(
        e.kind,
        RecoverableErrorKind::TimedPartMeasureMissing
    ));
}

#[test]
fn recoverable_error_source_span_missing_message_contains_index() {
    let e = RecoverableError::source_span_missing(Span::new(0, 0), 7);
    assert!(e.message().contains("7"));
}

#[test]
fn recoverable_error_measure_directives_missing_message_is_nonempty() {
    let e = RecoverableError::measure_directives_missing(Span::new(0, 0));
    assert!(!e.message().is_empty());
}
