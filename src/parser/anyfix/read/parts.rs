//! Reads `parts [ … ]`.

use std::collections::HashSet;

use crate::ast::parsed::{PartDecl, PartKind, PitchRange, RangeInstrument, Soundfont};
use crate::error::{RecoverableError, Span, Spanned};
use crate::parser::anyfix::decoded::{
    content_span, decode_error, invalid, span, unconverted_error, AbbreviationText, PartList,
    Signed, WordsText,
};
use crate::parser::parts_parser::{
    checked_octave_offset, checked_range_instrument, checked_volume, checked_written_range,
    resolve_declarations, validate_soundfont, InstrumentInfo, RawDecl, RawKind,
};
use crate::sound_label::sound_label;
use poentaro_grammar::{self as shape, Options, Part, PartOption};

use super::written_range::written_range;

pub(super) fn read_parts(
    parts: Option<&PartList>,
    doc_span: Span,
    instruments: &[InstrumentInfo],
) -> (Vec<PartDecl>, Vec<RecoverableError>) {
    let mut errors = Vec::new();
    let raw = parts.map_or_else(Vec::new, |parts| raw_parts(parts, instruments, &mut errors));
    let declarations = resolve_declarations(raw, &mut errors);
    // A part that failed to read has already explained why none are left.
    if declarations.is_empty() && errors.is_empty() {
        let section_span = parts.map_or(doc_span, |parts| span(parts.span));
        errors.push(RecoverableError::parts_empty_section(section_span));
    }
    (declarations, errors)
}

/// Every well-formed part in `parts [ … ]`, each abbreviation once.
pub(super) fn raw_parts(
    parts: &PartList,
    instruments: &[InstrumentInfo],
    errors: &mut Vec<RecoverableError>,
) -> Vec<RawDecl> {
    let mut seen = HashSet::new();
    let mut raw_parts = Vec::new();
    for element in &parts.value {
        let raw = element
            .as_ref()
            .map_err(decode_error)
            .and_then(|element| part(element, instruments, errors));
        match raw {
            Ok(raw) if seen.insert(raw.abbreviation.clone()) => raw_parts.push(raw),
            Ok(raw) => errors.push(RecoverableError::parts_duplicate_abbreviation(
                raw.span,
                &raw.abbreviation,
            )),
            Err(error) => errors.push(error),
        }
    }
    raw_parts
}

/// `"Name" [Abbr] : kind` or `"Name" : kind`.
fn part(
    part: &Part,
    instruments: &[InstrumentInfo],
    errors: &mut Vec<RecoverableError>,
) -> Result<RawDecl, RecoverableError> {
    let whole = span(part.span());
    let (display_name, abbreviation, abbreviation_span, kind) = match part {
        Part::Unconverted(unconverted) => return Err(unconverted_error(unconverted)),
        Part::Abbreviated {
            display,
            abbreviation,
            kind,
            ..
        } => {
            if display.value.value.is_empty() {
                return Err(RecoverableError::parts_empty_display_name(whole));
            }
            let text = abbreviation.text()?;
            if text.is_empty() {
                return Err(RecoverableError::parts_empty_abbreviation(whole));
            }
            (&display.value.value, text, abbreviation.text_span(), kind)
        }
        Part::Named { display, kind, .. } => {
            if display.value.value.is_empty() {
                return Err(RecoverableError::parts_empty_track_name(whole));
            }
            let name = &display.value.value;
            (name, name.clone(), content_span(display), kind)
        }
    };
    let (kind, options) = part_kind(kind, instruments, errors)?;
    Ok(RawDecl {
        display_name: display_name.clone(),
        abbreviation,
        abbreviation_span,
        span: whole,
        range_instrument: checked_range_instrument(options.range_instrument, &kind, errors),
        written_range: checked_written_range(options.written_range, &kind, errors),
        kind,
        soundfont: options.soundfont,
        volume: options.volume,
        octave_offset: options.octave_offset,
    })
}

#[derive(Default)]
struct PartOptions {
    range_instrument: Option<Spanned<RangeInstrument>>,
    written_range: Option<Spanned<PitchRange>>,
    soundfont: Option<Soundfont>,
    volume: Option<u8>,
    octave_offset: Option<i8>,
}

fn part_kind(
    kind: &shape::PartKind,
    instruments: &[InstrumentInfo],
    errors: &mut Vec<RecoverableError>,
) -> Result<(RawKind, PartOptions), RecoverableError> {
    let (kind, options) = match kind {
        shape::PartKind::Notes { options, .. } => (RawKind::Concrete(PartKind::Notes), options),
        shape::PartKind::Chords { options, .. } => (RawKind::Concrete(PartKind::Chords), options),
        shape::PartKind::Percussion { options, .. } => {
            (RawKind::Concrete(PartKind::Percussion), options)
        }
        shape::PartKind::Follow {
            target, options, ..
        } => (
            RawKind::Follow {
                target: target.text()?,
                target_span: target.text_span(),
            },
            options,
        ),
        shape::PartKind::Lyrics { target, .. } => {
            let kind = RawKind::Lyrics {
                target: target.text()?,
                target_span: target.text_span(),
            };
            return Ok((kind, PartOptions::default()));
        }
    };
    let is_percussion = matches!(kind, RawKind::Concrete(PartKind::Percussion));
    let options = options
        .as_ref()
        .map(|options| part_options(options, is_percussion, instruments, errors))
        .unwrap_or_default();
    Ok((kind, options))
}

/// `{ instrument : violin, range : G3 to E6, sound : 41, volume : 80%, octave : +1 }`.
fn part_options(
    options: &Options,
    is_percussion: bool,
    instruments: &[InstrumentInfo],
    errors: &mut Vec<RecoverableError>,
) -> PartOptions {
    let mut read = PartOptions::default();
    for option in &options.part_option {
        let option = match option {
            Ok(option) => option,
            Err(error) => {
                errors.push(decode_error(error));
                continue;
            }
        };
        match option {
            PartOption::Instrument { name, .. } => {
                let value_span = span(name.span);
                let name = name.text("_");
                match RangeInstrument::from_keyword(&name) {
                    Some(instrument) => {
                        read.range_instrument = Some(Spanned::new(instrument, value_span))
                    }
                    None => errors.push(RecoverableError::parts_unknown_instrument(
                        value_span, &name,
                    )),
                }
            }
            PartOption::Range { value, .. } => {
                read.written_range =
                    written_range(value, errors).map(|range| Spanned::new(range, span(value.span)));
            }
            PartOption::Sound { program, .. } => match u8::try_from(program.value) {
                Ok(number) => {
                    read.soundfont = validate_soundfont(
                        &sound_label(number, is_percussion, instruments),
                        span(program.span),
                        errors,
                        instruments,
                        is_percussion,
                    );
                }
                Err(_) => errors.push(invalid(
                    program.span,
                    "a sound is a General MIDI number, like `sound : 41`",
                )),
            },
            PartOption::Volume { percentage, .. } => {
                let written = u16::try_from(percentage.number.value).unwrap_or(u16::MAX);
                read.volume = Some(checked_volume(written, span(percentage.span), errors));
            }
            PartOption::Octave { signed_integer, .. } => {
                let value = signed_integer.value();
                let written =
                    i16::try_from(value).unwrap_or(if value < 0 { i16::MIN } else { i16::MAX });
                read.octave_offset = Some(checked_octave_offset(
                    written,
                    span(signed_integer.span()),
                    errors,
                ));
            }
        }
    }
    read
}
