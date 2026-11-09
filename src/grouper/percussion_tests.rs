use super::group;
use crate::ast::grouped::{GroupedPercussionHit, NoteEvent, Score};
use crate::parser;

fn parse_and_group(input: &str) -> Score {
    let doc = parser::parse(input, "test.poentaro", &[]).unwrap();
    group(doc).unwrap()
}

/// A document whose score is the directives, then `measures`.
fn with_measures(measures: &str) -> String {
    format!(
        r#"metadata {{ title "t", author "a" }},

parts ["Snare": percussion {{ sound 38 }}],

score """
    time=4/4 key=C4 bpm=120
    {measures}
    """,
"#
    )
}

fn percussion_events(score: &Score, measure_idx: usize) -> Vec<&NoteEvent> {
    score.measures[measure_idx].parts[0]
        .slice()
        .notes
        .events
        .iter()
        .collect()
}

fn hit_at(score: &Score, measure_idx: usize, event_idx: usize) -> &GroupedPercussionHit {
    match &score.measures[measure_idx].parts[0].slice().notes.events[event_idx] {
        NoteEvent::Percussion(p) => p,
        _ => panic!("expected Percussion at [{measure_idx}][{event_idx}]"),
    }
}

#[test]
fn groups_hits_and_rests() {
    let source = with_measures("[Snare] x 0 x 0");
    let score = parse_and_group(&source);
    let events = percussion_events(&score, 0);
    assert_eq!(events.len(), 4);
    assert!(matches!(events[0], NoteEvent::Percussion(_)));
    assert!(matches!(events[1], NoteEvent::Rest(_)));
    assert!(matches!(events[2], NoteEvent::Percussion(_)));
    assert!(matches!(events[3], NoteEvent::Rest(_)));
}

#[test]
fn each_hit_has_one_beat_duration_by_default() {
    let source = with_measures("[Snare] x x x x");
    let score = parse_and_group(&source);
    for i in 0..4 {
        assert_eq!(hit_at(&score, 0, i).duration, 4);
    }
}

#[test]
fn dash_extension_adds_to_previous_hit_duration() {
    let source = with_measures("[Snare] x- x x");
    let score = parse_and_group(&source);
    let events = percussion_events(&score, 0);
    assert_eq!(events.len(), 3);
    assert_eq!(hit_at(&score, 0, 0).duration, 8);
}

#[test]
fn tie_between_hits_marks_slur() {
    let source = with_measures("[Snare] (x~x) x x");
    let score = parse_and_group(&source);
    let first = hit_at(&score, 0, 0);
    assert!(first.tie_to_next());
}
