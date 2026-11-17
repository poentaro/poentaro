use crate::ast::parsed::{Accidental, KeyChange, NoteName, NumberedPitch};

/// A pitch spelled as a letter plus a number of sharps (positive) or flats
/// (negative), ignoring octave. Spelling by letter rather than by pitch class
/// is what makes `7` in G come out as `F#` and `4` in F as `Bb`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpelledPitch {
    /// 0 = C, 1 = D, ... 6 = B.
    letter_index: u8,
    alteration: i8,
}

/// A chord tone's distance from its root, as a letter-step count (third =
/// 2, fifth = 4, ...) plus the exact number of semitones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Interval {
    pub(crate) letter_steps: u8,
    pub(crate) semitones: i8,
}

const LETTERS: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
const NATURAL_PITCH_CLASSES: [i8; 7] = [0, 2, 4, 5, 7, 9, 11];

impl SpelledPitch {
    pub(crate) fn key_tonic(key: &KeyChange) -> Self {
        let letter_index = match key.note.name {
            NoteName::C => 0,
            NoteName::D => 1,
            NoteName::E => 2,
            NoteName::F => 3,
            NoteName::G => 4,
            NoteName::A => 5,
            NoteName::B => 6,
        };
        Self {
            letter_index,
            alteration: accidental_alteration(&key.note.accidental),
        }
    }

    /// Spells scale degree `degree` (plus its written `#`/`b`) of the major
    /// scale starting on this tonic.
    pub(crate) fn scale_degree(self, degree: &NumberedPitch, accidental: &Accidental) -> Self {
        let raised = self.up(major_scale_interval(degree));
        Self {
            alteration: raised.alteration + accidental_alteration(accidental),
            ..raised
        }
    }

    pub(crate) fn up(self, interval: Interval) -> Self {
        let letter_index = (self.letter_index + interval.letter_steps) % 7;
        let target_pitch_class = self.pitch_class() + interval.semitones;
        let alteration =
            (target_pitch_class - natural_pitch_class(letter_index) + 6).rem_euclid(12) - 6;
        Self {
            letter_index,
            alteration,
        }
    }

    /// 0 = C, 1 = C#/Db, ... 11 = B.
    pub(crate) fn pitch_class(self) -> i8 {
        (natural_pitch_class(self.letter_index) + self.alteration).rem_euclid(12)
    }

    pub(crate) fn name(self) -> String {
        self.name_with_accidentals("#", "b")
    }

    /// [`Self::name`] plus the octave number of `midi` (C4 = 60), the
    /// sounding pitch this spelling names. The octave goes by letter, so
    /// MIDI 60 spelled as B# is `B#3`.
    pub(crate) fn name_with_octave(self, midi: u8) -> String {
        let octave = (i32::from(midi) - i32::from(self.alteration)).div_euclid(12) - 1;
        format!("{}{octave}", self.name())
    }

    /// Like [`Self::name`], with the engraved `♯`/`♭` signs the numbered view
    /// draws its own accidentals with (see `font_metrics::accidental_symbol`).
    pub(crate) fn engraved_name(self) -> String {
        self.name_with_accidentals("\u{266F}", "\u{266D}")
    }

    fn name_with_accidentals(self, sharp: &str, flat: &str) -> String {
        let letter = LETTERS
            .get(usize::from(self.letter_index))
            .copied()
            .unwrap_or('C');
        let accidental = if self.alteration >= 0 { sharp } else { flat };
        format!(
            "{letter}{}",
            accidental.repeat(usize::from(self.alteration.unsigned_abs()))
        )
    }
}

fn natural_pitch_class(letter_index: u8) -> i8 {
    NATURAL_PITCH_CLASSES
        .get(usize::from(letter_index))
        .copied()
        .unwrap_or(0)
}

fn accidental_alteration(accidental: &Accidental) -> i8 {
    match accidental {
        Accidental::Sharp => 1,
        Accidental::Flat => -1,
        Accidental::Natural => 0,
    }
}

fn major_scale_interval(degree: &NumberedPitch) -> Interval {
    let letter_steps = match degree {
        NumberedPitch::One => 0,
        NumberedPitch::Two => 1,
        NumberedPitch::Three => 2,
        NumberedPitch::Four => 3,
        NumberedPitch::Five => 4,
        NumberedPitch::Six => 5,
        NumberedPitch::Seven => 6,
    };
    Interval {
        letter_steps,
        semitones: natural_pitch_class(letter_steps),
    }
}
