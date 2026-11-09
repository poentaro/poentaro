use crate::ast::parsed::{Notation, ParsedDocument, ParsedPreset};
use crate::error::{Diagnostic, RecoverableErrorKind, WarningKind};
use crate::parser::parse;

fn document_with_presets(presets: &str) -> String {
    format!(
        r#"metadata {{ title "Song", subtitle "Base" }},

parts [
    "Main" [main] : notes,
    "Soprano" [S1] : notes,
    "Alto" [A1] : notes,
],

{presets}

score """
    [main] 1
    [S1] 2
    [A1] 3
    """,
"#
    )
}

fn parse_document(source: &str) -> ParsedDocument {
    parse(source, "test.poentaro", &[]).expect("presets never abort parsing")
}

fn only_preset(doc: &ParsedDocument) -> &ParsedPreset {
    match doc.presets.as_slice() {
        [preset] => preset,
        presets => panic!("expected one preset, got {}", presets.len()),
    }
}

fn error_kinds(doc: &ParsedDocument) -> Vec<RecoverableErrorKind> {
    doc.preset_diagnostics
        .iter()
        .filter_map(|diagnostic| match diagnostic {
            Diagnostic::Error(error) => Some(error.kind.clone()),
            Diagnostic::Warning(_) => None,
        })
        .collect()
}

fn text_at(source: &str, span: crate::error::Span) -> &str {
    source.get(span.start..span.end).unwrap_or_default()
}

#[test]
fn parses_name_visible_parts_notation_and_overrides() {
    let source = document_with_presets(
        "preset \"Soprano 1\" { visible parts [main, S1], notation : violin, subtitle \"Soprano 1\", max measures per system : 3 },",
    );
    let doc = parse_document(&source);
    let preset = only_preset(&doc);

    assert!(doc.preset_diagnostics.is_empty());
    assert_eq!(preset.name, "Soprano 1");
    assert_eq!(text_at(&source, preset.name_span), "\"Soprano 1\"");
    assert_eq!(text_at(&source, preset.header_span), "preset \"Soprano 1\"");
    let parts: Vec<&str> = preset
        .visible_parts
        .iter()
        .flatten()
        .map(|reference| text_at(&source, reference.span))
        .collect();
    assert_eq!(parts, ["main", "S1"]);
    assert_eq!(preset.notation, Some(Notation::Violin));
    assert_eq!(preset.overrides.subtitle.as_deref(), Some("Soprano 1"));
    assert_eq!(preset.overrides.max_measures_per_system, Some(3));
    assert_eq!(preset.overrides.title, None);
}

#[test]
fn preset_does_not_change_the_song_metadata() {
    let source = document_with_presets("preset \"Choir\" { subtitle \"Choir\" },");
    let doc = parse_document(&source);
    assert_eq!(doc.metadata.subtitle.as_deref(), Some("Base"));
}

#[test]
fn preset_without_visible_parts_or_notation_leaves_them_unset() {
    let doc = parse_document(&document_with_presets(
        "preset \"Big\" { row height : 40 },",
    ));
    let preset = only_preset(&doc);
    assert!(preset.visible_parts.is_none());
    assert_eq!(preset.notation, None);
}

#[test]
fn empty_visible_parts_hides_every_part() {
    let doc = parse_document(&document_with_presets(
        "preset \"None\" { visible parts [] },",
    ));
    assert_eq!(only_preset(&doc).visible_parts.as_deref(), Some(&[][..]));
}

#[test]
fn visible_parts_skips_repeats() {
    let doc = parse_document(&document_with_presets(
        "preset \"P\" { visible parts [S1, S1, A1] },",
    ));
    let parts: Vec<&str> = only_preset(&doc)
        .visible_parts
        .iter()
        .flatten()
        .map(|reference| reference.abbreviation.as_str())
        .collect();
    assert!(doc.preset_diagnostics.is_empty());
    assert_eq!(parts, ["S1", "A1"]);
}

#[test]
fn unknown_part_is_reported_at_its_abbreviation_and_dropped() {
    let source = document_with_presets("preset \"P\" { visible parts [S1, T9] },");
    let doc = parse_document(&source);
    let parts: Vec<&str> = only_preset(&doc)
        .visible_parts
        .iter()
        .flatten()
        .map(|reference| reference.abbreviation.as_str())
        .collect();
    assert_eq!(parts, ["S1"]);
    match doc.preset_diagnostics.as_slice() {
        [Diagnostic::Error(error)] => {
            assert_eq!(
                error.kind,
                RecoverableErrorKind::PresetUnknownPart {
                    abbreviation: "T9".to_string()
                }
            );
            assert_eq!(text_at(&source, error.span), "T9");
        }
        diagnostics => panic!("expected one error, got {diagnostics:?}"),
    }
}

