use super::metadata_defaults;
use crate::ast::parsed::{ParsedMetadata, TextStyleKind};

fn defaults_of(source: &str) -> crate::ast::grouped::Metadata {
    let read = crate::parser::read_document(source, &[]);
    assert!(
        read.metadata_parse_errors.is_empty(),
        "{:?}",
        read.metadata_parse_errors
    );
    metadata_defaults(&ParsedMetadata::default(), &read.metadata)
}

fn font_size(metadata: &crate::ast::grouped::Metadata, kind: TextStyleKind) -> u32 {
    metadata.style(kind).font_size
}

#[test]
fn row_height_scaled_defaults_follow_an_explicit_row_height() {
    let unset = metadata_defaults(&ParsedMetadata::default(), &ParsedMetadata::default());
    let scaled = defaults_of(
        r#"metadata { row height 40 }
"#,
    );
    assert_eq!(font_size(&unset, TextStyleKind::Title), 36);
    assert_eq!(font_size(&scaled, TextStyleKind::Title), 60);
    assert_eq!(font_size(&scaled, TextStyleKind::Lyrics), 24);
    assert_eq!(font_size(&scaled, TextStyleKind::Sequence), 12);
    assert_eq!(
        scaled.row_height, 24,
        "row_height's own default ignores its value"
    );
}

#[test]
fn notes_and_chords_follow_lyrics_and_note_dash_follows_notes() {
    let defaults = defaults_of(
        r#"metadata { lyrics style { font size 20 }, notes style { font size 30 } }
"#,
    );
    assert_eq!(font_size(&defaults, TextStyleKind::Lyrics), 14);
    assert_eq!(font_size(&defaults, TextStyleKind::Notes), 20);
    assert_eq!(font_size(&defaults, TextStyleKind::Chords), 20);
    assert_eq!(font_size(&defaults, TextStyleKind::NoteDash), 30);
}

#[test]
fn a_kinds_own_components_do_not_leak_into_its_default() {
    let defaults = defaults_of(
        r#"metadata { title style { bold: yes, font size 50 } }
"#,
    );
    assert!(!defaults.title_style.bold);
    assert_eq!(defaults.title_style.font_size, 36);
}

#[test]
fn row_height_percent_defaults_round_to_the_nearest_point() {
    let defaults = defaults_of(
        r#"metadata { row height 25 }
"#,
    );
    assert_eq!(font_size(&defaults, TextStyleKind::Title), 38);
    assert_eq!(font_size(&defaults, TextStyleKind::Subtitle), 20);
    assert_eq!(font_size(&defaults, TextStyleKind::Lyrics), 15);
}
