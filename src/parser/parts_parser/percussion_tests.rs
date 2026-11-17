use crate::ast::parsed::{PartKind, Soundfont};

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
fn parses_percussion_track() {
    let content = r#"parts ["Snare": percussion { sound 38 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].abbreviation, "Snare");
    assert_eq!(decls[0].kind, PartKind::Percussion);
    assert_eq!(decls[0].soundfont, Soundfont(38));
}

#[test]
fn percussion_soundfont_number_skips_instrument_catalog_validation() {
    use super::InstrumentInfo;

    // The catalog only contains melodic instruments; a percussion key number
    // (38 = Acoustic Snare) is not among them and must not be rejected or
    // fuzzy-suggested against.
    let instruments = [InstrumentInfo {
        name: "Choir Aahs".to_owned(),
        program: 52,
        category: String::new(),
        source: String::new(),
        role: String::new(),
        articulation: String::new(),
    }];

    let content = r#"parts ["Snare": percussion { sound 38 }]
"#;
    let (decls, errors) = parse_parts(content, &instruments);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[0].soundfont, Soundfont(38));
}

#[test]
fn percussion_without_soundfont_uses_default() {
    let content = r#"parts ["Snare": percussion]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[0].kind, PartKind::Percussion);
    assert_eq!(decls[0].soundfont, Soundfont::ACOUSTIC_SNARE);
}

#[test]
fn multiple_percussion_parts_parse_independently() {
    let content = r#"parts ["Snare": percussion { sound 38 }, "Kick": percussion { sound 36 }]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls.len(), 2);
    assert_eq!(decls[0].soundfont, Soundfont(38));
    assert_eq!(decls[1].soundfont, Soundfont(36));
}
