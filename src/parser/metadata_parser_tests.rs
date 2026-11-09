use super::*;

pub(super) fn parse_metadata(source: &str) -> (ParsedMetadata, Vec<RecoverableError>) {
    let read = crate::parser::read_document(source, &[]);
    (read.metadata, read.metadata_parse_errors)
}

#[test]
fn parses_title_and_author() {
    let content = r#"metadata { title "hello world", author "foo" }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.title, Some("hello world".to_string()));
    assert_eq!(meta.author, Some("foo".to_string()));
    assert_eq!(meta.row_height, None);
    assert_eq!(meta.max_measures_per_system, None);
}

#[test]
fn parses_optional_row_height() {
    let content = r#"metadata { title "t", author "a", row height 16 }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.row_height, Some(16));
}

#[test]
fn parses_optional_part_label_width_pt_and_page_margin_pt() {
    let content = r#"metadata { title "t", part label width pt 60, page margin pt 40 }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.part_label_width_pt, Some(60));
    assert_eq!(meta.page_margin_pt, Some(40));
}

#[test]
fn part_label_width_pt_is_not_a_text_style_object_field() {
    let content = r#"metadata { title "t", author "a", part label style : { width pt : 60 } }
"#;
    let (_meta, errors) = parse_metadata(content);
    assert!(errors.iter().any(|e| e
        .message()
        .contains("unknown metadata field: part label style width pt")));
}

#[test]
fn parses_optional_max_measures_per_system() {
    let content = r#"metadata { title "t", author "a", max measures per system 6 }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.max_measures_per_system, Some(6));
}

#[test]
fn title_is_optional() {
    let content = r#"metadata { author "foo" }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.title, None);
}

#[test]
fn author_is_optional() {
    let content = r#"metadata { title "foo" }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.author, None);
}

#[test]
fn collects_error_for_unknown_field() {
    let content = r#"metadata { title "t", author "a", foo "bar" }
"#;
    let (_meta, errors) = parse_metadata(content);
    assert!(!errors.is_empty());
}

#[test]
fn collects_error_for_parts_field_in_metadata() {
    let content = r#"metadata { title "t", author "a", parts "notes: lyrics:" }
"#;
    let (_meta, errors) = parse_metadata(content);
    assert!(errors
        .iter()
        .any(|e| e.message().contains("unknown metadata field: parts")));
}

#[test]
fn collects_error_for_invalid_row_height() {
    let content = r#"metadata { title "t", author "a", row height: abc }
"#;
    let (_meta, errors) = parse_metadata(content);
    assert!(!errors.is_empty());
}

#[test]
fn invalid_value_span_covers_only_the_value() {
    let content = "metadata { title \"t\", row height : 20k },\n";
    let (_meta, errors) = parse_metadata(content);
    assert_eq!(errors.len(), 1);
    let span = errors[0].span;
    let spanned = &content[span.start..span.end];
    assert_eq!(spanned, "20k");
}

#[test]
fn collects_error_for_invalid_max_measures_per_system() {
    let content = r#"metadata { title "t", author "a", max measures per system 0 }
"#;
    let (_meta, errors) = parse_metadata(content);
    assert!(!errors.is_empty());
}

#[test]
fn parses_optional_subtitle() {
    let content = r#"metadata { title "hello", author "foo", subtitle "sub" }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.subtitle, Some("sub".to_string()));
}

#[test]
fn subtitle_defaults_to_none() {
    let content = r#"metadata { title "t", author "a" }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.subtitle, None);
}

#[test]
fn parses_lyrics_font_size() {
    let content = r#"metadata { title "t", author "a", lyrics style { font size 14 } }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.lyrics_style.font_size, Some(14));
}

#[test]
fn lyrics_font_size_defaults_to_none() {
    let content = r#"metadata { title "t", author "a" }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.lyrics_style.font_size, None);
}

#[test]
fn old_flat_lyrics_font_size_key_is_rejected() {
    let content = r#"metadata { title "t", author "a", lyrics font size : 14 }
"#;
    let (_meta, errors) = parse_metadata(content);
    assert!(errors.iter().any(|e| e
        .message()
        .contains("unknown metadata field: lyrics font size")));
}

#[test]
fn parses_measure_number_font_size() {
    let content = r#"metadata { title "t", author "a", measure number style { font size 8 } }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.measure_number_style.font_size, Some(8));
}

#[test]
fn measure_number_font_size_defaults_to_none() {
    let content = r#"metadata { title "t", author "a" }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.measure_number_style.font_size, None);
}

#[test]
fn parses_section_label_font_size() {
    let content = r#"metadata { title "t", author "a", section label style { font size 16 } }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.section_label_style.font_size, Some(16));
}

#[test]
fn section_label_font_size_defaults_to_none() {
    let content = r#"metadata { title "t", author "a" }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.section_label_style.font_size, None);
}

#[test]
fn parses_part_label_font_size() {
    let content = r#"metadata { title "t", author "a", part label style { font size 16 } }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.part_label_style.font_size, Some(16));
}

#[test]
fn part_label_font_size_defaults_to_none() {
    let content = r#"metadata { title "t", author "a" }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.part_label_style.font_size, None);
}

#[test]
fn parses_page_number_font_size() {
    let content = r#"metadata { title "t", author "a", page number style { font size 9 } }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.page_number_style.font_size, Some(9));
}

#[test]
fn page_number_font_size_defaults_to_none() {
    let content = r#"metadata { title "t", author "a" }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.page_number_style.font_size, None);
}

#[test]
fn parses_lyric_click_target_padding_pt() {
    let content = r#"metadata { title "t", author "a", lyrics style { vertical padding pt 24 } }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.lyrics_style.vertical_padding_pt, Some(24));
}

#[test]
fn lyric_click_target_padding_pt_defaults_to_none() {
    let content = r#"metadata { title "t", author "a" }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.lyrics_style.vertical_padding_pt, None);
}

#[test]
fn old_flat_lyric_click_target_padding_pt_key_is_rejected() {
    let content = r#"metadata { title "t", author "a", lyric click target padding pt : 24 }
"#;
    let (_meta, errors) = parse_metadata(content);
    assert!(errors.iter().any(|e| e
        .message()
        .contains("unknown metadata field: lyric click target padding pt")));
}

#[test]
fn old_flat_notes_horizontal_padding_pt_key_is_rejected() {
    let content = r#"metadata { title "t", author "a", notes horizontal padding pt : 4 }
"#;
    let (_meta, errors) = parse_metadata(content);
    assert!(errors.iter().any(|e| e
        .message()
        .contains("unknown metadata field: notes horizontal padding pt")));
}

#[test]
fn parses_merge_duplicate_measures_across_parts() {
    let content = r#"metadata { title "t", author "a", merge duplicate measures across parts: no }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.merge_duplicate_measures_across_parts, Some(false));
}

#[test]
fn merge_duplicate_measures_across_parts_defaults_to_none() {
    let content = r#"metadata { title "t", author "a" }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.merge_duplicate_measures_across_parts, None);
}

#[test]
fn collects_error_for_invalid_merge_duplicate_measures_across_parts() {
    let content = r#"metadata { title "t", author "a", merge duplicate measures across parts: maybe }
"#;
    let (_meta, errors) = parse_metadata(content);
    assert!(!errors.is_empty());
}

#[test]
fn parses_hide_resting_parts() {
    let content = r#"metadata { title "t", author "a", hide resting parts: no }
"#;
    let (meta, errors) = parse_metadata(content);
    assert!(errors.is_empty());
    assert_eq!(meta.hide_resting_parts, Some(false));
}
