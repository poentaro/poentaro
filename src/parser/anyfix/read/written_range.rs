//! Reads a part's `range : C3 to G4` option.

use anyx::{FlatWord, Node};

use crate::ast::parsed::{midi_note_name, parse_midi_note_name, PitchRange};
use crate::error::RecoverableError;
use crate::parser::anyfix::decoded::span;

/// The range `value` writes (`C3 to G4`, `Bb2 to "F#4"`), or `None` with a
/// diagnostic when it isn't one.
pub(super) fn written_range(
    value: &Node,
    errors: &mut Vec<RecoverableError>,
) -> Option<PitchRange> {
    let malformed = || RecoverableError::parts_malformed_range(span(value.span));
    // Keywords like `C3` split anyfix's word runs, so `C3 to G4` is nested
    // calls; read flat, it is its words in order. A quoted word (`"F#3"`,
    // since `#` starts a comment) is one word too.
    let Ok(words) = value.flat_words() else {
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
    let pitch = |word: &FlatWord, errors: &mut Vec<RecoverableError>| {
        let midi = parse_midi_note_name(&word.text);
        if midi.is_none() {
            errors.push(RecoverableError::parts_malformed_range_pitch(
                span(word.span),
                &word.text,
            ));
        }
        midi
    };
    let (lowest, highest) = (pitch(low, errors), pitch(high, errors));
    let (lowest, highest) = (lowest?, highest?);
    if lowest > highest {
        errors.push(RecoverableError::parts_range_low_above_high(
            span(value.span),
            &midi_note_name(lowest),
            &midi_note_name(highest),
        ));
        return None;
    }
    Some(PitchRange { lowest, highest })
}
