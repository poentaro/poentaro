use super::*;
use crate::ast::parsed::NoteName;

#[test]
fn first_measure_has_bpm_some() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#,
    );
    assert_eq!(score.measures[0].bpm, Some(120));
}

#[test]
fn bpm_change_sets_some_on_next_measure() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d

    bpm=90
    [Melody] 5 6 7 1
    [Melodyv1] e f g h
    """,
"#,
    );
    assert_eq!(score.measures[0].bpm, Some(120));
    assert_eq!(score.measures[1].bpm, Some(90));
}

#[test]
fn unchanged_bpm_is_none_on_second_measure() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d

    [Melody] 5 6 7 1
    [Melodyv1] e f g h
    """,
"#,
    );
    assert_eq!(score.measures[0].bpm, Some(120));
    assert_eq!(score.measures[1].bpm, None);
}

#[test]
fn key_change_propagates() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=G4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#,
    );
    assert_eq!(
        score.measures[0].key.as_ref().unwrap().note.name,
        NoteName::G
    );
}

#[test]
fn row_height_defaults_to_24() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#,
    );
    assert_eq!(score.metadata.row_height, 24);
}

#[test]
fn max_measures_per_system_defaults_to_4() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#,
    );
    assert_eq!(score.metadata.max_measures_per_system, 4);
}

#[test]
fn measure_number_font_size_defaults_to_10() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#,
    );
    assert_eq!(score.metadata.measure_number.font_size, 10);
}

#[test]
fn section_label_font_size_defaults_to_12() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#,
    );
    assert_eq!(score.metadata.section_label.font_size, 12);
}

#[test]
fn part_label_font_size_defaults_to_12() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#,
    );
    assert_eq!(score.metadata.part_label.font_size, 12);
}

#[test]
fn page_number_font_size_defaults_to_60_percent_of_row_height() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#,
    );
    assert_eq!(score.metadata.page_number.font_size, 14);
}

#[test]
fn lyric_click_target_padding_pt_defaults_to_12() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#,
    );
    assert_eq!(
        score.metadata.lyrics.vertical_padding_pt,
        crate::ast::grouped::DEFAULT_LYRIC_CLICK_TARGET_PADDING_PT
    );
}

#[test]
fn lyric_click_target_padding_pt_is_parsed_from_metadata() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a", lyrics style { vertical padding pt 20 } },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#,
    );
    assert_eq!(score.metadata.lyrics.vertical_padding_pt, 20);
}

#[test]
fn measure_number_font_size_is_parsed_from_metadata() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a", measure number style { font size 8 } },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#,
    );
    assert_eq!(score.metadata.measure_number.font_size, 8);
}

#[test]
fn label_directive_propagates_to_measure() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120 label="Verse 1"
    [Melody] 1 2 3 4
    """,
"#,
    );
    assert_eq!(score.measures[0].label, Some("Verse 1".to_string()));
}

#[test]
fn label_is_none_when_not_declared() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    """,
"#,
    );
    assert_eq!(score.measures[0].label, None);
}

#[test]
fn label_does_not_persist_to_next_measure() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120 label="Verse 1"
    [Melody] 1 2 3 4

    [Melody] 5 6 7 1
    """,
"#,
    );
    assert_eq!(score.measures[0].label, Some("Verse 1".to_string()));
    assert_eq!(score.measures[1].label, None);
}

#[test]
fn label_on_second_measure_not_first() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4

    label="Chorus"
    [Melody] 5 6 7 1
    """,
"#,
    );
    assert_eq!(score.measures[0].label, None);
    assert_eq!(score.measures[1].label, Some("Chorus".to_string()));
}

#[test]
fn hide_resting_parts_defaults_to_true_when_never_declared() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    """,
"#,
    );
    assert!(score.measures[0].hide_resting_parts);
}

#[test]
fn hide_resting_parts_directive_propagates_and_persists_to_next_measure() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120 hide_resting_parts=no
    [Melody] 1 2 3 4

    [Melody] 5 6 7 1
    """,
"#,
    );
    assert!(!score.measures[0].hide_resting_parts);
    assert!(
        !score.measures[1].hide_resting_parts,
        "sticky: stays disabled on the next measure with no override"
    );
}

#[test]
fn hide_resting_parts_seeds_from_metadata_default_then_can_be_overridden_mid_score() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a", hide resting parts: no },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4

    hide_resting_parts=yes
    [Melody] 5 6 7 1
    """,
"#,
    );
    assert!(
        !score.measures[0].hide_resting_parts,
        "seeded from metadata default"
    );
    assert!(
        score.measures[1].hide_resting_parts,
        "overridden mid-score by the directive line"
    );
}

#[test]
fn merge_duplicate_measures_across_parts_defaults_to_true_when_never_declared() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    """,
"#,
    );
    assert!(score.measures[0].merge_duplicate_measures_across_parts);
}
