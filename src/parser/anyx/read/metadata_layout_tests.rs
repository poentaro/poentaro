use super::metadata_tests::parse_metadata;
use super::*;

#[test]
fn hide_resting_parts_defaults_to_none() {
    let content = r#"metadata { title "t", author "a" }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.hide_resting_parts, None);
}

#[test]
fn collects_error_for_invalid_hide_resting_parts() {
    let content = r#"metadata { title "t", author "a", hide resting parts: maybe }
"#;
    let (_meta, errors) = parse_metadata(content);
    assert!(!errors.is_empty());
}

#[test]
fn parses_hide_system_dividers() {
    let content = r#"metadata { title "t", author "a", hide system dividers: yes }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.hide_system_dividers, Some(true));
}

#[test]
fn hide_system_dividers_defaults_to_none() {
    let content = r#"metadata { title "t", author "a" }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.hide_system_dividers, None);
}

#[test]
fn collects_error_for_invalid_hide_system_dividers() {
    let content = r#"metadata { title "t", author "a", hide system dividers: maybe }
"#;
    let (_meta, errors) = parse_metadata(content);
    assert!(!errors.is_empty());
}

#[test]
fn parses_directive_row_offset() {
    let content = r#"metadata { title "t", author "a", directive row offset (0, 12) }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.directive_row_offset, Some(Offset { x: 0, y: 12 }));
}

#[test]
fn parses_directive_row_offset_with_negative_values() {
    let content = r#"metadata { title "t", author "a", directive row offset (- 5, - 12) }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.directive_row_offset, Some(Offset { x: -5, y: -12 }));
}

#[test]
fn directive_row_offset_defaults_to_none() {
    let content = r#"metadata { title "t", author "a" }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.directive_row_offset, None);
}

#[test]
fn collects_error_for_invalid_directive_row_offset() {
    let content = r#"metadata { title "t", author "a", directive row offset: twelve }
"#;
    let (_meta, errors) = parse_metadata(content);
    assert!(!errors.is_empty());
}

#[test]
fn collects_error_for_directive_row_offset_with_too_many_values() {
    let content = r#"metadata { title "t", author "a", directive row offset "1 2 3" }
"#;
    let (_meta, errors) = parse_metadata(content);
    assert!(!errors.is_empty());
}

#[test]
fn collects_error_for_old_section_label_offset_key() {
    let content = r#"metadata { title "t", author "a", section label offset "0 12" }
"#;
    let (_meta, errors) = parse_metadata(content);
    assert!(!errors.is_empty());
}

#[test]
fn parses_font_family_on_lyrics() {
    let content = r#"metadata { title "t", author "a", lyrics style { font family: sans serif } }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(
        meta.lyrics_style.font_family,
        Some(FontFamilyChoice::SansSerif)
    );
}

#[test]
fn collects_error_for_invalid_font_family_value() {
    let content = r#"metadata { title "t", author "a", lyrics style { font family: comic sans } }
"#;
    let (meta, errors) = parse_metadata(content);
    assert_eq!(meta.lyrics_style.font_family, None);
    assert!(errors
        .iter()
        .any(|e| e.message().contains("lyrics style font family")));
}

#[test]
fn parses_font_family_on_notes_chords_and_note_dash() {
    use crate::ast::parsed::FontFamilyChoice;

    let content = r#"metadata {
    title "t",
    author "a",
    notes style { font family: serif },
    chords style { font family: sans serif },
    note dash style { font family: serif },
}
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    assert_eq!(meta.notes_style.font_family, Some(FontFamilyChoice::Serif));
    assert_eq!(
        meta.chords_style.font_family,
        Some(FontFamilyChoice::SansSerif)
    );
    assert_eq!(
        meta.note_dash_style.font_family,
        Some(FontFamilyChoice::Serif)
    );
}
