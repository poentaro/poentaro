//! A numbered note's sounding pitch as a MIDI note number (C4 = 60), shared by
//! MIDI export and the instrument range check.

use crate::ast::parsed::{Accidental, KeyChange, NoteName, NumberedPitch};

pub(crate) fn note_name_to_semitone(name: &NoteName) -> i32 {
    match name {
        NoteName::C => 0,
        NoteName::D => 2,
        NoteName::E => 4,
        NoteName::F => 5,
        NoteName::G => 7,
        NoteName::A => 9,
        NoteName::B => 11,
    }
}

pub(crate) fn pitch_to_scale_offset(pitch: &NumberedPitch) -> i32 {
    match pitch {
        NumberedPitch::One => 0,
        NumberedPitch::Two => 2,
        NumberedPitch::Three => 4,
        NumberedPitch::Four => 5,
        NumberedPitch::Five => 7,
        NumberedPitch::Six => 9,
        NumberedPitch::Seven => 11,
    }
}

pub(crate) fn accidental_offset(acc: &Accidental) -> i32 {
    match acc {
        Accidental::Sharp => 1,
        Accidental::Flat => -1,
        Accidental::Natural => 0,
    }
}

/// `pitch` (with its written `accidental`) shifted `octave` octaves, in
/// `key`, as a MIDI note number clamped to 0..=127.
pub(crate) fn resolve_midi_note_with_accidental(
    pitch: &NumberedPitch,
    accidental: &Accidental,
    octave: i8,
    key: &KeyChange,
) -> u8 {
    let root = 12 * (key.note.octave as i32 + 1)
        + note_name_to_semitone(&key.note.name)
        + accidental_offset(&key.note.accidental);
    let midi =
        root + pitch_to_scale_offset(pitch) + accidental_offset(accidental) + (octave as i32) * 12;
    midi.clamp(0, 127) as u8
}
