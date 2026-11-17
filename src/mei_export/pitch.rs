//! Spelling numbered degrees as staff pitches: letter names, key signatures and
//! when an accidental must be written.

use crate::ast::parsed::{Accidental, KeyChange, NoteName, NumberedPitch};
use std::collections::HashMap;

/// A written pitch: letter, octave, and alteration in semitones from the
/// natural letter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SpelledPitch {
    pub(super) letter: Letter,
    pub(super) octave: i32,
    pub(super) alteration: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Letter {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

const LETTERS: [Letter; 7] = [
    Letter::C,
    Letter::D,
    Letter::E,
    Letter::F,
    Letter::G,
    Letter::A,
    Letter::B,
];

/// Letters in the order sharps are added to a key signature; flats are added
/// in the reverse order.
const SHARP_ORDER: [Letter; 7] = [
    Letter::F,
    Letter::C,
    Letter::G,
    Letter::D,
    Letter::A,
    Letter::E,
    Letter::B,
];

impl Letter {
    fn from_note_name(name: &NoteName) -> Self {
        match name {
            NoteName::C => Self::C,
            NoteName::D => Self::D,
            NoteName::E => Self::E,
            NoteName::F => Self::F,
            NoteName::G => Self::G,
            NoteName::A => Self::A,
            NoteName::B => Self::B,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::C => 0,
            Self::D => 1,
            Self::E => 2,
            Self::F => 3,
            Self::G => 4,
            Self::A => 5,
            Self::B => 6,
        }
    }

    fn semitone(self) -> i32 {
        match self {
            Self::C => 0,
            Self::D => 2,
            Self::E => 4,
            Self::F => 5,
            Self::G => 7,
            Self::A => 9,
            Self::B => 11,
        }
    }

    /// MEI `@pname`.
    pub(super) fn pname(self) -> &'static str {
        match self {
            Self::C => "c",
            Self::D => "d",
            Self::E => "e",
            Self::F => "f",
            Self::G => "g",
            Self::A => "a",
            Self::B => "b",
        }
    }

    /// The letter `steps` letters above this one, and how many octaves that
    /// wrapped.
    fn step_up(self, steps: usize) -> SteppedLetter {
        let total = self.index() + steps;
        SteppedLetter {
            letter: LETTERS.get(total % 7).copied().unwrap_or(Self::C),
            octaves: (total / 7) as i32,
        }
    }
}

struct SteppedLetter {
    letter: Letter,
    octaves: i32,
}

pub(super) fn degree_steps(pitch: &NumberedPitch) -> usize {
    match pitch {
        NumberedPitch::One => 0,
        NumberedPitch::Two => 1,
        NumberedPitch::Three => 2,
        NumberedPitch::Four => 3,
        NumberedPitch::Five => 4,
        NumberedPitch::Six => 5,
        NumberedPitch::Seven => 6,
    }
}

