//! Reads a part's `range : C3 to G4` option.

use anyfix_expr::{NamePart, Node, NodeKind};

use crate::ast::parsed::{midi_note_name, parse_midi_note_name, PitchRange};
use crate::error::{RecoverableError, Span};
use crate::parser::anyfix::tree::span_of;

/// One word of a range as written: a bare word (`C3`, `to`) or a quoted
/// one (`"F#3"`, since `#` starts a comment).
struct RangeWord {
    text: String,
    span: Span,
}

/// The range `value` writes (`C3 to G4`, `Bb2 to "F#4"`), or `None` with a
/// diagnostic when it isn't one.
pub(super) fn written_range(
    value: &Node,
    errors: &mut Vec<RecoverableError>,
) -> Option<PitchRange> {
    let malformed = || RecoverableError::parts_malformed_range(span_of(value));
    let Some(words) = range_words(value) else {
        errors.push(malformed());
        return None;
    };
    let [low, to, high] = words.as_slice() else {
        errors.push(malformed());
        return None;
    };
    if to.text != "to" {
        errors.push(malformed());
        return None;
    }
    let pitch = |word: &RangeWord, errors: &mut Vec<RecoverableError>| {
        let midi = parse_midi_note_name(&word.text);
        if midi.is_none() {
            errors.push(RecoverableError::parts_malformed_range_pitch(
                word.span, &word.text,
            ));
        }
        midi
    };
    let (lowest, highest) = (pitch(low, errors), pitch(high, errors));
    let (lowest, highest) = (lowest?, highest?);
    if lowest > highest {
        errors.push(RecoverableError::parts_range_low_above_high(
            span_of(value),
            &midi_note_name(lowest),
            &midi_note_name(highest),
        ));
        return None;
    }
    Some(PitchRange { lowest, highest })
}

/// `value`'s words in order, flattening the calls anyfix nests them in
/// (keywords like `C3` split a run, so `C3 to G4` is `C3 _ G4` around `to`).
fn range_words(value: &Node) -> Option<Vec<RangeWord>> {
    match &value.kind {
        NodeKind::String(literal) => Some(vec![RangeWord {
            text: literal.value.clone(),
            span: Span::new(literal.content.start, literal.content.end),
        }]),
        NodeKind::Call(call) => {
            let mut arguments = call.arguments.iter();
            let words = call
                .name
                .iter()
                .map(|part| match part {
                    NamePart::Word { text, span } => Some(vec![RangeWord {
                        text: text.clone(),
                        span: Span::new(span.start, span.end),
                    }]),
                    NamePart::Slot => range_words(arguments.next()?),
                })
                .collect::<Option<Vec<_>>>()?;
            Some(words.into_iter().flatten().collect())
        }
        _ => None,
    }
}
