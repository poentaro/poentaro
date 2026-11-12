//! Reads `parts [ … ]`.

use std::collections::HashSet;

use anyfix_expr::{Node, NodeKind};
use itertools::Itertools;

use crate::ast::parsed::{
    PartDecl, PartKind, PitchRange, RangeInstrument, SingingVoice, Soundfont,
};
use crate::error::{RecoverableError, Span, Spanned};
use crate::parser::anyfix::tree::{
    abbreviation_text, as_bare_words, as_field, as_part, invalid, list_of, name_span, percentage,
    record_of, signed_number, span_of, unconverted_error, value_text, whole_number, PartNode,
};
use crate::parser::parts_parser::{
    checked_octave_offset, checked_range_instrument, checked_volume, checked_written_range,
    resolve_declarations, validate_soundfont, InstrumentInfo, LyricTarget, RawDecl, RawKind,
};
use crate::sound_label::sound_label;

use super::written_range::written_range;

pub(super) fn read_parts(
    body: Option<&Node>,
    doc_span: Span,
    instruments: &[InstrumentInfo],
) -> (Vec<PartDecl>, Vec<RecoverableError>) {
    let mut errors = Vec::new();
    let raw = body.map_or_else(Vec::new, |body| raw_parts(body, instruments, &mut errors));
    let declarations = resolve_declarations(raw, &mut errors);
    // A part that failed to read has already explained why none are left.
    if declarations.is_empty() && errors.is_empty() {
        let section_span = body.map_or(doc_span, span_of);
        errors.push(RecoverableError::parts_empty_section(section_span));
    }
    (declarations, errors)
}

/// Every well-formed part in `parts [ … ]`, each abbreviation once.
pub(super) fn raw_parts(
    body: &Node,
    instruments: &[InstrumentInfo],
    errors: &mut Vec<RecoverableError>,
) -> Vec<RawDecl> {
    let elements = match list_of(body) {
        Ok(elements) => elements,
        Err(error) => {
            errors.push(error);
            return Vec::new();
        }
    };
    let mut seen = HashSet::new();
    let mut parts = Vec::new();
    for element in elements {
        match part(element, instruments, errors) {
            Ok(raw) if seen.insert(raw.abbreviation.clone()) => parts.push(raw),
            Ok(raw) => errors.push(RecoverableError::parts_duplicate_abbreviation(
                raw.span,
                &raw.abbreviation,
            )),
            Err(error) => errors.push(error),
        }
    }
    parts
}

/// `"Name" [Abbr] : kind` or `"Name" : kind`.
fn part(
    node: &Node,
    instruments: &[InstrumentInfo],
    errors: &mut Vec<RecoverableError>,
) -> Result<RawDecl, RecoverableError> {
    if let Some(error) = unconverted_error(node) {
        return Err(error);
    }
    let Some(PartNode {
        display,
        abbreviation,
        kind,
    }) = as_part(node)
    else {
        return Err(invalid(
            node,
            "expected a part, like `\"Soprano\" [S] : notes`",
        ));
    };
    let NodeKind::String(display_name) = &display.kind else {
        return Err(invalid(display, "a part's display name must be a string"));
    };
    let display_name = display_name.value.clone();
    let span = span_of(node);
    let (abbreviation, abbreviation_span) = match abbreviation {
        Some(abbreviation) => {
            if display_name.is_empty() {
                return Err(RecoverableError::parts_empty_display_name(span));
            }
            let target = bracketed_abbreviation(abbreviation)?;
            if target.text.is_empty() {
                return Err(RecoverableError::parts_empty_abbreviation(span));
            }
            (target.text, target.span)
        }
        None => {
            if display_name.is_empty() {
                return Err(RecoverableError::parts_empty_track_name(span));
            }
            (display_name.clone(), name_span(display))
        }
    };
    let (kind, options) = part_kind(kind, instruments, errors)?;
    Ok(RawDecl {
        display_name,
        abbreviation,
        abbreviation_span,
        span,
        range_instrument: checked_range_instrument(options.range_instrument, &kind, errors),
        written_range: checked_written_range(options.written_range, &kind, errors),
        kind,
        soundfont: options.soundfont,
        volume: options.volume,
        octave_offset: options.octave_offset,
        singing_voice: options.singing_voice,
    })
}

