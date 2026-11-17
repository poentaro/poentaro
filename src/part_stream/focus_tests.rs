use crate::filters::PartToggleState;
use crate::part_stream::{part_focus_measures, FocusMeasure};

const FOUR_PARTS: &str = r#"parts ["Soprano" [S]: notes, "Melody" [M]: notes, "Words" [W]: lyrics [M], "Bass" [B]: notes],

score """
    label="Verse"
    [S] 5 5 5 5
    [M] 1 2 3 4
    [W] a b c d
    [B] 1. - 5. -

    [M] 5 6 7 1 // second
    [B] 1. - 5. -

    // a comment
    [B] 1. - - -
    [M] 3 4 5 6
    """,
"#;

fn toggles(hidden: &[&str], soloed: &[&str]) -> PartToggleState {
    PartToggleState {
        hidden_parts: hidden.iter().map(ToString::to_string).collect(),
        soloed_parts: soloed.iter().map(ToString::to_string).collect(),
    }
}

fn lines(texts: &[&str]) -> Vec<String> {
    texts.iter().map(ToString::to_string).collect()
}

fn measures(state: &PartToggleState) -> Vec<FocusMeasure> {
    part_focus_measures(FOUR_PARTS, "M", state).unwrap()
}

#[test]
fn lines_split_above_and_below_by_declaration_order() {
    let first = &measures(&PartToggleState::default())[0];
    assert_eq!(
        first.lines_above,
        lines(&["label=\"Verse\"", "[S] 5 5 5 5"])
    );
    assert_eq!(first.lines_below, lines(&["[W] a b c d", "[B] 1. - 5. -"]));
}

#[test]
fn a_part_the_measure_does_not_write_has_no_line() {
    let second = &measures(&PartToggleState::default())[1];
    assert_eq!(second.lines_above, Vec::<String>::new());
    assert_eq!(second.lines_below, lines(&["[B] 1. - 5. -"]));
}

#[test]
fn a_hidden_part_has_no_lines() {
    let first = &measures(&toggles(&["B"], &[]))[0];
    assert_eq!(first.lines_below, lines(&["[W] a b c d"]));
}

#[test]
fn solo_wins_over_hide() {
    let first = &measures(&toggles(&["B"], &["B"]))[0];
    assert_eq!(first.lines_above, lines(&["label=\"Verse\""]));
    assert_eq!(first.lines_below, lines(&["[B] 1. - 5. -"]));
}

#[test]
fn the_section_label_is_surfaced() {
    let all = measures(&PartToggleState::default());
    assert_eq!(all[0].section_label.as_deref(), Some("Verse"));
    assert_eq!(all[1].section_label, None);
}

#[test]
fn a_comment_line_is_kept_above() {
    let third = &measures(&toggles(&[], &["S"]))[2];
    assert_eq!(third.lines_above, lines(&["// a comment"]));
    assert_eq!(third.lines_below, Vec::<String>::new());
}

#[test]
fn an_ineligible_part_is_refused() {
    assert!(part_focus_measures(FOUR_PARTS, "X", &PartToggleState::default()).is_err());
}
