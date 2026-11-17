use super::*;

const SOURCE: &str = r#"metadata { title "Song" },

parts ["Soprano" [S]: notes, "Alto" [A]: notes],

preset "Choir" { visible parts [S, A], notation: numbered },

preset "Alto" { subtitle "Alto part" },

score """
    [S] 1 2 3 4
    [A] 5 6 7 1
    """,
"#;

fn parts(abbreviations: &[&str]) -> Vec<String> {
    abbreviations.iter().map(|&text| text.to_owned()).collect()
}

#[test]
fn a_new_preset_name_must_be_new_and_quote_free() {
    assert_eq!(new_preset_name_error(SOURCE, "Tenor"), None);
    assert_eq!(
        new_preset_name_error(SOURCE, " Choir "),
        Some("A preset is already called Choir.".to_owned())
    );
    assert_eq!(
        new_preset_name_error(SOURCE, "default"),
        Some("Default is the song without a preset.".to_owned())
    );
    assert_eq!(
        new_preset_name_error(SOURCE, "Say \"hi\""),
        Some("A preset name can't contain quotes or line breaks.".to_owned())
    );
    assert_eq!(
        new_preset_name_error(SOURCE, "  "),
        Some("Give the preset a name.".to_owned())
    );
}

fn part_shown(abbreviation: &str, shown: bool) -> PartShown {
    PartShown {
        abbreviation: abbreviation.to_owned(),
        shown,
    }
}

#[test]
fn hiding_a_part_drops_it_from_visible_parts() {
    let result = set_preset_part_shown(
        SOURCE,
        "Choir",
        &parts(&["S", "A"]),
        &part_shown("S", false),
    );
    assert_eq!(
        result,
        SOURCE.replace("visible parts [S, A]", "visible parts [A]")
    );
}

#[test]
fn showing_a_part_adds_it_back_in_declaration_order() {
    let source = SOURCE.replace("visible parts [S, A]", "visible parts [A]");
    let result = set_preset_part_shown(
        &source,
        "Choir",
        &parts(&["S", "A"]),
        &part_shown("S", true),
    );
    assert_eq!(result, SOURCE);
}

#[test]
fn hiding_a_part_in_a_preset_without_visible_parts_lists_the_rest() {
    let result =
        set_preset_part_shown(SOURCE, "Alto", &parts(&["S", "A"]), &part_shown("S", false));
    assert_eq!(
        result,
        SOURCE.replace(
            r#"preset "Alto" { subtitle"#,
            r#"preset "Alto" { visible parts [A], subtitle"#
        )
    );
}

#[test]
fn setting_a_part_in_an_unknown_preset_changes_nothing() {
    let result =
        set_preset_part_shown(SOURCE, "Nope", &parts(&["S", "A"]), &part_shown("S", false));
    assert_eq!(result, SOURCE);
}

fn part_soloed(abbreviation: &str, soloed: bool) -> PartSoloed {
    PartSoloed {
        abbreviation: abbreviation.to_owned(),
        soloed,
    }
}

#[test]
fn soloing_a_part_adds_a_soloed_parts_line_under_the_header() {
    let result = set_preset_part_soloed(
        SOURCE,
        "Choir",
        &parts(&["S", "A"]),
        &part_soloed("A", true),
    );
    assert_eq!(
        result,
        SOURCE.replace(
            r#"preset "Choir" { visible"#,
            r#"preset "Choir" { soloed parts [A], visible"#
        )
    );
}

#[test]
fn soloing_another_part_keeps_declaration_order() {
    let source = SOURCE.replace("notation: numbered", "notation: numbered, soloed parts [A]");
    let result = set_preset_part_soloed(
        &source,
        "Choir",
        &parts(&["S", "A"]),
        &part_soloed("S", true),
    );
    assert_eq!(
        result,
        source.replace("soloed parts [A]", "soloed parts [S, A]")
    );
}

#[test]
fn unsoloing_the_last_part_removes_the_line() {
    let source = SOURCE.replace("notation: numbered", "notation: numbered, soloed parts [A]");
    let result = set_preset_part_soloed(
        &source,
        "Choir",
        &parts(&["S", "A"]),
        &part_soloed("A", false),
    );
    assert_eq!(result, SOURCE);
}

#[test]
fn unsoloing_the_last_part_removes_every_repeated_line() {
    let source = SOURCE.replace(
        "visible parts [S, A], notation: numbered",
        "soloed parts [S], visible parts [S, A], notation: numbered, soloed parts [A]",
    );
    let result = set_preset_part_soloed(
        &source,
        "Choir",
        &parts(&["S", "A"]),
        &part_soloed("A", false),
    );
    assert_eq!(result, SOURCE);
}

#[test]
fn soloing_in_an_unknown_preset_changes_nothing() {
    let result =
        set_preset_part_soloed(SOURCE, "Nope", &parts(&["S", "A"]), &part_soloed("S", true));
    assert_eq!(result, SOURCE);
}

#[test]
fn rewrites_an_existing_visible_parts_line_in_place() {
    let result = update_preset_field(SOURCE, "Choir", &PresetEdit::VisibleParts(parts(&["A"])));
    assert_eq!(
        result,
        SOURCE.replace("visible parts [S, A]", "visible parts [A]")
    );
}

#[test]
fn inserts_a_missing_line_under_the_preset_header() {
    let result = update_preset_field(SOURCE, "Alto", &PresetEdit::Notation(Notation::Violin));
    assert_eq!(
        result,
        SOURCE.replace(
            r#"preset "Alto" { subtitle"#,
            r#"preset "Alto" { notation: violin, subtitle"#
        )
    );
}

