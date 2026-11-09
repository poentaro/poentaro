use crate::compiler::{compile, label_note_heads, CompileResult, ElementContent, NoteLabels};

fn compile_with(source: &str, labels: NoteLabels) -> CompileResult {
    let doc = crate::parser::parse(source, "test", &[]).unwrap();
    let score = crate::grouper::group(doc).unwrap();
    label_note_heads(compile(&score), &score, labels)
}

/// Each block's note heads' labels after any leading sharp/flat, in order:
/// the letter name, or the digit followed by it in the guitar chord view
/// (`None` for a head left unlabelled).
fn letter_names(result: &CompileResult) -> Vec<Vec<Option<String>>> {
    result
        .blocks
        .iter()
        .map(|block| {
            block
                .rows
                .iter()
                .flat_map(|row| &row.elements)
                .filter_map(|element| match &element.content {
                    ElementContent::NoteHead {
                        pitch, head_label, ..
                    } => Some(
                        head_label
                            .as_ref()
                            .map(|label| label.run_text(pitch.to_digit())),
                    ),
                    _ => None,
                })
                .collect()
        })
        .collect()
}

fn names(names: &[&str]) -> Vec<Option<String>> {
    names.iter().map(|name| Some(name.to_string())).collect()
}

#[test]
fn letter_names_spell_degrees_against_the_key_including_implied_accidentals() {
    let result = compile_with(
        r#"parts ["S": notes],

score """
    time=4/4 key=D4 bpm=120
    [S] 1 2 3 4

    [S] 5 6 7 1'
    """,
"#,
        NoteLabels::LetterNames,
    );
    assert_eq!(
        letter_names(&result),
        vec![
            names(&["D", "E", "F\u{266F}", "G"]),
            names(&["A", "B", "C\u{266F}", "D"]),
        ]
    );
}

#[test]
fn letter_names_carry_written_accidentals() {
    let result = compile_with(
        r#"parts ["S": notes],

score """
    time=4/4 key=F4 bpm=120
    [S] 4# 7b 7 1#
    """,
"#,
        NoteLabels::LetterNames,
    );
    assert_eq!(
        letter_names(&result),
        vec![names(&["B", "E\u{266D}", "E", "F\u{266F}"])]
    );
}

#[test]
fn letter_names_follow_a_mid_score_key_change_and_keep_it_for_later_measures() {
    let result = compile_with(
        r#"parts ["S": notes],

score """
    time=4/4 key=C4 bpm=120
    [S] 1 2 3 4

    key=G4
    [S] 1 2 3 7

    [S] 7 - - -
    """,
"#,
        NoteLabels::LetterNames,
    );
    assert_eq!(
        letter_names(&result),
        vec![
            names(&["C", "D", "E", "F"]),
            names(&["G", "A", "B", "F\u{266F}"]),
            names(&["F\u{266F}"]),
        ]
    );
}

#[test]
fn letter_names_track_the_key_across_a_merged_rest_run() {
    let result = compile_with(
        r#"parts ["S": notes],

score """
    time=4/4 key=E4 bpm=120
    [S] 1 2 3 4

    [S] 0 0 0 0

    [S] 0 0 0 0

    [S] 3 - - -
    """,
"#,
        NoteLabels::LetterNames,
    );
    assert_eq!(
        letter_names(&result),
        vec![
            names(&["E", "F\u{266F}", "G\u{266F}", "A"]),
            vec![],
            names(&["G\u{266F}"]),
        ]
    );
}

#[test]
fn degrees_leave_note_heads_unlabelled() {
    let result = compile_with(
        r#"parts ["S": notes],

score """
    time=4/4 key=D4 bpm=120
    [S] 1 2 3 4
    """,
"#,
        NoteLabels::Degrees,
    );
    assert_eq!(letter_names(&result), vec![vec![None, None, None, None]]);
}

#[test]
fn letter_names_leave_percussion_hits_alone() {
    let result = compile_with(
        r#"parts ["Snare": percussion { sound 38 }],

score """
    time=4/4 key=D4 bpm=120
    [Snare] x 0 x 0
    """,
"#,
        NoteLabels::LetterNames,
    );
    let hits = result.blocks[0]
        .rows
        .iter()
        .flat_map(|row| &row.elements)
        .filter(|element| matches!(element.content, ElementContent::PercussionHit))
        .count();
    assert_eq!(hits, 2);
    assert_eq!(letter_names(&result), vec![vec![]]);
}

/// Each block's chord symbols' text, in order.
fn chord_texts(result: &CompileResult) -> Vec<Vec<String>> {
    result
        .blocks
        .iter()
        .map(|block| {
            block
                .rows
                .iter()
                .flat_map(|row| &row.elements)
                .filter_map(|element| match &element.content {
                    ElementContent::ChordSymbol { text, .. } => Some(text.clone()),
                    _ => None,
                })
                .collect()
        })
        .collect()
}

const CHORDS_IN_D: &str = r#"parts ["C": chords],

score """
    time=4/4 key=D4 bpm=120
    [C] 1 6m/3 4M7 5b7

    key=F4
    [C] 1 7bsus4 3#o 2m7/5
    """,
"#;

#[test]
fn letter_names_spell_chord_roots_and_slash_basses_against_the_key() {
    assert_eq!(
        chord_texts(&compile_with(CHORDS_IN_D, NoteLabels::LetterNames)),
        vec![
            vec!["D", "Bm/F\u{266F}", "G△⁷", "A\u{266D}⁷"],
            vec!["F", "E\u{266D}sus4", "A\u{266F}°", "Gm⁷/C"],
        ]
    );
}

#[test]
fn degrees_keep_chord_symbols_in_numbered() {
    assert_eq!(
        chord_texts(&compile_with(CHORDS_IN_D, NoteLabels::Degrees)),
        vec![
            vec!["1", "6m/3", "4△⁷", "\u{266D}5⁷"],
            vec!["1", "\u{266D}7sus4", "\u{266F}3°", "2m⁷/5"],
        ]
    );
}

#[test]
fn guitar_chords_follow_each_degree_with_its_letter_name() {
    let result = compile_with(
        r#"parts ["S": notes],

score """
    time=4/4 key=D4 bpm=120
    [S] 1 3 4# 7b
    """,
"#,
        NoteLabels::GuitarChords,
    );
    assert_eq!(
        letter_names(&result),
        vec![names(&["1 (D)", "3 (F\u{266F})", "4 (G\u{266F})", "7 (C)"])]
    );
}

#[test]
fn guitar_chords_follow_each_chord_symbol_with_its_letter_name() {
    assert_eq!(
        chord_texts(&compile_with(CHORDS_IN_D, NoteLabels::GuitarChords)),
        vec![
            vec![
                "1 (D)",
                "6m/3 (Bm/F\u{266F})",
                "4△⁷ (G△⁷)",
                "\u{266D}5⁷ (A\u{266D}⁷)"
            ],
            vec![
                "1 (F)",
                "\u{266D}7sus4 (E\u{266D}sus4)",
                "\u{266F}3° (A\u{266F}°)",
                "2m⁷/5 (Gm⁷/C)"
            ],
        ]
    );
}
