//! Reads `parts [ … ]`.

use std::collections::HashSet;

use anyfix_expr::{Node, NodeKind};

use crate::ast::parsed::{PartDecl, PartKind, RangeInstrument, Soundfont};
use crate::error::{RecoverableError, Span, Spanned};
use crate::parser::anyfix::tree::{
    abbreviation_text, as_field, as_part, invalid, list_of, name_span, percentage, record_of,
    signed_number, span_of, unconverted_error, value_text, whole_number, PartNode,
};
use crate::parser::parts_parser::{
    checked_octave_offset, checked_range_instrument, checked_volume, resolve_declarations,
    validate_soundfont, InstrumentInfo, RawDecl, RawKind,
};
use crate::sound_label::sound_label;

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
        kind,
        soundfont: options.soundfont,
        volume: options.volume,
        octave_offset: options.octave_offset,
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
    soundfont: Option<Soundfont>,
    volume: Option<u8>,
    octave_offset: Option<i8>,
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
        ("lyrics _", [target]) => {
            let target = bracketed_abbreviation(target)?;
            (
                RawKind::Lyrics {
                    target: target.text,
                    target_span: target.span,
                },
                None,
            )
        }
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
    let is_percussion = matches!(kind, RawKind::Concrete(PartKind::Percussion));
    let options = match options {
        Some(options) => part_options(options, is_percussion, instruments, errors)?,
        None => PartOptions::default(),
    };
    Ok((kind, options))
}

fn follow_kind(target: &Node) -> Result<RawKind, RecoverableError> {
    let target = bracketed_abbreviation(target)?;
    Ok(RawKind::Follow {
        target: target.text,
        target_span: target.span,
    })
}

/// `{ instrument : violin, sound : 41, volume : 80%, octave : +1 }`.
fn part_options(
    node: &Node,
    is_percussion: bool,
    instruments: &[InstrumentInfo],
    errors: &mut Vec<RecoverableError>,
) -> Result<PartOptions, RecoverableError> {
    let mut options = PartOptions::default();
    for element in record_of(node)? {
        let Some(field) = as_field(element) else {
            errors.push(invalid(element, "expected an option, like `volume : 80%`"));
            continue;
        };
        let value = field.value;
        let span = span_of(value);
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
                    "a sound is a General MIDI number, like `sound : 41`",
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
                    "unknown part option `{}`; expected instrument, sound, volume or octave",
                    words.join(" ")
                ),
            )),
        }
    }
    Ok(options)
}
