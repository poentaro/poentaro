use crate::ast::grouped::Score;
use crate::error::{Diagnostic, Warning, WarningKind};
use crate::grouper::group;
use crate::parser;

fn parse_and_group(parts: &str, score: &str) -> Score {
    let input =
        format!("parts [{parts}],\n\nscore \"\"\"\ntime=4/4 key=C4 bpm=120\n{score}\n\"\"\",\n");
    let doc = parser::parse(&input, "test.poentaro", &[]).unwrap();
    group(doc).unwrap()
}

fn range_warnings(score: &Score) -> Vec<&Warning> {
    score
        .measures
        .iter()
        .flat_map(|measure| &measure.diagnostics)
        .filter_map(|diagnostic| match diagnostic {
            Diagnostic::Warning(warning) if warning.kind == WarningKind::NoteOutOfRange => {
                Some(warning)
            }
            _ => None,
        })
        .collect()
}

fn range_messages(score: &Score) -> Vec<String> {
    range_warnings(score)
        .into_iter()
        .map(|warning| warning.message.clone())
        .collect()
}

#[test]
fn part_without_instrument_gets_no_warnings() {
    let score = parse_and_group("\"Violin\" [V] : notes", "[V] 1,, 2,, 3,, 4,,");
    assert!(range_warnings(&score).is_empty());
}

#[test]
fn notes_inside_the_range_get_no_warnings() {
    let score = parse_and_group(
        "\"Violin\" [V] : notes { instrument : violin }",
        "[V] 5, 1 3 5",
    );
    assert!(range_warnings(&score).is_empty());
}

#[test]
fn each_note_below_the_range_gets_its_own_warning() {
    let score = parse_and_group(
        "\"Violin\" [V] : notes { instrument : violin }",
        "[V] 1 4, 5, 3,",
    );
    assert_eq!(
        range_messages(&score),
        vec![
            "V: note sounds as F3, below the violin's lowest note G3",
            "V: note sounds as E3, below the violin's lowest note G3",
        ]
    );
}

#[test]
fn warning_points_at_the_note() {
    let input = "[V] 1 4, 5 5";
    let score = parse_and_group("\"Violin\" [V] : notes { instrument : violin }", input);
    let warnings = range_warnings(&score);
    let [warning] = warnings.as_slice() else {
        panic!("expected one warning, got {}", warnings.len());
    };
    let source = format!(
        "parts [\"Violin\" [V] : notes {{ instrument : violin }}],\n\nscore \"\"\"\ntime=4/4 key=C4 bpm=120\n{input}\n\"\"\",\n"
    );
    assert_eq!(&source[warning.span.start..warning.span.end], "4,");
}

#[test]
fn note_above_the_range_names_the_highest_note() {
    let score = parse_and_group(
        "\"Soprano\" [S] : notes { instrument : soprano }",
        "[S] 1 1' 6' 1",
    );
    assert_eq!(
        range_messages(&score),
        vec!["S: note sounds as A5, above the soprano's highest note G5"]
    );
}

#[test]
fn octave_offset_shifts_the_sounding_pitch() {
    let score = parse_and_group(
        "\"Cello\" [C] : notes { instrument : cello, octave : -1 }",
        "[C] 1 1 1 1'",
    );
    assert!(range_warnings(&score).is_empty());

    let score = parse_and_group(
        "\"Cello\" [C] : notes { instrument : cello, octave : -2 }",
        "[C] 1, 1 1 1",
    );
    assert_eq!(
        range_messages(&score),
        vec!["C: note sounds as C1, below the cello's lowest note C2"]
    );
}

#[test]
fn key_shifts_the_sounding_pitch() {
    let input = r#"parts ["Alto" [A]: notes { instrument: alto }],

score """
    time=4/4 key=A3 bpm=120
    [A] 1 3 5 3
    """,
"#;
    let score = group(parser::parse(input, "test.poentaro", &[]).unwrap()).unwrap();
    assert!(range_warnings(&score).is_empty());

    let input = r#"parts ["Alto" [A]: notes { instrument: alto }],

score """
    time=4/4 key=Eb4 bpm=120
    [A] 1 3 5 1'
    """,
"#;
    let score = group(parser::parse(input, "test.poentaro", &[]).unwrap()).unwrap();
    assert_eq!(
        range_messages(&score),
        vec!["A: note sounds as Eb5, above the alto's highest note D5"]
    );
}

#[test]
fn follow_part_inherits_its_targets_instrument() {
    let score = parse_and_group(
        "\"Violin\" [V] : notes { instrument : violin }, \"Violin 2\" [V2] : follow [V] { octave : -1 }",
        "[V] 1 2 3 4",
    );
    assert_eq!(range_messages(&score).len(), 4);
    assert!(range_messages(&score)
        .iter()
        .all(|message| message.starts_with("V2: ")));
}

#[test]
fn follow_part_can_override_its_targets_instrument() {
    let score = parse_and_group(
        "\"Violin\" [V] : notes { instrument : violin }, \"Cello\" [C] : follow [V] { instrument : cello, octave : -1 }",
        "[V] 1 2 3 4",
    );
    assert!(range_warnings(&score).is_empty());
}

#[test]
fn chords_part_cannot_name_an_instrument() {
    let input = r#"parts ["Chords" [C]: chords { instrument: violin }],

score """
    [C] 1 - - -
    """,
"#;
    let doc = parser::parse(input, "test.poentaro", &[]).unwrap();
    assert!(doc
        .parts_parse_errors
        .iter()
        .any(|error| error.message().contains("only notes parts")));
}

#[test]
fn unknown_instrument_is_reported_and_the_part_kept() {
    let input = r#"parts ["Violin" [V]: notes { instrument: viola }],

score """
    [V] 1 2 3 4
    """,
"#;
    let doc = parser::parse(input, "test.poentaro", &[]).unwrap();
    assert!(doc
        .parts_parse_errors
        .iter()
        .any(|error| error.message().contains("unknown instrument \"viola\"")));
    assert_eq!(doc.declarations.len(), 1);
}

#[test]
fn a_harmonica16_reaches_an_octave_below_the_12_hole() {
    let score = parse_and_group(
        "\"H1\" [H1] : notes { instrument : harmonica }, \"H2\" [H2] : notes { instrument : harmonica16 }",
        "[H1] 1, 1 0 0\n[H2] 1, 7,, 0 0",
    );
    assert_eq!(
        range_messages(&score),
        vec![
            "H1: note sounds as C3, below the harmonica's lowest note C4",
            "H2: note sounds as B2, below the harmonica16's lowest note C3",
        ]
    );
}
