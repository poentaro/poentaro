use crate::compiler::{compile, label_note_heads, CompileResult, ElementContent, NoteLabels};

fn compile_tabs(source: &str) -> CompileResult {
    let doc = crate::parser::parse(source, "test", &[]).unwrap();
    let score = crate::grouper::group(doc).unwrap();
    label_note_heads(compile(&score), &score, NoteLabels::HarmonicaTabs)
}

/// Every note head's tab, in order, with an unplayable one marked `!` and
/// one played with the slide button pressed in followed by `←`.
fn tabs(result: &CompileResult) -> Vec<String> {
    result
        .blocks
        .iter()
        .flat_map(|block| &block.rows)
        .flat_map(|row| &row.elements)
        .filter_map(|element| match &element.content {
            ElementContent::NoteHead { head_label, .. } => head_label.as_ref(),
            _ => None,
        })
        .map(|label| match (label.unplayable, label.slide_button) {
            (true, _) => format!("!{}", label.text),
            (false, true) => format!("{}←", label.text),
            (false, false) => label.text.clone(),
        })
        .collect()
}

fn melody(key: &str, notes: &str) -> String {
    format!(
        r#"parts ["S": notes],

score """
    time=4/4 key={key} bpm=120
    {notes}
    """,
"#
    )
}

#[test]
fn the_c_scale_from_middle_c_climbs_four_holes_an_octave() {
    // Each C sits between a draw on the hole below and one on the hole
    // above, a tie that goes to the lower hole.
    assert_eq!(
        tabs(&compile_tabs(&melody(
            "C4",
            r"[S] 1 2 3 4

[S] 5 6 7 1'

[S] 2' 3' 4' 5'

[S] 6' 7' 1'' 2''

[S] 3'' 4'' 5'' 6''

[S] 7'' 1''' 0 0"
        ))),
        [
            "1", "(1)", "2", "(2)", "3", "(3)", "(4)", "4", "(5)", "6", "(6)", "7", "(7)", "(8)",
            "8", "(9)", "10", "(10)", "11", "(11)", "(12)", "12",
        ]
    );
}

#[test]
fn the_harmonica_stays_in_c_and_sharps_use_the_slide() {
    // The harmonica player's D chart: (5) 6 (6)← 7 (7) (8) 8← (9).
    assert_eq!(
        tabs(&compile_tabs(&melody(
            "D4",
            r"[S] 1' 2' 3' 4'

[S] 5' 6' 7' 1''"
        ))),
        ["(5)", "6", "(6)←", "7", "(7)", "(8)", "8←", "(9)"]
    );
}

#[test]
fn a_g_scale_starts_on_hole_3_blow() {
    // The harmonica player's G chart reads 3 (3) (4) 5 (5) 6 (6)← 7: their
    // C on 5 ties with 4 here, and a tie goes to the lower hole.
    assert_eq!(
        tabs(&compile_tabs(&melody(
            "G4",
            r"[S] 1 2 3 4

[S] 5 6 7 1'"
        ))),
        ["3", "(3)", "(4)", "4", "(5)", "6", "(6)←", "7"]
    );
}

#[test]
fn a_c_shared_by_two_holes_goes_where_the_melody_moves_least() {
    // B C B stays on hole 4; D C D stays on hole 5.
    assert_eq!(
        tabs(&compile_tabs(&melody(
            "C4",
            r"[S] 7 1' 7 0

[S] 2' 1' 2' 0"
        ))),
        ["(4)", "4", "(4)", "(5)", "5", "(5)"]
    );
}

#[test]
fn flats_are_the_note_below_with_the_slide() {
    // B♭ is A with the slide; E♭ is D with the slide.
    assert_eq!(
        tabs(&compile_tabs(&melody("C4", "[S] 7b 3b 1# 4#"))),
        ["(3)←", "(1)←", "1←", "(2)←"]
    );
}

#[test]
fn a_key_change_keeps_the_same_c_harmonica() {
    // key=G4's `1` is G, hole 3 blow.
    assert_eq!(
        tabs(&compile_tabs(&melody(
            "C4",
            r"[S] 1 2 3 4

key=G4
[S] 1 2 3 4"
        ))),
        ["1", "(1)", "2", "(2)", "3", "(3)", "(4)", "4"]
    );
}

#[test]
fn pitches_outside_the_harmonica_s_range_are_flagged() {
    // B3 is below hole 1; D7 is above hole 12 blow with the slide (C♯7).
    assert_eq!(
        tabs(&compile_tabs(&melody("C4", "[S] 7, 1 1''' 2'''"))),
        ["!?", "1", "12", "!?"]
    );
}

#[test]
fn tabs_drop_octave_dots_since_the_hole_gives_the_octave() {
    let result = compile_tabs(&melody("C4", "[S] 1, 1 1' 1''"));
    let octaves: Vec<i8> = result
        .blocks
        .iter()
        .flat_map(|block| &block.rows)
        .flat_map(|row| &row.elements)
        .filter_map(|element| match &element.content {
            ElementContent::NoteHead { octave, .. } => Some(*octave),
            _ => None,
        })
        .collect();
    assert_eq!(octaves, [0, 0, 0, 0]);
}

#[test]
fn a_harmonica16_part_reaches_down_to_c3_on_hole_1() {
    // The 16-hole adds a low octave, so its middle C is hole 5 and its top
    // is hole 16 blow with the slide (C♯7), the same as the 12-hole's.
    assert_eq!(
        tabs(&compile_tabs(
            r#"parts ["H": notes { instrument: harmonica16 }],

score """
    time=4/4 key=C4 bpm=120
    [H] 1, 7, 1 1#'''

    [H] 2''' 0 0 0
    """,
"#
        )),
        ["1", "(4)", "4", "16←", "!?"]
    );
}

#[test]
fn each_part_plays_its_own_harmonica() {
    // B3 has a hole on the 16-hole only.
    assert_eq!(
        tabs(&compile_tabs(
            r#"parts ["H1": notes { instrument: harmonica }, "H2": notes { instrument: harmonica16 }],

score """
    time=4/4 key=C4 bpm=120
    [H1] 7, 1 0 0
    [H2] 7, 1 0 0
    """,
"#
        )),
        ["!?", "1", "(4)", "4"]
    );
}