struct Abbreviation {
    text: String,
    span: Span,
}

/// The one abbreviation inside `[ ]`.
fn bracketed_abbreviation(node: &Node) -> Result<Abbreviation, RecoverableError> {
    match &node.kind {
        NodeKind::Brackets(elements) => match elements.as_slice() {
            [element] => Ok(Abbreviation {
                text: abbreviation_text(element)?,
                span: name_span(element),
            }),
            _ => Err(invalid(
                node,
                "expected one part abbreviation in brackets, like `[S1]`",
            )),
        },
        _ => Err(invalid(
            node,
            "expected one part abbreviation in brackets, like `[S1]`",
        )),
    }
}

#[derive(Default)]
struct PartOptions {
    range_instrument: Option<Spanned<RangeInstrument>>,
    written_range: Option<Spanned<PitchRange>>,
    soundfont: Option<Soundfont>,
    volume: Option<u8>,
    octave_offset: Option<i8>,
    singing_voice: Option<Spanned<SingingVoice>>,
}

/// Which options a part kind takes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum OptionsOwner {
    Percussion,
    Lyrics,
    /// A `notes` or `follow` part: its sound may be a Chimera voice.
    Notes,
    Other,
}

fn part_kind(
    node: &Node,
    instruments: &[InstrumentInfo],
    errors: &mut Vec<RecoverableError>,
) -> Result<(RawKind, PartOptions), RecoverableError> {
    let NodeKind::Call(call) = &node.kind else {
        return Err(invalid(
            node,
            "expected a part kind: notes, chords, percussion, follow [X] or lyrics [X]",
        ));
    };
    let name = call.name_text();
    let (kind, options) = match (name.as_str(), call.arguments.as_slice()) {
        ("notes", []) => (RawKind::Concrete(PartKind::Notes), None),
        ("chords", []) => (RawKind::Concrete(PartKind::Chords), None),
        ("percussion", []) => (RawKind::Concrete(PartKind::Percussion), None),
        ("notes _", [options]) => (RawKind::Concrete(PartKind::Notes), Some(options)),
        ("chords _", [options]) => (RawKind::Concrete(PartKind::Chords), Some(options)),
        ("percussion _", [options]) => (RawKind::Concrete(PartKind::Percussion), Some(options)),
        ("follow _", [target]) => (follow_kind(target)?, None),
        ("follow _ _", [target, options]) => (follow_kind(target)?, Some(options)),
        ("lyrics _", [target]) => (lyrics_kind(target)?, None),
        ("lyrics _ _", [target, options]) => (lyrics_kind(target)?, Some(options)),
        _ => {
            return Err(invalid(
                node,
                format!(
                    "found `{name}`; expected notes, chords, percussion, follow [X] or lyrics [X], \
                     each optionally followed by `{{ options }}`"
                ),
            ))
        }
    };
    let owner = match kind {
        RawKind::Concrete(PartKind::Percussion) => OptionsOwner::Percussion,
        RawKind::Lyrics { .. } => OptionsOwner::Lyrics,
        RawKind::Concrete(PartKind::Notes) | RawKind::Follow { .. } => OptionsOwner::Notes,
        RawKind::Concrete(_) => OptionsOwner::Other,
    };
    let options = match options {
        Some(options) => part_options(options, owner, instruments, errors)?,
        None => PartOptions::default(),
    };
    Ok((kind, options))
}

/// `lyrics [S]` or `lyrics [S, A, T, B]`.
fn lyrics_kind(targets: &Node) -> Result<RawKind, RecoverableError> {
    let expected = "expected the parts the lyrics are sung on, in brackets, like `[S]` or `[S, A]`";
    let NodeKind::Brackets(elements) = &targets.kind else {
        return Err(invalid(targets, expected));
    };
    if elements.is_empty() {
        return Err(invalid(targets, expected));
    }
    let targets = elements
        .iter()
        .map(|element| {
            Ok(LyricTarget {
                abbreviation: abbreviation_text(element)?,
                span: name_span(element),
            })
        })
        .collect::<Result<Vec<_>, RecoverableError>>()?;
    Ok(RawKind::Lyrics { targets })
}