#[test]
fn unknown_notation_is_an_error() {
    let doc = parse_document(&document_with_presets("preset \"P\" { notation : lute },"));
    assert_eq!(only_preset(&doc).notation, None);
    assert!(matches!(
        error_kinds(&doc).as_slice(),
        [RecoverableErrorKind::MetadataInvalidEnum { field, value, .. }]
            if field == "notation" && value == "lute"
    ));
}

#[test]
fn every_notation_keyword_parses() {
    for notation in Notation::ALL {
        let doc = parse_document(&document_with_presets(&format!(
            "preset \"P\" {{ notation : {} }},",
            notation.keyword()
        )));
        assert_eq!(only_preset(&doc).notation, Some(*notation));
    }
}

#[test]
fn the_legacy_jianpu_keyword_reads_as_numbered() {
    let doc = parse_document(&document_with_presets(
        "preset \"P\" { notation : jianpu },",
    ));
    assert_eq!(only_preset(&doc).notation, Some(Notation::Numbered));
    assert!(error_kinds(&doc).is_empty());
}

#[test]
fn unknown_key_is_an_error() {
    let doc = parse_document(&document_with_presets("preset \"P\" { colour : red },"));
    assert!(matches!(
        error_kinds(&doc).as_slice(),
        [RecoverableErrorKind::MetadataUnknownField { field }] if field == "colour"
    ));
}

#[test]
fn duplicate_name_keeps_the_first_preset() {
    let source = document_with_presets(
        "preset \"P\" { row height : 10 },\n\npreset \"P\" { row height : 20 },",
    );
    let doc = parse_document(&source);
    assert_eq!(only_preset(&doc).overrides.row_height, Some(10));
    match doc.preset_diagnostics.as_slice() {
        [Diagnostic::Error(error)] => {
            assert_eq!(
                error.kind,
                RecoverableErrorKind::PresetDuplicateName {
                    name: "P".to_string()
                }
            );
            assert_eq!(error.span.start, source.rfind("\"P\"").unwrap_or_default());
        }
        diagnostics => panic!("expected one error, got {diagnostics:?}"),
    }
}

#[test]
fn default_is_a_reserved_name_in_any_case() {
    for name in ["Default", "default"] {
        let doc = parse_document(&document_with_presets(&format!(
            "preset \"{name}\" {{ row height : 10 }},"
        )));
        assert!(doc.presets.is_empty());
        assert_eq!(
            error_kinds(&doc),
            [RecoverableErrorKind::PresetReservedName {
                name: name.to_string()
            }]
        );
    }
}

#[test]
fn preset_only_keys_are_errors_in_metadata() {
    let source = "metadata { visible parts : main, notation : violin },\n\nparts [\"Main\" [main] : notes],\n\nscore \"\"\"\n[main] 1\n\"\"\",\n";
    let doc = parse_document(source);
    let kinds: Vec<RecoverableErrorKind> = doc
        .metadata_parse_errors
        .iter()
        .map(|error| error.kind.clone())
        .collect();
    assert_eq!(
        kinds,
        [
            RecoverableErrorKind::PresetOnlyField {
                field: "visible parts".to_string()
            },
            RecoverableErrorKind::PresetOnlyField {
                field: "notation".to_string()
            },
        ]
    );
}

#[test]
fn staff_notation_with_text_styles_warns_at_the_notation() {
    let source = document_with_presets(
        "preset \"P\" { notation : cello, title style : { font size : 30 } },",
    );
    let doc = parse_document(&source);
    match doc.preset_diagnostics.as_slice() {
        [Diagnostic::Warning(warning)] => {
            assert_eq!(warning.kind, WarningKind::PresetTextStylesIgnored);
            assert_eq!(text_at(&source, warning.span), "cello");
        }
        diagnostics => panic!("expected one warning, got {diagnostics:?}"),
    }
}

#[test]
fn staff_notation_without_text_styles_does_not_warn() {
    let doc = parse_document(&document_with_presets(
        "preset \"P\" { notation : violin, subtitle \"Violin\", row height : 30 },",
    ));
    assert!(doc.preset_diagnostics.is_empty());
}

#[test]
fn numbered_notation_with_text_styles_does_not_warn() {
    let doc = parse_document(&document_with_presets(
        "preset \"P\" { notation : numbered, title style : { font size : 30 } },",
    ));
    assert!(doc.preset_diagnostics.is_empty());
}

#[test]
fn soloed_parts_parses_like_visible_parts() {
    let source = document_with_presets("preset \"P\" { soloed parts [A1, A1, T9] },");
    let doc = parse_document(&source);
    assert_eq!(only_preset(&doc).soloed_abbreviations(), vec!["A1"]);
    assert_eq!(
        error_kinds(&doc),
        [RecoverableErrorKind::PresetUnknownPart {
            abbreviation: "T9".to_string()
        }]
    );
}

#[test]
fn preset_without_soloed_parts_solos_nothing() {
    let doc = parse_document(&document_with_presets(
        "preset \"P\" { notation : violin },",
    ));
    assert!(only_preset(&doc).soloed_parts.is_empty());
}
