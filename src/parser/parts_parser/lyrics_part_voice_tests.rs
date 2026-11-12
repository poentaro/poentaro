use crate::ast::parsed::{PartDecl, SingingVoice};
use crate::error::{RecoverableError, RecoverableErrorKind};

fn parse_parts(source: &str) -> (Vec<PartDecl>, Vec<RecoverableError>) {
    let read = crate::parser::read_document(source, &[]);
    (read.declarations, read.parts_parse_errors)
}

fn lyric_part(decls: &[PartDecl]) -> &PartDecl {
    decls
        .iter()
        .find(|decl| decl.abbreviation == "l")
        .expect("lyric part `l` is declared")
}

#[test]
fn a_lyric_part_takes_a_voice_and_a_volume() {
    let (decls, errors) = parse_parts(
        "parts [\"A\" : notes, \"l\" : lyrics [A] { voice : female, volume : 50% }],\n",
    );
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let lyrics = lyric_part(&decls);
    assert_eq!(lyrics.singing_voice, Some(SingingVoice::Female));
    assert_eq!(lyrics.volume, 50);
}

#[test]
fn a_lyric_part_without_a_voice_stays_silent() {
    let (decls, errors) = parse_parts("parts [\"A\" : notes, \"l\" : lyrics [A]],\n");
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(lyric_part(&decls).singing_voice, None);
}

#[test]
fn an_unknown_voice_is_an_error_naming_the_voices() {
    let (_decls, errors) =
        parse_parts("parts [\"A\" : notes, \"l\" : lyrics [A] { voice : tenor }],\n");
    assert_eq!(errors.len(), 1, "errors: {errors:?}");
    let message = errors[0].message();
    assert!(
        message.contains("tenor") && message.contains("male, female, child"),
        "{message}"
    );
}

#[test]
fn a_lyric_part_rejects_instrument_settings() {
    let (_decls, errors) =
        parse_parts("parts [\"A\" : notes, \"l\" : lyrics [A] { octave : + 1 }],\n");
    assert_eq!(errors.len(), 1, "errors: {errors:?}");
}

#[test]
fn only_a_lyric_part_takes_a_voice() {
    let (_decls, errors) = parse_parts("parts [\"A\" : notes { voice : female }],\n");
    assert_eq!(errors.len(), 1, "errors: {errors:?}");
    assert!(errors[0].message().contains("only a lyric part sings"));
}

#[test]
fn a_voice_on_chord_lyrics_is_ignored_with_a_warning() {
    let (decls, errors) =
        parse_parts("parts [\"C\" : chords, \"l\" : lyrics [C] { voice : male }],\n");
    assert_eq!(lyric_part(&decls).singing_voice, None);
    assert_eq!(errors.len(), 1, "errors: {errors:?}");
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::PartsVoiceOnChordLyrics
    ));
}
