use crate::compiler::{compile, label_note_heads, CompileResult, ElementContent, NoteLabels};

fn compile_with(source: &str, labels: NoteLabels) -> CompileResult {
    let doc = crate::parser::parse(source, "test", &[]).unwrap();
    let score = crate::grouper::group(doc).unwrap();
    label_note_heads(compile(&score), &score, labels)
}

/// Each chord symbol's diagram, as absolute frets (`"x 3 2 0 1 0"`), in
/// written order; `None` for a chord drawn without one.
fn diagram_frets(result: &CompileResult) -> Vec<Option<String>> {
    result
        .blocks
        .iter()
        .flat_map(|block| &block.rows)
        .flat_map(|row| &row.elements)
        .filter_map(|element| match &element.content {
            ElementContent::ChordSymbol { guitar_voicing, .. } => {
                Some(guitar_voicing.as_ref().map(|voicing| voicing.frets_label()))
            }
            _ => None,
        })
        .collect()
}

fn chords(key: &str, chords: &str) -> String {
    format!(
        r#"parts ["C": chords],

score """
    time=4/4 key={key} bpm=120
    [C] {chords}
    """,
"#
    )
}

fn frets(frets: &[&str]) -> Vec<Option<String>> {
    frets.iter().map(|frets| Some(frets.to_string())).collect()
}

#[test]
fn an_open_progression_stays_in_open_shapes() {
    assert_eq!(
        diagram_frets(&compile_with(
            &chords("C4", "1 6m 4 5"),
            NoteLabels::GuitarChords
        )),
        frets(&["x 3 2 0 1 0", "x 0 2 2 1 0", "1 3 3 2 1 1", "3 2 0 0 0 3"])
    );
}

#[test]
fn diagrams_follow_the_key() {
    assert_eq!(
        diagram_frets(&compile_with(
            &chords("D4", "1 - - -"),
            NoteLabels::GuitarChords
        )),
        frets(&["x x 0 2 3 2"])
    );
}

#[test]
fn a_repeated_chord_keeps_its_voicing() {
    assert_eq!(
        diagram_frets(&compile_with(
            &chords("C4", "1 1 1 1"),
            NoteLabels::GuitarChords
        )),
        frets(&["x 3 2 0 1 0"; 4])
    );
}

#[test]
fn a_chord_without_a_chords_db_voicing_has_no_diagram() {
    assert_eq!(
        diagram_frets(&compile_with(
            &chords("C4", "1 1sus27 1 1"),
            NoteLabels::GuitarChords
        )),
        vec![
            Some("x 3 2 0 1 0".to_string()),
            None,
            Some("x 3 2 0 1 0".to_string()),
            Some("x 3 2 0 1 0".to_string()),
        ]
    );
}

#[test]
fn other_views_draw_no_diagrams() {
    let source = chords("C4", "1 6m 4 5");
    for labels in [NoteLabels::Degrees, NoteLabels::LetterNames] {
        assert_eq!(
            diagram_frets(&compile_with(&source, labels)),
            vec![None; 4],
            "{labels:?}"
        );
    }
}