fn major_scale_semitone(pitch: &NumberedPitch) -> i32 {
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

fn accidental_offset(accidental: &Accidental) -> i32 {
    match accidental {
        Accidental::Sharp => 1,
        Accidental::Flat => -1,
        Accidental::Natural => 0,
    }
}

fn tonic_midi(key: &KeyChange) -> i32 {
    12 * (i32::from(key.note.octave) + 1)
        + Letter::from_note_name(&key.note.name).semitone()
        + accidental_offset(&key.note.accidental)
}

/// MIDI pitch of a numbered degree, `octave` octaves (dots plus part offset)
/// from the key's tonic octave.
pub(super) fn sounding_midi(
    pitch: &NumberedPitch,
    accidental: &Accidental,
    octave: i32,
    key: &KeyChange,
) -> i32 {
    tonic_midi(key) + major_scale_semitone(pitch) + accidental_offset(accidental) + 12 * octave
}

/// Spells a scale degree in the key: the letter follows the degree, so `#4`
/// in C is F♯ and `b5` is G♭.
pub(super) fn spell(
    pitch: &NumberedPitch,
    accidental: &Accidental,
    octave: i32,
    key: &KeyChange,
) -> SpelledPitch {
    let stepped = Letter::from_note_name(&key.note.name).step_up(degree_steps(pitch));
    let letter_octave = i32::from(key.note.octave) + stepped.octaves + octave;
    let natural = 12 * (letter_octave + 1) + stepped.letter.semitone();
    SpelledPitch {
        letter: stepped.letter,
        octave: letter_octave,
        alteration: sounding_midi(pitch, accidental, octave, key) - natural,
    }
}

/// Fifths of the major key whose tonic is `key`: positive counts sharps,
/// negative counts flats.
pub(super) fn key_fifths(key: &KeyChange) -> i32 {
    let tonic =
        Letter::from_note_name(&key.note.name).semitone() + accidental_offset(&key.note.accidental);
    let fifths = match tonic.rem_euclid(12) {
        0 => 0,
        1 => -5,
        2 => 2,
        3 => -3,
        4 => 4,
        5 => -1,
        6 => 6,
        7 => 1,
        8 => -4,
        9 => 3,
        10 => -2,
        _ => 5,
    };
    // Keep the enharmonic spelling the key was written with (G♭ not F♯).
    match (fifths, &key.note.accidental) {
        (6, Accidental::Flat) => -6,
        (5, Accidental::Flat) => -7,
        (-5, Accidental::Sharp) => 7,
        _ => fifths,
    }
}

/// MEI `@key.sig`, e.g. `1f` for F major.
pub(super) fn key_signature(key: &KeyChange) -> String {
    match key_fifths(key) {
        0 => "0".to_string(),
        fifths if fifths > 0 => format!("{fifths}s"),
        fifths => format!("{}f", -fifths),
    }
}

/// The alteration the key signature gives `letter`.
fn key_alteration(fifths: i32, letter: Letter) -> i32 {
    let position = SHARP_ORDER
        .iter()
        .position(|sharp| *sharp == letter)
        .unwrap_or(0) as i32;
    if fifths > position {
        1
    } else if fifths < 0 && 7 + fifths <= position {
        -1
    } else {
        0
    }
}

/// Decides when a written accidental is needed: against the key signature,
/// remembering what's already shown earlier in the measure.
pub(super) struct AccidentalTracker {
    fifths: i32,
    shown: HashMap<StaffPosition, i32>,
}

#[derive(PartialEq, Eq, Hash)]
struct StaffPosition {
    letter: Letter,
    octave: i32,
}

impl AccidentalTracker {
    pub(super) fn new(key: &KeyChange) -> Self {
        Self {
            fifths: key_fifths(key),
            shown: HashMap::new(),
        }
    }

    /// MEI accidental attribute for `pitch`: `@accid` when it must be
    /// written, `@accid.ges` when the key or an earlier accidental implies it.
    pub(super) fn attribute(&mut self, pitch: SpelledPitch) -> String {
        let position = StaffPosition {
            letter: pitch.letter,
            octave: pitch.octave,
        };
        let current = self
            .shown
            .get(&position)
            .copied()
            .unwrap_or_else(|| key_alteration(self.fifths, pitch.letter));
        let symbol = match pitch.alteration {
            2 => "x",
            1 => "s",
            0 => "n",
            -1 => "f",
            _ => "ff",
        };
        if current != pitch.alteration {
            self.shown.insert(position, pitch.alteration);
            format!(" accid=\"{symbol}\"")
        } else if pitch.alteration == 0 {
            String::new()
        } else {
            format!(" accid.ges=\"{symbol}\"")
        }
    }
}

/// A pitch class for a chord symbol, e.g. `B♭`.
pub(super) fn pitch_class_name(pitch: SpelledPitch) -> String {
    let accidental = match pitch.alteration {
        1 => "♯",
        -1 => "♭",
        2 => "𝄪",
        -2 => "𝄫",
        _ => "",
    };
    format!("{}{accidental}", pitch.letter.pname().to_uppercase())
}
