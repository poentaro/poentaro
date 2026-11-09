use crate::ast::parsed::PartKind;
use crate::error::RecoverableErrorKind;

pub(super) fn parse_parts(
    source: &str,
    instruments: &[super::InstrumentInfo],
) -> (
    Vec<crate::ast::parsed::PartDecl>,
    Vec<crate::error::RecoverableError>,
) {
    let read = crate::parser::read_document(source, instruments);
    (read.declarations, read.parts_parse_errors)
}

#[test]
fn parses_abbreviated_track() {
    let content = r#"parts ["Alto 1 & Tenor" [A1 & T]: notes]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].display_name, "Alto 1 & Tenor");
    assert_eq!(decls[0].abbreviation, "A1&T");
    assert_eq!(decls[0].kind, PartKind::Notes);
    assert_eq!(decls[0].follow_target, None);
}

#[test]
fn parses_chord_track() {
    let content = r#"parts ["main": chords]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls[0].abbreviation, "main");
    assert_eq!(decls[0].display_name, "main");
    assert_eq!(decls[0].kind, PartKind::Chords);
    assert_eq!(decls[0].follow_target, None);
}

#[test]
fn omits_brackets_uses_name_as_abbreviation() {
    let content = r#"parts ["Melody": notes]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls[0].abbreviation, "Melody");
    assert_eq!(decls[0].display_name, "Melody");
}

#[test]
fn skips_duplicate_abbreviation_and_keeps_first() {
    let content = "parts [\"A\" [x] : notes, \"B\" [x] : notes],\n";
    let (decls, errors) = parse_parts(content, &[]);
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].display_name, "A");
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::PartsDuplicateAbbreviation { .. }
    ));
}

#[test]
fn empty_section_collects_error() {
    let (decls, errors) = parse_parts("parts [],\n", &[]);
    assert!(decls.is_empty());
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::PartsEmptySection
    ));
}

#[test]
fn skips_malformed_line_and_collects_error() {
    let content = "parts [ title ],\n";
    let (decls, errors) = parse_parts(content, &[]);
    assert!(decls.is_empty());
    assert!(!errors.is_empty());
}

#[test]
fn skips_bad_line_keeps_valid_declaration() {
    let content = "parts [malformed, \"Melody\" : notes],\n";
    let (decls, errors) = parse_parts(content, &[]);
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].abbreviation, "Melody");
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::AnyfixInvalid { .. }
    ));
}

#[test]
fn follow_copies_kind_from_target_and_sets_follow_target() {
    let content = r#"parts ["Soprano" [S]: notes, "Alto" [A]: follow [S]]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls.len(), 2);
    assert_eq!(decls[1].abbreviation, "A");
    assert_eq!(decls[1].kind, PartKind::Notes);
    assert_eq!(decls[1].follow_target, Some("S".to_string()));
}

#[test]
fn follow_first_part_emits_error() {
    let content = r#"parts ["Soprano" [S]: follow [X], "Alto" [A]: notes]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].abbreviation, "A");
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::PartsFirstPartCannotFollow
    ));
}

#[test]
fn follow_with_soundfont_parses_correctly() {
    use crate::ast::parsed::Soundfont;
    let content = r#"parts ["A": notes, "B": follow [A] { sound 1 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls.len(), 2);
    assert_eq!(decls[1].follow_target, Some("A".to_string()));
    assert_eq!(decls[1].soundfont, Soundfont(1));
}

#[test]
fn follow_unknown_target_emits_error() {
    let content = r#"parts ["Soprano" [S]: notes, "Alto" [A]: follow [UNKNOWN]]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].abbreviation, "S");
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::PartsFollowUnknownTarget { ref target } if target == "UNKNOWN"
    ));
    assert_eq!(
        errors[0].message(),
        "no part with abbreviation 'UNKNOWN'; follow targets must match a part declared earlier in parts"
    );
    let unknown_start = content.find("UNKNOWN").expect("UNKNOWN in source");
    assert_eq!(errors[0].span.start, unknown_start);
    assert_eq!(errors[0].span.end, unknown_start + "UNKNOWN".len());
}

#[test]
fn follow_unknown_target_span_is_specific_for_simple_names() {
    let content = r#"parts ["a": notes, "b": follow [xxx]],
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert_eq!(decls.len(), 1);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::PartsFollowUnknownTarget { ref target } if target == "xxx"
    ));
    let xxx_start = content.find("xxx").expect("xxx in source");
    assert_eq!(errors[0].span.start, xxx_start);
    assert_eq!(errors[0].span.end, xxx_start + "xxx".len());
}