fn follow_kind(target: &Node) -> Result<RawKind, RecoverableError> {
    let target = bracketed_abbreviation(target)?;
    Ok(RawKind::Follow {
        target: target.text,
        target_span: target.span,
    })
}

/// `chimera soprano`: `None` when `value` isn't a `chimera` sound, and `Some`
/// error when it is but names no known voice.
fn chimera_voice(value: &Node) -> Option<Result<Spanned<SingingVoice>, RecoverableError>> {
    let words = as_bare_words(value)?;
    let [SingingVoice::SOUND_WORD, voice] = words.as_slice() else {
        return None;
    };
    Some(
        SingingVoice::from_keyword(voice)
            .map(|voice| Spanned::new(voice, span_of(value)))
            .ok_or_else(|| {
                invalid(
                    value,
                    format!(
                        "unknown voice `{voice}`; expected {}",
                        SingingVoice::ALL
                            .iter()
                            .map(|voice| format!("`chimera {}`", voice.keyword()))
                            .join(", ")
                    ),
                )
            }),
    )
}

/// `{ instrument : violin, range : G3 to E6, sound : 41, volume : 80%, octave : +1 }`.
fn part_options(
    node: &Node,
    owner: OptionsOwner,
    instruments: &[InstrumentInfo],
    errors: &mut Vec<RecoverableError>,
) -> Result<PartOptions, RecoverableError> {
    let is_percussion = owner == OptionsOwner::Percussion;
    let mut options = PartOptions::default();
    for element in record_of(node)? {
        let Some(field) = as_field(element) else {
            errors.push(invalid(element, "expected an option, like `volume : 80%`"));
            continue;
        };
        let value = field.value;
        let span = span_of(value);
        if let Some(message) = misplaced_option(&field.words, owner) {
            errors.push(invalid(element, message));
            continue;
        }
        match field.words.as_slice() {
            ["instrument"] => match value_text(value) {
                Ok(name) => match RangeInstrument::from_keyword(&name) {
                    Some(instrument) => {
                        options.range_instrument = Some(Spanned::new(instrument, span))
                    }
                    None => errors.push(RecoverableError::parts_unknown_instrument(span, &name)),
                },
                Err(error) => errors.push(error),
            },
            ["range"] => {
                options.written_range =
                    written_range(value, errors).map(|range| Spanned::new(range, span));
            }
            ["sound"] if owner == OptionsOwner::Notes && chimera_voice(value).is_some() => {
                match chimera_voice(value) {
                    Some(Ok(voice)) => options.singing_voice = Some(voice),
                    Some(Err(error)) => errors.push(error),
                    None => {}
                }
            }
            ["sound"] => match whole_number(value) {
                Some(program) => {
                    options.soundfont = validate_soundfont(
                        &sound_label(program, is_percussion, instruments),
                        span,
                        errors,
                        instruments,
                        is_percussion,
                    );
                }
                None => errors.push(invalid(
                    value,
                    if owner == OptionsOwner::Notes {
                        "a sound is a General MIDI number, like `sound : 41`, or a voice, like `sound : chimera soprano`"
                    } else {
                        "a sound is a General MIDI number, like `sound : 41`"
                    },
                )),
            },
            ["volume"] => match percentage(value) {
                Some(written) => options.volume = Some(checked_volume(written, span, errors)),
                None => errors.push(invalid(
                    value,
                    "a volume is a percentage, like `volume : 80%`",
                )),
            },
            ["octave"] => match signed_number(value) {
                Some(written) => {
                    options.octave_offset = Some(checked_octave_offset(written, span, errors))
                }
                None => errors.push(invalid(
                    value,
                    "an octave is a signed number, like `octave : +1`",
                )),
            },
            words => errors.push(invalid(
                element,
                format!(
                    "unknown part option `{}`; expected instrument, range, sound, volume or octave",
                    words.join(" ")
                ),
            )),
        }
    }
    Ok(options)
}

/// Why `words` can't be an option of a part `owner`, if it can't: a lyric
/// part takes no options, since the parts it is sung on set the voice and
/// the volume.
fn misplaced_option(_words: &[&str], owner: OptionsOwner) -> Option<&'static str> {
    (owner == OptionsOwner::Lyrics).then_some(
        "a lyric part takes no options; the voice and the volume are the options of the parts it is sung on",
    )
}
