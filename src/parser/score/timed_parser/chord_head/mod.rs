use super::{EventAttrs, ParseHeadError, TimedUnitHead};
use crate::ast::parsed::{
    Accidental, BassDegree, Extension, NumberedPitch, ParsedChordNote, ParsedRest, ScoreEvent,
    TriadQuality,
};
use crate::error::{
    Diagnostic, RecoverableError, RecoverableErrorKind, Span, Warning, WarningKind,
};

pub struct ChordHead {
    degree: NumberedPitch,
    accidental: Accidental,
    triad: TriadQuality,
    extension: Option<Extension>,
    bass: Option<BassDegree>,
    is_rest: bool,
}

struct ParsedChordSymbolFields {
    degree: NumberedPitch,
    accidental: Accidental,
    triad: TriadQuality,
    extension: Option<Extension>,
    bass: Option<BassDegree>,
}

struct ChordSymbolParse {
    fields: ParsedChordSymbolFields,
    errors: Vec<Diagnostic>,
}

impl TimedUnitHead for ChordHead {
    fn parse_head(
        chars: &[char],
        start: usize,
        span: &Span,
    ) -> Result<(Self, usize, bool, Vec<Diagnostic>), ParseHeadError> {
        let Some(&degree_char) = chars.get(start) else {
            return Err(ParseHeadError::Recoverable(Some(Diagnostic::Error(
                RecoverableError {
                    span: *span,
                    kind: RecoverableErrorKind::ChordExpectedDegreeDigit { ch: '\0' },
                },
            ))));
        };
        if !matches!(degree_char, '0'..='7') {
            let pos = span.start + byte_offset_at_char_index_from_chars(chars, start);
            return Err(ParseHeadError::Recoverable(Some(Diagnostic::Error(
                RecoverableError {
                    span: Span::new(pos, pos + degree_char.len_utf8()),
                    kind: RecoverableErrorKind::ChordExpectedDegreeDigit { ch: degree_char },
                },
            ))));
        }

        if degree_char == '0' {
            return Ok((
                ChordHead {
                    degree: NumberedPitch::One,
                    accidental: Accidental::Natural,
                    triad: TriadQuality::Major,
                    extension: None,
                    bass: None,
                    is_rest: true,
                },
                start + 1,
                true,
                Vec::new(),
            ));
        }

        let (head_end, ChordSymbolParse { fields, errors }) =
            find_symbol_end(chars, start, span)
                .map_err(|e| ParseHeadError::Recoverable(Some(Diagnostic::Error(e))))?;
        Ok((
            ChordHead {
                degree: fields.degree,
                accidental: fields.accidental,
                triad: fields.triad,
                extension: fields.extension,
                bass: fields.bass,
                is_rest: false,
            },
            head_end,
            false,
            errors,
        ))
    }

    fn head_boundary(chars: &[char], i: usize) -> bool {
        crate::parser::score::timed_parser::repeat_atom_boundary(chars, i)
    }

    fn allows_octave_suffixes() -> bool {
        false
    }

    fn to_event(head: &Self, attrs: EventAttrs) -> ScoreEvent {
        let EventAttrs {
            duration,
            dotted,
            double_dotted,
            octave,
            group_membership,
            group_continuation,
            tuplet,
        } = attrs;
        std::hint::black_box(octave);
        if head.is_rest {
            ScoreEvent::Rest(ParsedRest {
                duration,
                dotted,
                double_dotted,
                group_membership: 0,
                group_continuation: 0,
                tuplet,
                implicit_fill: false,
            })
        } else {
            ScoreEvent::Chord(ParsedChordNote {
                degree: head.degree.clone(),
                accidental: head.accidental.clone(),
                triad: head.triad.clone(),
                extension: head.extension.clone(),
                bass: head.bass.clone(),
                duration,
                slur: group_continuation > 0,
                tie_to_next_span: None,
                group_membership,
                group_continuation,
                dotted,
                double_dotted,
                slur_group_close_at_duration: None,
                tuplet,
            })
        }
    }
}

fn find_symbol_end(
    chars: &[char],
    start: usize,
    span: &Span,
) -> Result<(usize, ChordSymbolParse), RecoverableError> {
    let max_end = chars.len().min(
        chars
            .get(start..)
            .unwrap_or_default()
            .iter()
            .position(|&c| matches!(c, '_' | '=' | '.' | '-' | '\'' | ',' | '(' | ')' | '~'))
            .map(|p| start + p)
            .unwrap_or(chars.len()),
    );

    for end in (start + 1..=max_end).rev() {
        let token: String = chars.get(start..end).unwrap_or_default().iter().collect();
        if let Some(parse) = parse_chord_symbol(&token, *span) {
            return Ok((end, parse));
        }
    }

    let token: String = chars
        .get(start..start + 1)
        .unwrap_or_default()
        .iter()
        .collect();
    let byte_pos = span.start + byte_offset_at_char_index_from_chars(chars, start);
    let token_len = token.len();
    Err(RecoverableError {
        span: Span::new(byte_pos, byte_pos + token_len),
        kind: RecoverableErrorKind::ChordInvalidToken {
            message: format!("invalid chord token '{token}'"),
        },
    })
}

