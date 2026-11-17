use crate::ast::parsed::{PartDecl, SingingVoice};
use crate::error::{RecoverableError, RecoverableErrorKind};

fn parse_parts(source: &str) -> (Vec<PartDecl>, Vec<RecoverableError>) {
    let read = crate::parser::read_document(source, &[]);
    (read.declarations, read.parts_parse_errors)
}

fn part<'a>(decls: &'a [PartDecl], abbreviation: &str) -> &'a PartDecl {
    decls
        .iter()
        .find(|decl| decl.abbreviation == abbreviation)
        .unwrap_or_else(|| panic!("part `{abbreviation}` is declared"))
}

#[test]
fn a_choral_instrument_sings_in_its_chimera_voice_by_default() {
    let (decls, errors) = parse_parts(
        "parts [\"S\" : notes { instrument : soprano }, \"B\" : notes { instrument : bass }, \"l\" : lyrics [S, B]],\n",
    );
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(part(&decls, "S").singing_voice, Some(SingingVoice::Soprano));
    assert_eq!(part(&decls, "B").singing_voice, Some(SingingVoice::Bass));
}

#[test]
fn any_part_sings_with_a_chimera_sound() {
    let (decls, errors) = parse_parts(
        "parts [\"V\" : notes { instrument : violin, sound : chimera tenor }, \"l\" : lyrics [V]],\n",
    );
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(part(&decls, "V").singing_voice, Some(SingingVoice::Tenor));
}

#[test]
fn a_general_midi_sound_on_a_choral_instrument_plays_instead_of_singing() {
    let (decls, errors) =
        parse_parts("parts [\"S\" : notes { instrument : soprano, sound : 52 }],\n");
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(part(&decls, "S").singing_voice, None);
}

#[test]
fn a_lyric_part_is_sung_on_every_part_it_lists() {
    let (decls, errors) = parse_parts(
        "parts [\"S\" : notes { instrument : soprano }, \"A\" : notes { instrument : alto }, \"l\" : lyrics [S, A]],\n",
    );
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let targets = &part(&decls, "l").lyric_targets;
    let names = targets
        .iter()
        .map(|index| decls[*index].abbreviation.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["S", "A"]);
}

#[test]
fn a_lyric_part_takes_no_options() {
    let (_decls, errors) =
        parse_parts("parts [\"A\" : notes, \"l\" : lyrics [A] { volume : 50% }],\n");
    assert_eq!(errors.len(), 1, "errors: {errors:?}");
    assert!(errors[0].message().contains("takes no options"));
}

#[test]
fn a_singing_part_without_lyrics_is_an_error() {
    let (_decls, errors) = parse_parts("parts [\"S\" : notes { instrument : soprano }],\n");
    assert_eq!(errors.len(), 1, "errors: {errors:?}");
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::PartsSingingPartWithoutLyrics { .. }
    ));
}

#[test]
fn a_singing_part_with_two_lyric_parts_is_an_error() {
    let (_decls, errors) = parse_parts(
        "parts [\"S\" : notes { instrument : soprano }, \"l\" : lyrics [S], \"m\" : lyrics [S]],\n",
    );
    assert_eq!(errors.len(), 1, "errors: {errors:?}");
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::PartsSingingPartWithSeveralLyrics { .. }
    ));
}
