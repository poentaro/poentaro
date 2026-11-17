use super::PART_SETTING_LIMITS;
use crate::error::{RecoverableErrorKind, Span};

fn parse_parts(
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
fn volume_at_either_bound_is_accepted() {
    let content = r#"parts ["A": notes { volume: 0 % }, "B": notes { volume: 100 % }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[0].volume, PART_SETTING_LIMITS.volume.min);
    assert_eq!(decls[1].volume, PART_SETTING_LIMITS.volume.max);
}

#[test]
fn volume_above_max_clamps_with_diagnostic_on_the_volume_token() {
    let content = r#"parts ["B": notes { volume: 150 % }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert_eq!(decls[0].volume, 100);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(
        errors[0].kind,
        RecoverableErrorKind::PartsVolumeOutOfRange { volume: 150 }
    );
    assert_eq!(errors[0].span, Span::new(28, 33));
    assert_eq!(
        errors[0].kind.message(),
        "volume 150% is out of range; valid range is 0% to 100%; clamped to 100%"
    );
}

#[test]
fn volume_too_wide_for_a_byte_still_clamps() {
    let content = r#"parts ["B": notes { volume: 999 % }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert_eq!(decls[0].volume, 100);
    assert_eq!(
        errors[0].kind,
        RecoverableErrorKind::PartsVolumeOutOfRange { volume: 999 }
    );
}

#[test]
fn octave_offset_at_either_bound_is_accepted() {
    let content = r#"parts ["A": notes { octave: - 4 }, "B": notes { octave: + 4 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(
        decls[0].octave_offset,
        PART_SETTING_LIMITS.octave_offset.min
    );
    assert_eq!(
        decls[1].octave_offset,
        PART_SETTING_LIMITS.octave_offset.max
    );
}

#[test]
fn octave_offset_above_max_clamps_with_diagnostic() {
    let content = r#"parts ["B": notes { octave: +5 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert_eq!(decls[0].octave_offset, 4);
    assert_eq!(
        errors[0].kind,
        RecoverableErrorKind::PartsOctaveOffsetOutOfRange { offset: 5 }
    );
    assert_eq!(errors[0].span, Span::new(28, 30));
    assert_eq!(
        errors[0].kind.message(),
        "octave offset +5 is out of range; valid range is -4 to +4; clamped to +4"
    );
}

#[test]
fn octave_offset_below_min_clamps_with_diagnostic() {
    let content = r#"parts ["B": notes { octave: -200 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert_eq!(decls[0].octave_offset, -4);
    assert_eq!(
        errors[0].kind.message(),
        "octave offset -200 is out of range; valid range is -4 to +4; clamped to -4"
    );
}

#[test]
fn follow_part_inherits_its_targets_clamped_volume() {
    let content = r#"parts ["A": notes { volume: 150 % }, "B": follow [A]]
"#;
    let (decls, _) = parse_parts(content, &[]);
    assert_eq!(decls[1].volume, 100);
}
