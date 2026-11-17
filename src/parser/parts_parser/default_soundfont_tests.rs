use crate::ast::parsed::Soundfont;

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
fn chords_and_notes_parts_default_to_piano() {
    let content = r#"parts ["C": chords, "M": notes]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[0].soundfont, Soundfont::ACOUSTIC_GRAND_PIANO);
    assert_eq!(decls[1].soundfont, Soundfont::ACOUSTIC_GRAND_PIANO);
}

#[test]
fn follow_part_inherits_the_piano_default() {
    let content = r#"parts ["S": notes, "A": follow [S]]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[1].soundfont, Soundfont::ACOUSTIC_GRAND_PIANO);
}

#[test]
fn explicit_soundfont_overrides_the_default() {
    let content = r#"parts ["S": notes { sound 52 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[0].soundfont, Soundfont::CHOIR_AAHS);
}

#[test]
fn unreadable_soundfont_falls_back_to_the_kind_default() {
    let content = r#"parts ["S": notes { sound : "not a program" }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(!errors.is_empty());
    assert_eq!(decls[0].soundfont, Soundfont::ACOUSTIC_GRAND_PIANO);
}