#[test]
fn rewrites_only_the_named_preset() {
    let result = update_preset_field(SOURCE, "Alto", &PresetEdit::VisibleParts(parts(&["A"])));
    assert!(result.contains("visible parts [S, A]"));
    assert!(result.contains(r#"preset "Alto" { visible parts [A], subtitle"#));
}

#[test]
fn rewrites_the_last_of_repeated_lines() {
    let source = r#"preset "P" { notation: abc, notation: guitar }
"#;
    let result = update_preset_field(source, "P", &PresetEdit::Notation(Notation::Cello));
    assert_eq!(
        result,
        r#"preset "P" { notation: abc, notation: cello }
"#
    );
}

#[test]
fn no_visible_parts_writes_an_empty_value() {
    let result = update_preset_field(SOURCE, "Choir", &PresetEdit::VisibleParts(Vec::new()));
    assert!(result.contains(r#"preset "Choir" { visible parts [], notation"#));
}

#[test]
fn inserts_under_a_header_on_the_last_line() {
    let result = update_preset_field(
        r#"preset "P" {}
"#,
        "P",
        &PresetEdit::Notation(Notation::Abc),
    );
    assert_eq!(
        result,
        r#"preset "P" { notation: abc }
"#
    );
}

#[test]
fn leaves_the_source_alone_for_an_unknown_preset() {
    let result = update_preset_field(SOURCE, "Gone", &PresetEdit::Notation(Notation::Abc));
    assert_eq!(result, SOURCE);
}

#[test]
fn follows_a_preset_renamed_in_the_editor() {
    let renamed = SOURCE.replace("\"Alto\"", "\"Alto solo\"");
    assert_eq!(
        follow_preset_rename(SOURCE, &renamed, "Alto"),
        Some("Alto solo".to_owned())
    );
    assert_eq!(
        follow_preset_rename(SOURCE, SOURCE, "Choir"),
        Some("Choir".to_owned())
    );
}

#[test]
fn drops_a_preset_that_was_deleted_or_renamed_to_another_preset() {
    let deleted = SOURCE.replace("preset \"Alto\" { subtitle \"Alto part\" },\n\n", "");
    assert_eq!(follow_preset_rename(SOURCE, &deleted, "Alto"), None);
    let duplicate = SOURCE.replace("\"Alto\"", "\"Choir\"");
    assert_eq!(follow_preset_rename(SOURCE, &duplicate, "Alto"), None);
    assert_eq!(follow_preset_rename(SOURCE, SOURCE, "Gone"), None);
}

fn new_preset(name: &str) -> NewPreset {
    NewPreset {
        name: name.to_owned(),
        visible_parts: parts(&["S"]),
        soloed_parts: Vec::new(),
        notation: Notation::Violin,
    }
}

#[test]
fn adds_a_preset_after_the_last_one() {
    let result = add_preset_section(SOURCE, new_preset("Soprano"));
    assert_eq!(
        result,
        SOURCE.replace(
            "score",
            "preset \"Soprano\" { visible parts [S], notation: violin },\n\nscore"
        )
    );
}

#[test]
fn adds_the_first_preset_before_the_score() {
    let source = r#"metadata { title "Song" },

score """
    1 2 3 4
    """,
"#;
    let result = add_preset_section(source, new_preset("Soprano"));
    assert_eq!(
        result,
        r#"metadata { title "Song" },

preset "Soprano" { visible parts [S], notation: violin },

score """
    1 2 3 4
    """,
"#
    );
}

#[test]
fn adds_a_preset_at_the_end_when_there_is_no_music() {
    let result = add_preset_section(
        r#"metadata { title "Song" }
"#,
        new_preset("Soprano"),
    );
    assert_eq!(
        result,
        r#"metadata { title "Song" },

preset "Soprano" { visible parts [S], notation: violin },
"#
    );
}

#[test]
fn an_added_preset_parses_back() {
    let result = add_preset_section(SOURCE, new_preset("Soprano"));
    let document = crate::parser::parse(&result, "song.poentaro", &[]).expect("parses");
    let preset = document.preset("Soprano").expect("Soprano preset");
    assert_eq!(preset.visible_abbreviations(), Some(parts(&["S"])));
    assert_eq!(preset.notation, Some(Notation::Violin));
}

#[test]
fn a_new_preset_keeps_the_solos_shown_when_it_was_made() {
    let preset = NewPreset {
        soloed_parts: parts(&["A"]),
        ..new_preset("Soprano")
    };
    let result = add_preset_section(SOURCE, preset);
    assert!(result.contains(
        r#"preset "Soprano" { visible parts [S], soloed parts [A], notation: violin },"#
    ));
}

#[test]
fn lists_presets_in_file_order_with_their_notation() {
    assert_eq!(
        list_presets(SOURCE),
        vec![
            PresetSummary {
                name: "Choir".to_owned(),
                notation: Some(Notation::Numbered),
            },
            PresetSummary {
                name: "Alto".to_owned(),
                notation: None,
            },
        ]
    );
}

#[test]
fn the_menu_reads_a_legacy_jianpu_preset_as_numbered() {
    let source = r#"preset "Old" { notation: jianpu }
"#;
    assert_eq!(
        list_presets(source),
        vec![PresetSummary {
            name: "Old".to_owned(),
            notation: Some(Notation::Numbered),
        }]
    );
}