fn parse_chord_symbol(token: &str, span: Span) -> Option<ChordSymbolParse> {
    let mut errors = Vec::new();
    let mut chars = token.chars();

    let degree = chars.next().and_then(char_to_pitch)?;

    let rest: String = chars.collect();
    let mut rest = rest.as_str();

    let accidental = if let Some(stripped) = rest.strip_prefix('#') {
        rest = stripped;
        Accidental::Sharp
    } else if let Some(stripped) = rest.strip_prefix('b') {
        rest = stripped;
        Accidental::Flat
    } else {
        Accidental::Natural
    };

    let (chord_part, bass_str) = match rest.find('/') {
        Some(pos) => (&rest[..pos], Some(&rest[pos + 1..])),
        None => (rest, None),
    };

    let (triad, ext_str) = if let Some(stripped) = chord_part.strip_prefix("sus2") {
        (TriadQuality::Sus2, stripped)
    } else if let Some(stripped) = chord_part.strip_prefix("sus4") {
        (TriadQuality::Sus4, stripped)
    } else if let Some(stripped) = chord_part.strip_prefix("sus") {
        (TriadQuality::Sus4, stripped)
    } else if let Some(stripped) = chord_part.strip_prefix('m') {
        (TriadQuality::Minor, stripped)
    } else if let Some(stripped) = chord_part.strip_prefix('o') {
        (TriadQuality::Diminished, stripped)
    } else if let Some(stripped) = chord_part.strip_prefix('+') {
        (TriadQuality::Augmented, stripped)
    } else {
        (TriadQuality::Major, chord_part)
    };

    let extension = if ext_str == "M7" {
        Some(Extension::MajorSeventh)
    } else if ext_str == "7" {
        Some(Extension::DominantSeventh)
    } else if ext_str.is_empty() {
        None
    } else {
        errors.push(Diagnostic::Warning(Warning {
            span,
            message: format!("unknown chord suffix '{ext_str}' in token '{token}'"),
            kind: WarningKind::ChordUnknownSuffix,
        }));
        None
    };

    let bass = bass_str.and_then(|s| parse_bass(s, span, &mut errors));

    Some(ChordSymbolParse {
        fields: ParsedChordSymbolFields {
            degree,
            accidental,
            triad,
            extension,
            bass,
        },
        errors,
    })
}

fn parse_bass(s: &str, span: Span, errors: &mut Vec<Diagnostic>) -> Option<BassDegree> {
    let mut chars = s.chars();
    let degree = chars.next().and_then(char_to_pitch).or_else(|| {
        errors.push(Diagnostic::Warning(Warning {
            span,
            message: format!("invalid bass note '{s}'"),
            kind: WarningKind::ChordInvalidBass,
        }));
        None
    })?;
    let accidental = match chars.next() {
        Some('#') => Accidental::Sharp,
        Some('b') => Accidental::Flat,
        None => Accidental::Natural,
        Some(c) => {
            errors.push(Diagnostic::Warning(Warning {
                span,
                message: format!("unexpected character '{c}' in bass note '{s}'"),
                kind: WarningKind::ChordBassUnexpectedChar,
            }));
            return None;
        }
    };
    if chars.next().is_some() {
        errors.push(Diagnostic::Warning(Warning {
            span,
            message: format!("bass note '{s}' has trailing characters"),
            kind: WarningKind::ChordBassTrailingChars,
        }));
        return None;
    }
    Some(BassDegree { degree, accidental })
}

fn char_to_pitch(c: char) -> Option<NumberedPitch> {
    match c {
        '1' => Some(NumberedPitch::One),
        '2' => Some(NumberedPitch::Two),
        '3' => Some(NumberedPitch::Three),
        '4' => Some(NumberedPitch::Four),
        '5' => Some(NumberedPitch::Five),
        '6' => Some(NumberedPitch::Six),
        '7' => Some(NumberedPitch::Seven),
        _ => None,
    }
}

fn byte_offset_at_char_index_from_chars(chars: &[char], char_index: usize) -> usize {
    chars
        .get(..char_index)
        .map(|slice| slice.iter().map(|c| c.len_utf8()).sum())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests;
