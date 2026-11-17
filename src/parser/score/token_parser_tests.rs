use super::*;
use crate::ast::parsed::{NumberedPitch, ParsedNote, ParsedRest, ScoreEvent};
use crate::error::Spanned;

fn parse(input: &str) -> Result<TimedLineParse, IrrecoverableError> {
    parse_notes_line(input, 0, &mut GroupStack::default())
}

fn parse_with_state(
    input: &str,
    state: &mut GroupStack,
) -> Result<TimedLineParse, IrrecoverableError> {
    parse_notes_line(input, 0, state)
}

fn parse_events(input: &str) -> Vec<Spanned<ScoreEvent>> {
    parse(input).unwrap().events
}

fn note(events: &[Spanned<ScoreEvent>], i: usize) -> &ParsedNote {
    match &events[i].value {
        ScoreEvent::Note(n) => n,
        _ => panic!("expected Note at index {i}"),
    }
}

fn rest(events: &[Spanned<ScoreEvent>], i: usize) -> &ParsedRest {
    match &events[i].value {
        ScoreEvent::Rest(r) => r,
        _ => panic!("expected Rest at index {i}"),
    }
}

#[test]
fn parses_full_beat_note() {
    let events = parse_events("1");
    let n = note(&events, 0);
    assert_eq!(n.pitch, NumberedPitch::One);
    assert_eq!(n.duration, 4);
    assert_eq!(n.octave, 0);
    assert!(!n.slur);
}

#[test]
fn parses_half_beat_note() {
    let events = parse_events("3_");
    let n = note(&events, 0);
    assert_eq!(n.pitch, NumberedPitch::Three);
    assert_eq!(n.duration, 2);
}

#[test]
fn parses_quarter_beat_note() {
    let events = parse_events("5=");
    let n = note(&events, 0);
    assert_eq!(n.pitch, NumberedPitch::Five);
    assert_eq!(n.duration, 1);
}

#[test]
fn parses_octave_up() {
    let events = parse_events("1'");
    let n = note(&events, 0);
    assert_eq!(n.octave, 1);
}

#[test]
fn parses_two_octaves_up() {
    let events = parse_events("1''");
    let n = note(&events, 0);
    assert_eq!(n.octave, 2);
}

#[test]
fn parses_octave_down() {
    let events = parse_events("1,");
    let n = note(&events, 0);
    assert_eq!(n.octave, -1);
}

#[test]
fn parses_two_octaves_down() {
    let events = parse_events("1,,");
    let n = note(&events, 0);
    assert_eq!(n.octave, -2);
}

#[test]
fn mixed_octave_markers_recoverable_with_zero_octave() {
    use crate::error::{Diagnostic, RecoverableErrorKind};
    let result = parse("1',").expect("mixed octave markers must not abort");
    assert_eq!(result.events.len(), 1, "note must still be emitted");
    let note_event = note(&result.events, 0);
    assert_eq!(
        note_event.octave, 0,
        "octave must be zeroed on mixed markers"
    );
    assert!(
        result.chord_errors.iter().any(|d| matches!(
            d,
            Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::DurationMixedOctaveMarkers)
        )),
        "expected DurationMixedOctaveMarkers error"
    );
}

#[test]
fn parses_tie_group() {
    let events = parse_events("(23)");
    assert_eq!(events.len(), 2);
    assert!(note(&events, 0).slur);
    assert!(!note(&events, 1).slur);
}

#[test]
fn parses_concatenated_notes() {
    let events = parse_events("505");
    assert_eq!(events.len(), 3);
}

#[test]
fn parses_standalone_extension() {
    let events = parse_events("2 - - -");
    assert!(matches!(events[0].value, ScoreEvent::Note(_)));
    assert!(matches!(events[1].value, ScoreEvent::Extension { .. }));
    assert!(matches!(events[2].value, ScoreEvent::Extension { .. }));
    assert!(matches!(events[3].value, ScoreEvent::Extension { .. }));
}