#[test]
fn octave_offset_positive_parses() {
    let content = r#"parts ["Bass": notes { octave: + 2 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls[0].octave_offset, 2);
}

#[test]
fn octave_offset_negative_parses() {
    let content = r#"parts ["Bass": notes { octave: - 1 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls[0].octave_offset, -1);
}

#[test]
fn octave_offset_absent_defaults_to_zero() {
    let content = r#"parts ["Bass": notes]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls[0].octave_offset, 0);
}

#[test]
fn octave_offset_with_soundfont_before() {
    let content = r#"parts ["G": notes { sound 5, octave: - 2 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls[0].octave_offset, -2);
}

#[test]
fn octave_offset_with_volume() {
    let content = r#"parts ["B": notes { volume: 75 %, octave: - 1 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls[0].octave_offset, -1);
    assert_eq!(decls[0].volume, 75);
}

#[test]
fn octave_offset_follow_part() {
    let content = r#"parts ["A": notes, "B": follow [A] { octave: - 1 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[1].octave_offset, -1);
}

#[test]
fn follow_inherits_soundfont_volume_and_octave_from_target() {
    use crate::ast::parsed::Soundfont;
    let content = r#"parts ["A": notes { sound 48, volume: 60 %, octave: + 1 }, "B": follow [A]]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[1].soundfont, Soundfont(48));
    assert_eq!(decls[1].volume, 60);
    assert_eq!(decls[1].octave_offset, 1);
}

#[test]
fn follow_overrides_soundfont_while_inheriting_volume_and_octave() {
    use crate::ast::parsed::Soundfont;
    let content = r#"parts ["A": notes { sound 48, volume: 60 %, octave: + 1 }, "B": follow [A] { sound 1 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[1].soundfont, Soundfont(1));
    assert_eq!(decls[1].volume, 60);
    assert_eq!(decls[1].octave_offset, 1);
}

#[test]
fn follow_overrides_volume_while_inheriting_soundfont_and_octave() {
    use crate::ast::parsed::Soundfont;
    let content = r#"parts ["A": notes { sound 48, volume: 60 %, octave: + 1 }, "B": follow [A] { volume: 80 % }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[1].soundfont, Soundfont(48));
    assert_eq!(decls[1].volume, 80);
    assert_eq!(decls[1].octave_offset, 1);
}

#[test]
fn follow_overrides_octave_while_inheriting_soundfont_and_volume() {
    use crate::ast::parsed::Soundfont;
    let content = r#"parts ["A": notes { sound 48, volume: 60 %, octave: + 1 }, "B": follow [A] { octave: - 2 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[1].soundfont, Soundfont(48));
    assert_eq!(decls[1].volume, 60);
    assert_eq!(decls[1].octave_offset, -2);
}

#[test]
fn follow_chain_inherits_resolved_properties_from_intermediate_target() {
    use crate::ast::parsed::Soundfont;
    let content = r#"parts ["A": notes { sound 48, volume: 60 %, octave: + 1 }, "B": follow [A], "C": follow [B]]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[2].soundfont, Soundfont(48));
    assert_eq!(decls[2].volume, 60);
    assert_eq!(decls[2].octave_offset, 1);
}

#[test]
fn soundfont_name_may_contain_equals() {
    use crate::ast::parsed::Soundfont;
    let content = r#"parts ["Bass": notes { sound 1 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].soundfont, Soundfont(1));
}

#[test]
fn source_level_follow_soundfont_is_explicit_only() {
    use super::SourcePartMode;
    use crate::parser::parts_parser::InstrumentInfo;

    let instruments = [InstrumentInfo {
        name: "Violin".to_owned(),
        program: 40,
        category: String::new(),
        source: String::new(),
        role: String::new(),
        articulation: String::new(),
    }];

    let content = r#"parts ["Chords" [C]: follow [M] { sound 40 }]
"#;
    let mut errors = Vec::new();
    let decls = crate::parser::anyfix::source_part_declarations(content, &mut errors, &instruments);
    assert!(errors.is_empty());
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].mode, SourcePartMode::Follow);
    assert_eq!(decls[0].follow_target.as_deref(), Some("M"));
    assert_eq!(decls[0].soundfont.map(|sf| sf.0), Some(40));
}

#[test]
fn source_level_follow_without_soundfont_does_not_inherit() {
    use super::SourcePartMode;

    let content = r#"parts ["Melody" [M]: notes { sound 48 }, "Chords" [C]: follow [M]]
"#;
    let mut errors = Vec::new();
    let decls = crate::parser::anyfix::source_part_declarations(content, &mut errors, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls.len(), 2);
    assert_eq!(decls[1].mode, SourcePartMode::Follow);
    assert!(decls[1].soundfont.is_none());
}
