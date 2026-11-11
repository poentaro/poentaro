//! A range of sounding pitches, and the letter-plus-octave names (`C4`,
//! `Bb3`, `F#3`) a part's `range` option and the out-of-range warning use.

/// An inclusive range of sounding pitches, as MIDI note numbers (C4 = 60).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PitchRange {
    pub lowest: u8,
    pub highest: u8,
}

const SHARP_NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// `midi`'s name with its octave (C4 = 60), spelling black keys as sharps.
pub fn midi_note_name(midi: u8) -> String {
    let name = SHARP_NOTE_NAMES
        .get(usize::from(midi % 12))
        .copied()
        .unwrap_or_default();
    let octave = i32::from(midi / 12) - 1;
    format!("{name}{octave}")
}

/// The MIDI note a pitch name spells: a letter (either case), an optional
/// `#` (sharp) or `b` (flat), and an octave (C4 = 60). `None` when `text`
/// isn't one, or names a note outside MIDI.
pub fn parse_midi_note_name(text: &str) -> Option<u8> {
    let mut chars = text.chars();
    let letter_semitone = match chars.next()?.to_ascii_uppercase() {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let rest = chars.as_str();
    let (accidental, octave) = match rest.chars().next()? {
        '#' => (1, &rest[1..]),
        'b' => (-1, &rest[1..]),
        _ => (0, rest),
    };
    if octave.is_empty() || !octave.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let octave: i32 = octave.parse().ok()?;
    let midi = (octave + 1) * 12 + letter_semitone + accidental;
    u8::try_from(midi).ok().filter(|midi| *midi <= 127)
}

/// One pitch the Edit Parts range pickers offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangePitchChoice {
    pub midi: u8,
    pub name: String,
}

/// The piano's keys, A0 to C8: the pitches the Edit Parts range pickers
/// offer, lowest first.
pub fn range_pitch_choices() -> Vec<RangePitchChoice> {
    (21..=108)
        .map(|midi| RangePitchChoice {
            midi,
            name: midi_note_name(midi),
        })
        .collect()
}

impl PitchRange {
    /// The range a part writes from the bounds the Edit Parts modal picked,
    /// a missing bound taken from `default` (the instrument's range) or else
    /// the other bound, and the bounds put in order. `None` when neither
    /// bound is picked.
    pub fn from_bounds(
        lowest: Option<u8>,
        highest: Option<u8>,
        default: Option<PitchRange>,
    ) -> Option<PitchRange> {
        let picked = lowest.or(highest)?;
        let first = lowest
            .or(default.map(|range| range.lowest))
            .unwrap_or(picked);
        let second = highest
            .or(default.map(|range| range.highest))
            .unwrap_or(picked);
        Some(PitchRange {
            lowest: first.min(second),
            highest: first.max(second),
        })
    }

    /// The range as a `range` option writes it: `C3 to G4`, a sharp quoted
    /// (`"F#3"`) since `#` starts an anyfix comment.
    pub fn source_text(self) -> String {
        let pitch = |midi| {
            let name = midi_note_name(midi);
            if name.contains('#') {
                format!("\"{name}\"")
            } else {
                name
            }
        };
        format!("{} to {}", pitch(self.lowest), pitch(self.highest))
    }
}

#[cfg(test)]
#[path = "pitch_range_tests.rs"]
mod tests;
