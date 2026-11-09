use super::*;
use crate::ast::parsed::NumberedPitch;

#[test]
fn suffix_dash_after_rest_is_allowed() {
    // `0---` is conventional numbered notation for a rest held across the whole measure.
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["a": notes],

score """
    time=4/4 key=C4 bpm=120
    [a] 0---
    """,
"#,
    );
    assert_eq!(
        score.measures[0].diagnostics.len(),
        0,
        "0--- should parse without error, got {:?}",
        score.measures[0].diagnostics
    );
    let notes = first_part_notes(&score, 0);
    assert_eq!(notes.len(), 1);
    match &notes[0] {
        NoteEvent::Rest(r) => assert_eq!(r.duration, 16, "0--- should hold the rest for 4 beats"),
        NoteEvent::Note(_) | NoteEvent::Chord(_) | NoteEvent::Percussion(_) => {
            panic!("expected Rest")
        }
    }
}

#[test]
fn standalone_dash_after_rest_is_allowed() {
    // `0 - - -` is the space-separated equivalent of `0---`.
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 0 - - -
    [Melodyv1] _
    """,
"#,
    );
    assert_eq!(
        score.measures[0].diagnostics.len(),
        0,
        "0 - - - should parse without error, got {:?}",
        score.measures[0].diagnostics
    );
    let notes = first_part_notes(&score, 0);
    assert_eq!(notes.len(), 1);
    match &notes[0] {
        NoteEvent::Rest(r) => assert_eq!(r.duration, 16),
        NoteEvent::Note(_) | NoteEvent::Chord(_) | NoteEvent::Percussion(_) => {
            panic!("expected Rest")
        }
    }
}

#[test]
fn standalone_tie_marker_after_extension_that_flushes_measure() {
    // `(6---` fills a 4/4 measure exactly; `7)` closes the cross-measure group.
    // The outgoing tie on 6 must carry into the next measure.
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] (6---

    [Melody] 7) 0 0 0
    """,
"#,
    );
    let notes_m0 = first_part_notes(&score, 0);
    match notes_m0.last().unwrap() {
        NoteEvent::Note(n) => assert!(n.slur, "note 6 in measure 0 should be tied"),
        NoteEvent::Rest(_) | NoteEvent::Chord(_) | NoteEvent::Percussion(_) => {
            panic!("expected Note")
        }
    }
}

#[test]
fn standalone_tie_marker_sets_tie_on_preceding_note() {
    // `(6-7)` means note 6 extended by one beat, slurred into note 7
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] (6-7) 0
    """,
"#,
    );
    let notes = first_part_notes(&score, 0);
    match &notes[0] {
        NoteEvent::Note(n) => {
            assert_eq!(n.duration, 8, "note 6 should be extended to 2 beats");
            assert!(n.slur, "note 6 should have tie=true");
        }
        NoteEvent::Rest(_) | NoteEvent::Chord(_) | NoteEvent::Percussion(_) => {
            panic!("expected Note")
        }
    }
    match &notes[1] {
        NoteEvent::Note(n) => assert_eq!(n.pitch, NumberedPitch::Seven),
        NoteEvent::Rest(_) | NoteEvent::Chord(_) | NoteEvent::Percussion(_) => {
            panic!("expected Note")
        }
    }
}

#[test]
fn notes_extension_no_preceding_event_is_recoverable() {
    use crate::error::{Diagnostic, RecoverableErrorKind};
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["n": notes],

score """
    time=4/4 key=C4 bpm=120
    [n] - 2 3 4
    """,
"#,
    );
    assert!(
        score.measures[0].diagnostics.iter().any(|d| matches!(
            d,
            Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::ExtensionNoPrecedingEvent { chord_track: false, .. })
        )),
        "expected ExtensionNoPrecedingEvent error on measure 0"
    );
    assert_eq!(
        first_part_notes(&score, 0).len(),
        3,
        "remaining notes should render after the discarded extension"
    );
}

#[test]
fn measure_span_covers_first_note_byte_offset() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    """,
"#;
    let score = parse_and_group(source);
    let span = &score.measures[0].source_span;
    let first_note_offset = source.find("1 2 3 4").unwrap();
    assert!(
        span.start <= first_note_offset && first_note_offset < span.end,
        "span {span:?} should contain first note offset {first_note_offset}"
    );
}

#[test]
fn second_measure_span_covers_its_first_note() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4

    [Melody] 5 6 7 1
    """,
"#;
    let score = parse_and_group(source);
    assert_eq!(score.measures.len(), 2);
    let span = &score.measures[1].source_span;
    let second_note_offset = source.rfind("5 6 7 1").unwrap();
    assert!(
        span.start <= second_note_offset && second_note_offset < span.end,
        "span {span:?} should contain second measure offset {second_note_offset}"
    );
    // Second measure span must not overlap with first
    assert!(
        span.start >= score.measures[0].source_span.end,
        "measure spans must not overlap: measure[0] ends at {}, measure[1] starts at {}",
        score.measures[0].source_span.end,
        span.start,
    );
}
