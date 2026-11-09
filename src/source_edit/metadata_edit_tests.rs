use super::*;
use crate::ast::parsed::{FontFamilyChoice, Offset};

const SOURCE_WITH_METADATA: &str = r#"metadata { title "My Song", author "Alice" },

score """
    1 2 3 4
    """,
"#;

fn text_edit(key: MetadataTextKey, value: Option<&str>) -> MetadataEdit {
    MetadataEdit::Text {
        key,
        value: value.map(str::to_owned),
    }
}

#[test]
fn adds_a_field_to_existing_metadata() {
    let result = update_metadata_field(
        SOURCE_WITH_METADATA,
        None,
        text_edit(MetadataTextKey::Subtitle, Some("A Subtitle")),
    );
    assert!(result.contains(r#"subtitle "A Subtitle""#));
}

#[test]
fn replaces_an_existing_field() {
    let result = update_metadata_field(
        SOURCE_WITH_METADATA,
        None,
        text_edit(MetadataTextKey::Title, Some("New Title")),
    );
    assert!(result.contains(r#"title "New Title""#));
    assert!(!result.contains("My Song"));
}

#[test]
fn removes_a_field_set_to_none_or_empty() {
    for value in [None, Some("")] {
        let result = update_metadata_field(
            SOURCE_WITH_METADATA,
            None,
            text_edit(MetadataTextKey::Author, value),
        );
        assert!(!result.contains("author"), "{value:?}: {result}");
    }
}

#[test]
fn rewrites_the_section_canonically_with_one_trailing_blank_line() {
    let source = r#"metadata { author "Bob", title "Song", row height 80, subtitle "Sub" },

score """
    1 2 3
    """,
"#;
    let result = update_metadata_field(
        source,
        None,
        MetadataEdit::Number {
            key: MetadataNumberKey::MaxMeasuresPerSystem,
            value: Some(4),
        },
    );
    assert_eq!(
        result,
        r#"metadata { title "Song", subtitle "Sub", author "Bob", row height 80, max measures per system 4 },

score """
    1 2 3
    """,
"#
    );
}

#[test]
fn leaves_other_sections_untouched() {
    let source = r#"parts ["M": notes],

metadata { title "Song" },

score """
    1 2
    """,
"#;
    let result = update_metadata_field(
        source,
        None,
        text_edit(MetadataTextKey::Title, Some("Changed")),
    );
    assert!(result.starts_with(
        r#"parts ["M": notes],

metadata { title "Changed" },
"#
    ));
    assert!(result.ends_with(
        r#"score """
    1 2
    """,
"#
    ));
}

#[test]
fn returns_the_source_unchanged_without_a_metadata_section() {
    let source = r#"score """
    1 2
    """
"#;
    let result = update_metadata_field(source, None, text_edit(MetadataTextKey::Title, Some("x")));
    assert_eq!(result, source);
}

#[test]
fn writes_into_a_metadata_header_on_the_last_line() {
    let result = update_metadata_field(
        r#"metadata {}
"#,
        None,
        text_edit(MetadataTextKey::Title, Some("x")),
    );
    assert_eq!(
        result,
        r#"metadata { title "x" }
"#
    );
}

#[test]
fn formats_flags_as_yes_or_no() {
    let result = update_metadata_field(
        SOURCE_WITH_METADATA,
        None,
        MetadataEdit::Flag {
            key: MetadataFlagKey::HideSystemDividers,
            value: Some(true),
        },
    );
    assert!(result.contains("hide system dividers: yes"));
    let result = update_metadata_field(
        &result,
        None,
        MetadataEdit::Flag {
            key: MetadataFlagKey::MergeDuplicateMeasuresAcrossParts,
            value: Some(false),
        },
    );
    assert!(result.contains("merge duplicate measures across parts: no"));
}

#[test]
fn round_trips_style_components_in_component_order() {
    let with_italic = update_metadata_field(
        SOURCE_WITH_METADATA,
        None,
        MetadataEdit::Style {
            kind: TextStyleKind::Title,
            value: TextStyleComponentValue::Italic(Some(true)),
        },
    );
    let result = update_metadata_field(
        &with_italic,
        None,
        MetadataEdit::Style {
            kind: TextStyleKind::Title,
            value: TextStyleComponentValue::FontSize(Some(32)),
        },
    );
    assert!(result.contains("title style { font size 32, italic: yes }"));
}

#[test]
fn clears_a_style_component_and_drops_an_empty_style_line() {
    let with_family = update_metadata_field(
        SOURCE_WITH_METADATA,
        None,
        MetadataEdit::Style {
            kind: TextStyleKind::PartLegend,
            value: TextStyleComponentValue::FontFamily(Some(FontFamilyChoice::SansSerif)),
        },
    );
    assert!(with_family.contains("part legend style { font family: sans serif }"));
    let result = update_metadata_field(
        &with_family,
        None,
        MetadataEdit::Style {
            kind: TextStyleKind::PartLegend,
            value: TextStyleComponentValue::FontFamily(None),
        },
    );
    assert!(!result.contains("part legend"));
}

#[test]
fn emits_every_key_group_in_canonical_order() {
    let source = r#"metadata {
    directive row offset (0, 12),
    hide resting parts: no,
    lyrics style { vertical padding pt 20 },
    part label width pt 60,
    title style { bold: yes },
    title "Song",
},

score """
    1
    """,
"#;
    let result =
        update_metadata_field(source, None, text_edit(MetadataTextKey::Author, Some("Me")));
    assert!(
        result.starts_with(
            r#"metadata {
    title "Song",
    title style { bold: yes },
    author "Me",
    part label width pt 60,
    lyrics style { vertical padding pt 20 },
    hide resting parts: no,
    directive row offset (0, 12),
},"#
        ),
        "{result}"
    );
}

#[test]
fn keeps_an_unparseable_directive_row_offset_verbatim() {
    let source = r#"metadata { directive row offset (0, -) },

score """
    1
    """,
"#;
    let fields = parse_metadata_fields(source, None);
    assert_eq!(fields.directive_row_offset.as_deref(), Some("0 -"));
    assert_eq!(fields.metadata.directive_row_offset, None);
    let result = update_metadata_field(source, None, text_edit(MetadataTextKey::Title, Some("t")));
    assert!(result.contains("directive row offset (0, -)"));
}

#[test]
fn parses_every_field_kind() {
    let source = r#"metadata {
    title "Song",
    row height 30,
    hide system dividers: yes,
    directive row offset (- 3, 12),
    notes style { font family: serif, underline: no },
},

score """
    1
    """,
"#;
    let MetadataFields {
        metadata,
        directive_row_offset,
        ..
    } = parse_metadata_fields(source, None);
    assert_eq!(metadata.title.as_deref(), Some("Song"));
    assert_eq!(metadata.row_height, Some(30));
    assert_eq!(metadata.hide_system_dividers, Some(true));
    assert_eq!(metadata.directive_row_offset, Some(Offset { x: -3, y: 12 }));
    assert_eq!(directive_row_offset.as_deref(), Some("-3 12"));
    assert_eq!(
        metadata.notes_style.font_family,
        Some(FontFamilyChoice::Serif)
    );
    assert_eq!(metadata.notes_style.underline, Some(false));
}

const SOURCE_WITH_PRESET: &str = r#"metadata { title "My Song", row height 30 },

preset "Choir" { visible parts [S, A], notation: violin, title "My Song (Choir)" },

score """
    1 2 3 4
    """,
"#;

#[test]
fn reads_a_presets_overrides_with_the_songs_metadata_as_inherited() {
    let fields = parse_metadata_fields(SOURCE_WITH_PRESET, Some("Choir"));
    assert_eq!(fields.metadata.title.as_deref(), Some("My Song (Choir)"));
    assert_eq!(fields.metadata.row_height, None);
    assert_eq!(fields.inherited.title.as_deref(), Some("My Song"));
    assert_eq!(fields.inherited.row_height, Some(30));
}

#[test]
fn edits_a_preset_keeping_its_parts_and_notation_and_the_songs_metadata() {
    let edit = MetadataEdit::Number {
        key: MetadataNumberKey::RowHeight,
        value: Some(40),
    };
    let result = update_metadata_field(SOURCE_WITH_PRESET, Some("Choir"), edit);
    assert_eq!(
        result,
        SOURCE_WITH_PRESET.replace(
            r#"title "My Song (Choir)" }"#,
            r#"title "My Song (Choir)", row height 40 }"#,
        )
    );
}

#[test]
fn resetting_a_presets_field_removes_only_its_override() {
    let result = update_metadata_field(
        SOURCE_WITH_PRESET,
        Some("Choir"),
        text_edit(MetadataTextKey::Title, None),
    );
    assert_eq!(
        result,
        SOURCE_WITH_PRESET.replace(r#", title "My Song (Choir)""#, "")
    );
}

#[test]
fn reset_style_removes_every_component_of_that_style() {
    let source = r#"metadata { title "t", title style { font size 9, bold: yes } },

score """
    1
    """,
"#;
    let result =
        update_metadata_field(source, None, MetadataEdit::ResetStyle(TextStyleKind::Title));
    assert_eq!(
        result,
        r#"metadata { title "t" },

score """
    1
    """,
"#
    );
}
