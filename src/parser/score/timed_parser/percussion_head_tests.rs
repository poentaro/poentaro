use crate::ast::parsed::{ParsedPercussionHit, ParsedRest, ScoreEvent};
use crate::error::{Diagnostic, RecoverableErrorKind, Span, Spanned};
use crate::parser::score::timed_parser::{
    parse_timed_line, EventAttrs, GroupStack, LexContext, ParseHeadError, PercussionHead,
    TimedUnitHead,
};

fn parse_events(input: &str) -> Vec<Spanned<ScoreEvent>> {
    parse_timed_line::<PercussionHead>(input, 0, &mut GroupStack::default(), LexContext::Percussion)
        .unwrap()
        .events
}

fn hit(events: &[Spanned<ScoreEvent>], i: usize) -> &ParsedPercussionHit {
    match &events[i].value {
        ScoreEvent::PercussionHit(h) => h,
        other => panic!("expected PercussionHit at index {i}, got: {other:?}"),
    }
}

fn rest(events: &[Spanned<ScoreEvent>], i: usize) -> &ParsedRest {
    match &events[i].value {
        ScoreEvent::Rest(r) => r,
        other => panic!("expected Rest at index {i}, got: {other:?}"),
    }
}

#[test]
fn parse_head_returns_recoverable_for_pitch_digit() {
    let chars: Vec<char> = "5".chars().collect();
    let span = Span::new(2, 3);
    let result = PercussionHead::parse_head(&chars, 0, &span);
    let Err(ParseHeadError::Recoverable(Some(diagnostic))) = result else {
        panic!("expected Err(ParseHeadError::Recoverable(Some(...))), got: {result:?}");
    };
    assert!(
        matches!(
            &diagnostic,
            Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::PercussionExpectedHitOrRest { ch: '5' })
        ),
        "expected PercussionExpectedHitOrRest error diagnostic, got: {diagnostic:?}"
    );
}

#[test]
fn parse_head_returns_recoverable_for_empty_input() {
    let chars: Vec<char> = Vec::new();
    let span = Span::new(0, 0);
    let result = PercussionHead::parse_head(&chars, 0, &span);
    let Err(ParseHeadError::Recoverable(Some(diagnostic))) = result else {
        panic!("expected Err(ParseHeadError::Recoverable(Some(...))), got: {result:?}");
    };
    assert!(
        matches!(
            &diagnostic,
            Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::PercussionExpectedHitOrRest { ch: '\0' })
        ),
        "expected PercussionExpectedHitOrRest error diagnostic, got: {diagnostic:?}"
    );
}

#[test]
fn parses_hit() {
    let chars: Vec<char> = "x".chars().collect();
    let span = Span::new(0, 1);
    let (head, next, is_rest, _diags) =
        PercussionHead::parse_head(&chars, 0, &span).expect("parse_head should succeed");
    assert_eq!(next, 1);
    assert!(!is_rest);
    let event = PercussionHead::to_event(
        &head,
        EventAttrs {
            duration: 4,
            dotted: false,
            double_dotted: false,
            fermata: false,
            octave: 0,
            group_membership: 0,
            group_continuation: 0,
            tuplet: None,
        },
    );
    assert!(matches!(event, ScoreEvent::PercussionHit(_)));
}

#[test]
fn parses_rest() {
    let chars: Vec<char> = "0".chars().collect();
    let span = Span::new(0, 1);
    let (head, next, is_rest, _diags) =
        PercussionHead::parse_head(&chars, 0, &span).expect("parse_head should succeed");
    assert_eq!(next, 1);
    assert!(is_rest);
    let event = PercussionHead::to_event(
        &head,
        EventAttrs {
            duration: 4,
            dotted: false,
            double_dotted: false,
            fermata: false,
            octave: 0,
            group_membership: 0,
            group_continuation: 0,
            tuplet: None,
        },
    );
    assert!(matches!(event, ScoreEvent::Rest(_)));
}

#[test]
fn allows_octave_suffixes_is_false() {
    assert!(!PercussionHead::allows_octave_suffixes());
}

#[test]
fn parses_full_beat_hit() {
    let events = parse_events("x");
    assert_eq!(hit(&events, 0).duration, 4);
}

#[test]
fn parses_dash_extension_suffix() {
    let events = parse_events("x---");
    assert_eq!(hit(&events, 0).duration, 16);
}

#[test]
fn parses_dotted_hit() {
    let events = parse_events("x.");
    let h = hit(&events, 0);
    assert_eq!(h.duration, 6);
    assert!(h.dotted);
}

#[test]
fn parses_repeat_atom_after_hit() {
    let events = parse_events("x r");
    assert_eq!(events.len(), 2);
    hit(&events, 0);
    hit(&events, 1);
}

#[test]
fn parses_tie_between_hits() {
    let events = parse_events("(xx)");
    assert_eq!(events.len(), 2);
    assert!(hit(&events, 0).slur);
    assert!(!hit(&events, 1).slur);
}

#[test]
fn parses_tilde_tie_between_hits() {
    let events = parse_events("x~x");
    assert_eq!(events.len(), 2);
    assert!(hit(&events, 0).tie_to_next());
}

#[test]
fn rest_is_unaffected_by_percussion_context() {
    let events = parse_events("0");
    assert_eq!(rest(&events, 0).duration, 4);
}

#[test]
fn rejects_pitch_digits() {
    use crate::error::Diagnostic;
    for input in &["1", "2", "3", "4", "5", "6", "7"] {
        let result = parse_timed_line::<PercussionHead>(
            input,
            0,
            &mut GroupStack::default(),
            LexContext::Percussion,
        )
        .unwrap();
        assert!(
            result.chord_errors.iter().any(|d| matches!(
                d,
                Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::PercussionExpectedHitOrRest { .. })
            )),
            "expected PercussionExpectedHitOrRest error for input {input}"
        );
    }
}
