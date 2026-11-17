use super::parse;
use crate::ast::parsed::{ParsedDocument, ScoreEvent};
use crate::error::Diagnostic;

fn document(score: &str) -> ParsedDocument {
    let input = format!(
        r#"parts ["Melody": notes],

score """
    {score}
    """,
"#
    );
    parse(&input, "test.poentaro", &[]).expect("relative key must not abort parsing")
}

/// The key each measure's directive line changes to, spelled like `key=`
/// (`D4`, `Bb3`), or `None` when the measure has no key change.
fn resolved_keys(doc: &ParsedDocument) -> Vec<Option<String>> {
    doc.directive_events_per_measure
        .iter()
        .map(|events| {
            events.iter().rev().find_map(|event| match &event.value {
                ScoreEvent::KeyChange(key) => {
                    let accidental = match key.note.accidental {
                        crate::ast::parsed::Accidental::Flat => "b",
                        crate::ast::parsed::Accidental::Sharp => "#",
                        crate::ast::parsed::Accidental::Natural => "",
                    };
                    Some(format!(
                        "{:?}{accidental}{}",
                        key.note.name, key.note.octave
                    ))
                }
                _ => None,
            })
        })
        .collect()
}

fn diagnostic_messages(doc: &ParsedDocument) -> Vec<String> {
    doc.per_measure_parse_errors
        .iter()
        .flatten()
        .map(Diagnostic::message)
        .collect()
}

fn key(text: &str) -> Option<String> {
    Some(text.to_string())
}

#[test]
fn shifts_the_previous_key_up() {
    let doc = document(
        r#"key=C4
[Melody] 1 2 3 4

key=+2
[Melody] 1 2 3 4
"#,
    );
    assert_eq!(resolved_keys(&doc), [key("C4"), key("D4")]);
    assert_eq!(diagnostic_messages(&doc), Vec::<String>::new());
}

#[test]
fn stacks_relative_changes() {
    let doc = document(
        r#"key=C4
[Melody] 1 2 3 4

key=+2
[Melody] 1 2 3 4

[Melody] 1 2 3 4

key=+2
[Melody] 1 2 3 4
"#,
    );
    assert_eq!(resolved_keys(&doc), [key("C4"), key("D4"), None, key("E4")]);
}

#[test]
fn shifts_down_across_an_octave() {
    let doc = document(
        r#"key=C4
[Melody] 1 2 3 4

key=-2
[Melody] 1 2 3 4
"#,
    );
    assert_eq!(resolved_keys(&doc), [key("C4"), key("Bb3")]);
}

#[test]
fn shifts_the_default_key_when_none_was_written() {
    let doc = document(
        r#"key=+2
[Melody] 1 2 3 4
"#,
    );
    assert_eq!(resolved_keys(&doc), [key("D4")]);
}

#[test]
fn clamps_an_out_of_range_shift() {
    let doc = document(
        r#"key=C4
[Melody] 1 2 3 4

key=+13
[Melody] 1 2 3 4
"#,
    );
    assert_eq!(resolved_keys(&doc), [key("C4"), key("C5")]);
    assert_eq!(
        diagnostic_messages(&doc),
        ["relative key +13 is out of range; valid range is -12 to +12; clamped to +12"]
    );
}

#[test]
fn warns_that_a_zero_shift_leaves_the_key_unchanged() {
    let doc = document(
        r#"key=D4
[Melody] 1 2 3 4

key=+0
[Melody] 1 2 3 4
"#,
    );
    assert_eq!(resolved_keys(&doc), [key("D4"), None]);
    let warnings: Vec<&Diagnostic> = doc.per_measure_parse_errors.iter().flatten().collect();
    assert!(
        matches!(warnings.as_slice(), [Diagnostic::Warning(_)]),
        "expected exactly one warning, got: {warnings:?}"
    );
    assert_eq!(
        diagnostic_messages(&doc),
        ["'key=+0' shifts the key by 0 semitones, so the key is unchanged"]
    );
}

#[test]
fn rejects_a_shift_that_is_not_a_whole_number() {
    let doc = document(
        r#"key=+two
[Melody] 1 2 3 4
"#,
    );
    assert_eq!(resolved_keys(&doc), [None]);
    assert_eq!(
        diagnostic_messages(&doc),
        ["invalid relative key 'key=+two': expected a whole number of semitones, e.g. +2 or -1"]
    );
}
