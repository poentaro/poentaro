fn parse_score(notes_line: &str) -> Result<crate::RenderOutput, crate::error::IrrecoverableError> {
    let input = format!(
        r#"metadata {{ title "t", author "a" }},

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] {notes_line}
    """,
"#
    );
    crate::render_svgs_from_source(&input, "test.poentaro", &[])
}

#[test]
fn chord_half_bar_boundary_validation_matches_notes() {
    let input = r#"metadata { title "t", author "a" },

parts ["c": chords, "n": notes],

score """
    time=4/4 key=C4 bpm=120
    [c] 1. 2. 3_ 4_
    [n] 1 2 3 4
    """,
"#;
    let output = crate::render_svgs_from_source(input, "t.poentaro", &[]).unwrap();
    assert!(output
        .diagnostics
        .iter()
        .any(|e| e.message().contains("half-bar boundary")));
}

#[test]
fn recovers_half_bar_crossing() {
    let output = parse_score("1. 2. 3_ 4_\n").unwrap();
    assert!(output
        .diagnostics
        .iter()
        .any(|e| e.message().contains("half-bar boundary")));
}

#[test]
fn recovers_half_bar_crossing_on_half_note() {
    let output = parse_score("1 2- 0_ 0_\n").unwrap();
    assert!(output
        .diagnostics
        .iter()
        .any(|e| e.message().contains("half-bar boundary")));
}

#[test]
fn accepts_half_bar_split_with_beam_group() {
    assert!(parse_score("1. (2_ 2_) 3_ 4_ 0_\n").is_ok());
}

#[test]
fn accepts_tied_note_crossing_half_bar() {
    // 2~2-0: quarter tied to half note, then quarter rest.
    // The second note (2-) starts at beat 2 and spans the half-bar;
    // because the composer explicitly tied across the boundary, no warning should fire.
    let output = parse_score("2~2-0\n").unwrap();
    assert!(
        !output
            .diagnostics
            .iter()
            .any(|e| e.message().contains("half-bar boundary")),
        "tied note crossing half-bar should not warn, but got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|e| e.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn recovers_dotted_eighth_without_tail_group() {
    use super::validate_measure_grouping;
    use crate::parser::score::token_parser;
    let bar = "1_. 2_ 3_ 4_ 5_ 6_ 7_ 0=";
    let events = token_parser::parse_notes_line(bar, 0, &mut Default::default())
        .unwrap()
        .events;
    let errors = validate_measure_grouping(&events, 4, 4, 1).unwrap();
    assert!(!errors.is_empty());
    assert!(errors[0].message().contains("dotted eighth"));
}

#[test]
fn accepts_dotted_eighth_with_sixteenth_tail() {
    assert!(parse_score("1_. 2= 3_ 4_ 5_ 6_ 7_ 1_\n").is_ok());
}

#[test]
fn recovers_dotted_eighth_rest_without_tail_group() {
    let output = parse_score("0_. 1_ 2_ 3_ 4_ 5_ 6_ 0=\n").unwrap();
    assert!(output
        .diagnostics
        .iter()
        .any(|e| e.message().contains("dotted eighth")));
}

#[test]
fn accepts_extension_notes_that_start_on_beat_three() {
    assert!(parse_score("(6- 7-)\n").is_ok());
}

#[test]
fn skips_validation_for_non_four_four() {
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=3/4 key=C4 bpm=120
    [Melody] 1 2 3
    """,
"#;
    assert!(crate::render_svgs_from_source(input, "test.poentaro", &[]).is_ok());
}

#[test]
fn allows_half_bar_crossing_inside_beam_group() {
    use super::validate_measure_grouping;
    use crate::parser::score::token_parser;
    let mut state = token_parser::GroupStack::default();
    let bar1 = "5_ 5_ 5_ 5= 5= 5_ 3_ 2_ (3_";
    token_parser::parse_notes_line(bar1, 0, &mut state).unwrap();
    let bar2 = "3_) (1_1-) 0_ 1= 1=";
    let events = token_parser::parse_notes_line(bar2, 0, &mut state)
        .unwrap()
        .events;
    validate_measure_grouping(&events, 4, 4, 1).expect("grouped crossing should be allowed");
}

#[test]
fn dotted_extension_fills_compound_meter_measure() {
    // In 9/8 (a compound triple meter), each beat is a dotted quarter (6 quarter-beats).
    // `-.` extends the previous note by one dotted-quarter beat, so a note followed by
    // two `-.` atoms spans the whole 3-beat measure (6 + 6 + 6 = 18 quarter-beats).
    let input = r#"metadata { title "t", author "a" },

parts ["a": notes],

score """
    time=9/8
    [a] 1. -. -.
    """,
"#;
    let output = crate::render_svgs_from_source(input, "test.poentaro", &[]).unwrap();
    assert!(
        output.diagnostics.is_empty(),
        "expected no diagnostics, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|e| e.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn double_dotted_extension_fills_measure() {
    // In 7/4, each measure holds 28 quarter-beats. A double-dotted quarter note
    // (4 + 2 + 1 = 7 quarter-beats) followed by three `-..` (double-dotted-beat)
    // extensions spans the whole measure (7 + 7 + 7 + 7 = 28 quarter-beats).
    let input = r#"metadata { title "t", author "a" },

parts ["a": notes],

score """
    time=7/4
    [a] 1.. -.. -.. -..
    """,
"#;
    let output = crate::render_svgs_from_source(input, "test.poentaro", &[]).unwrap();
    assert!(
        output.diagnostics.is_empty(),
        "expected no diagnostics, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|e| e.message())
            .collect::<Vec<_>>()
    );
}