#[test]
fn parses_extension_suffix() {
    let events = parse_events("1---");
    let n = note(&events, 0);
    assert_eq!(n.duration, 16);
}

#[test]
fn parses_rest() {
    let events = parse_events("0");
    let r = rest(&events, 0);
    assert_eq!(r.duration, 4);
}

#[test]
fn parses_half_beat_rest() {
    let events = parse_events("0_");
    let r = rest(&events, 0);
    assert_eq!(r.duration, 2);
}

#[test]
fn parses_sequence() {
    let events = parse_events("1 2_ 3");
    assert_eq!(events.len(), 3);
}

#[test]
fn parses_dotted_half_beat_note() {
    let events = parse_events("1_.");
    let n = note(&events, 0);
    assert_eq!(n.duration, 3);
    assert!(n.dotted);
}

#[test]
fn parses_dotted_full_beat_note() {
    let events = parse_events("1.");
    let n = note(&events, 0);
    assert_eq!(n.duration, 6);
    assert!(n.dotted);
}

#[test]
fn parses_dotted_note_with_lower_octave() {
    let events = parse_events("1_,.");
    let n = note(&events, 0);
    assert_eq!(n.duration, 3);
    assert_eq!(n.octave, -1);
    assert!(n.dotted);
}

#[test]
fn parses_dotted_half_beat_rest() {
    let events = parse_events("0_.");
    let r = rest(&events, 0);
    assert_eq!(r.duration, 3);
    assert!(r.dotted);
}

#[test]
fn dash_suffix_extends_rest() {
    let parsed = parse("0---").unwrap();
    assert_eq!(parsed.events.len(), 1);
    assert_eq!(rest(&parsed.events, 0).duration, 16);
}

#[test]
fn dash_suffix_extends_rest_in_group() {
    let parsed = parse("(0-1)").unwrap();
    assert_eq!(parsed.events.len(), 2);
    assert_eq!(rest(&parsed.events, 0).duration, 8);
}

#[test]
fn parses_repeated_quarter_rests() {
    let events = parse_events("0 0 0 0");
    assert_eq!(events.len(), 4);
    for i in 0..4 {
        assert_eq!(rest(&events, i).duration, 4);
    }
}

#[test]
fn dotted_quarter_beat_note_is_recoverable() {
    use crate::error::{Diagnostic, RecoverableErrorKind};
    let result = parse("1=.").expect("dotted quarter-beat must not be irrecoverable");
    assert_eq!(result.events.len(), 1);
    assert_eq!(note(&result.events, 0).duration, 1);
    assert!(
        result.chord_errors.iter().any(|d| matches!(
            d,
            Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::DurationCannotDotQuarterBeat)
        )),
        "expected DurationCannotDotQuarterBeat error, got: {:?}",
        result.chord_errors.iter().map(|d| d.message()).collect::<Vec<_>>()
    );
}

#[test]
fn non_dotted_note_has_dotted_false() {
    let events = parse_events("1_");
    let n = note(&events, 0);
    assert!(!n.dotted);
}

#[test]
fn single_note_group_emits_warning() {
    use crate::error::{Diagnostic, WarningKind};
    for input in &["(3)", "(5)"] {
        let result = parse(input).expect("should not be irrecoverable");
        assert_eq!(
            result.events.len(),
            1,
            "note should still be emitted for {input}"
        );
        assert!(
            result.chord_errors.iter().any(|d| matches!(
                d,
                Diagnostic::Warning(w) if matches!(w.kind, WarningKind::GroupTooFewNotes)
            )),
            "expected GroupTooFewNotes warning for {input}"
        );
    }
}

#[test]
fn single_note_cross_measure_group_emits_warning() {
    use crate::error::{Diagnostic, WarningKind};
    let mut state = GroupStack::default();
    parse_with_state("(1", &mut state).unwrap();
    let result = parse_with_state(")", &mut state).expect("should not be irrecoverable");
    assert!(
        result.chord_errors.iter().any(|d| matches!(
            d,
            Diagnostic::Warning(w) if matches!(w.kind, WarningKind::GroupTooFewNotes)
        )),
        "expected GroupTooFewNotes warning"
    );
}

