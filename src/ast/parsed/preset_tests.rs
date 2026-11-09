use super::*;
use crate::ast::parsed::{FontFamilyChoice, Offset};

#[test]
fn overlay_keeps_unset_fields_and_replaces_set_ones() {
    let base = ParsedMetadata {
        title: Some("Song".to_string()),
        subtitle: Some("Base".to_string()),
        row_height: Some(20),
        hide_resting_parts: Some(true),
        directive_row_offset: Some(Offset { x: 1, y: 2 }),
        ..Default::default()
    };
    let overrides = ParsedMetadata {
        subtitle: Some("Choir".to_string()),
        row_height: Some(30),
        ..Default::default()
    };
    let merged = base.overlaid_with(&overrides);
    assert_eq!(merged.title.as_deref(), Some("Song"));
    assert_eq!(merged.subtitle.as_deref(), Some("Choir"));
    assert_eq!(merged.row_height, Some(30));
    assert_eq!(merged.hide_resting_parts, Some(true));
    assert_eq!(merged.directive_row_offset, Some(Offset { x: 1, y: 2 }));
}

#[test]
fn overlay_merges_text_styles_one_component_at_a_time() {
    let base = ParsedMetadata {
        title_style: TextStyle {
            font_size: Some(20),
            bold: Some(true),
            ..Default::default()
        },
        ..Default::default()
    };
    let overrides = ParsedMetadata {
        title_style: TextStyle {
            font_size: Some(30),
            font_family: Some(FontFamilyChoice::SansSerif),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(
        base.overlaid_with(&overrides).title_style,
        TextStyle {
            font_size: Some(30),
            bold: Some(true),
            font_family: Some(FontFamilyChoice::SansSerif),
            ..Default::default()
        }
    );
}

#[test]
fn sets_any_text_style_only_counts_style_objects() {
    let text_only = ParsedMetadata {
        title: Some("Song".to_string()),
        row_height: Some(30),
        ..Default::default()
    };
    let styled = ParsedMetadata {
        lyrics_style: TextStyle {
            italic: Some(true),
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(!text_only.sets_any_text_style());
    assert!(styled.sets_any_text_style());
}

#[test]
fn only_staff_notations_ignore_text_styles() {
    let ignoring: Vec<Notation> = Notation::ALL
        .iter()
        .copied()
        .filter(|notation| notation.ignores_text_styles())
        .collect();
    assert_eq!(ignoring, [Notation::Violin, Notation::Cello]);
}

fn document(source: &str) -> ParsedDocument {
    crate::parser::parse(source, "test.poentaro", &[]).expect("parses")
}

const SONG_WITH_CHOIR_PRESET: &str = r#"metadata { title "Song", subtitle "Base" },

parts ["Main" [main]: notes, "Soprano" [S1]: notes],

score """
    [main] 1
    [S1] 2
    """,

preset "Choir" { visible parts [S1, main], subtitle "Choir" },
"#;

#[test]
fn with_preset_overlays_the_chosen_preset() {
    let metadata = document(SONG_WITH_CHOIR_PRESET)
        .with_preset("Choir")
        .metadata;
    assert_eq!(metadata.title.as_deref(), Some("Song"));
    assert_eq!(metadata.subtitle.as_deref(), Some("Choir"));
}

#[test]
fn with_an_unknown_preset_keeps_the_song_metadata() {
    let metadata = document(SONG_WITH_CHOIR_PRESET)
        .with_preset("Gone")
        .metadata;
    assert_eq!(metadata.subtitle.as_deref(), Some("Base"));
}

#[test]
fn visible_abbreviations_keep_the_written_order() {
    let doc = document(SONG_WITH_CHOIR_PRESET);
    assert_eq!(
        doc.preset("Choir")
            .and_then(ParsedPreset::visible_abbreviations),
        Some(vec!["S1".to_string(), "main".to_string()])
    );
}
