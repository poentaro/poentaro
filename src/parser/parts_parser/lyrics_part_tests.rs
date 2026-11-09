use crate::ast::parsed::PartKind;
use crate::error::RecoverableErrorKind;

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

fn abbreviations(decls: &[crate::ast::parsed::PartDecl]) -> Vec<&str> {
    decls
        .iter()
        .map(|decl| decl.abbreviation.as_str())
        .collect()
}

#[test]
fn lyrics_parts_are_parts_of_their_own_linked_to_their_target() {
    let content = r#"parts ["bass" [b]: notes, "Verse 1" [v1]: lyrics [b], "Verse 2" [v2]: lyrics [b]]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(abbreviations(&decls), ["b", "v1", "v2"]);
    assert_eq!(decls[1].display_name, "Verse 1");
    assert_eq!(
        decls[1].kind,
        PartKind::Lyrics {
            target_part_index: 0
        }
    );
    assert_eq!(
        decls[2].kind,
        PartKind::Lyrics {
            target_part_index: 0
        }
    );
}

#[test]
fn lyrics_parts_sit_directly_after_their_target_whatever_the_declaration_order() {
    let content = r#"parts ["A": notes, "B": notes, "lb": lyrics [B], "la": lyrics [A], "lb2": lyrics [B]]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(abbreviations(&decls), ["A", "la", "B", "lb", "lb2"]);
    assert_eq!(
        decls[1].kind,
        PartKind::Lyrics {
            target_part_index: 0
        }
    );
    assert_eq!(
        decls[3].kind,
        PartKind::Lyrics {
            target_part_index: 2
        }
    );
    assert_eq!(
        decls[4].kind,
        PartKind::Lyrics {
            target_part_index: 2
        }
    );
}

#[test]
fn lyrics_part_may_target_a_follow_part_or_chords() {
    let content = r#"parts ["A": notes, "B": follow [A], "C": chords, "lb": lyrics [B], "lc": lyrics [C]]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(abbreviations(&decls), ["A", "B", "lb", "C", "lc"]);
    assert_eq!(
        decls[2].kind,
        PartKind::Lyrics {
            target_part_index: 1
        }
    );
    assert_eq!(
        decls[4].kind,
        PartKind::Lyrics {
            target_part_index: 3
        }
    );
}

#[test]
fn lyrics_with_unknown_target_emits_error_at_the_target() {
    let content = r#"parts ["A": notes, "l": lyrics [Z]]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert_eq!(abbreviations(&decls), ["A"]);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::PartsLyricsInvalidTarget { ref target } if target == "Z"
    ));
    let target_start = content.find('Z').expect("Z in source");
    assert_eq!(errors[0].span.start, target_start);
}

#[test]
fn lyrics_targeting_percussion_or_a_later_part_is_an_error() {
    let content = r#"parts ["l1": lyrics [P], "P": percussion, "l2": lyrics [P], "A": notes]
"#;
    let (_decls, errors) = parse_parts(content, &[]);
    assert_eq!(errors.len(), 2);
}

#[test]
fn lyrics_targeting_another_lyrics_part_is_an_error() {
    let content = r#"parts ["A": notes, "l1": lyrics [A], "l2": lyrics [l1]]
"#;
    let (decls, errors) = parse_parts(content, &[]);
    assert_eq!(abbreviations(&decls), ["A", "l1"]);
    assert_eq!(errors.len(), 1);
}

#[test]
fn lyrics_part_rejects_settings() {
    let content = "parts [\"A\" : notes, \"l\" : lyrics [A] { volume : 50% }],\n";
    let (decls, errors) = parse_parts(content, &[]);
    assert_eq!(abbreviations(&decls), ["A"]);
    assert_eq!(errors.len(), 1);
}

#[test]
fn lyrics_abbreviation_cannot_duplicate_a_part_abbreviation() {
    let content = "parts [\"A\" : notes, \"A\" : lyrics [A]],\n";
    let (decls, errors) = parse_parts(content, &[]);
    assert_eq!(abbreviations(&decls), ["A"]);
    assert_eq!(errors.len(), 1);
}