#[test]
fn parses_group_followed_by_notes() {
    let events = parse_events("(12)31");
    assert_eq!(events.len(), 4);
    assert!(note(&events, 0).slur);
    assert!(!note(&events, 1).slur);
}

#[test]
fn parses_open_group_at_end_of_token() {
    let mut state = GroupStack::default();
    let events = parse_with_state("111(1", &mut state).unwrap().events;
    assert_eq!(events.len(), 4);
    assert!(note(&events, 3).slur);
    assert!(state.is_open());
}

#[test]
fn parses_cross_measure_group_continuation() {
    let mut state = GroupStack::default();
    // Open a group with one note so state.is_open() is true
    parse_with_state("(1", &mut state).unwrap();
    let events = parse_with_state("2)345", &mut state).unwrap().events;
    assert_eq!(events.len(), 4);
    assert!(!note(&events, 0).slur);
    assert!(!state.is_open());
}

#[test]
fn cross_measure_group_sets_tie_on_opening_note() {
    let mut state = GroupStack::default();
    parse_with_state("111(1", &mut state).unwrap();
    let events = parse_with_state("2)345", &mut state).unwrap().events;
    assert!(note(&events, 0).pitch == NumberedPitch::Two);
    assert!(!note(&events, 0).slur);
}

#[test]
fn open_group_continues_across_spaced_tokens_in_same_measure() {
    let mut state = GroupStack::default();
    parse_with_state("(6", &mut state).unwrap();
    parse_with_state("-", &mut state).unwrap();
    let events = parse_with_state("7", &mut state).unwrap().events;
    assert!(state.is_open());
    assert_eq!(events.len(), 1);
    assert!(note(&events, 0).pitch == NumberedPitch::Seven);
    assert!(note(&events, 0).slur);
}

#[test]
fn parses_nested_tie_group() {
    let mut state = GroupStack::default();
    let events1 = parse_with_state("(3=", &mut state).unwrap().events;
    assert_eq!(events1.len(), 1);
    assert!(state.is_open());
    let events2 = parse_with_state("(2_1_))", &mut state).unwrap().events;
    assert_eq!(events2.len(), 2);
    assert!(!state.is_open());
    let events = parse_events("(3= (2_1_))");
    assert_eq!(events.len(), 3);
    let n0 = note(&events, 0);
    assert_eq!(n0.pitch, NumberedPitch::Three);
    assert_eq!(n0.duration, 1);
    assert_eq!(n0.group_membership, 1);
    assert_eq!(n0.group_continuation, 1);
    let n1 = note(&events, 1);
    assert_eq!(n1.pitch, NumberedPitch::Two);
    assert_eq!(n1.duration, 2);
    assert_eq!(n1.group_membership, 2);
    assert_eq!(n1.group_continuation, 2);
    let n2 = note(&events, 2);
    assert_eq!(n2.pitch, NumberedPitch::One);
    assert_eq!(n2.duration, 2);
    assert_eq!(n2.group_membership, 2);
    assert_eq!(n2.group_continuation, 0);
}

#[test]
fn open_group_closes_on_spaced_tokens_across_measures() {
    let mut state = GroupStack::default();
    parse_with_state("(6", &mut state).unwrap();
    parse_with_state("-", &mut state).unwrap();
    parse_with_state("7", &mut state).unwrap();
    parse_with_state("-", &mut state).unwrap();
    let events = parse_with_state("7)", &mut state).unwrap().events;
    assert!(!state.is_open());
    assert_eq!(events.len(), 1);
    assert!(note(&events, 0).pitch == NumberedPitch::Seven);
    assert!(!note(&events, 0).slur);
}

#[path = "token_parser_double_dot_tests.rs"]
mod double_dot_tests;
