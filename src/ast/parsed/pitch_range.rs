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

#[cfg(test)]
#[path = "pitch_range_tests.rs"]
mod tests;
