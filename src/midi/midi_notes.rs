use crate::ast::parsed::{KeyChange, NumberedPitch};
pub(crate) use crate::sounding_pitch::{
    accidental_offset, note_name_to_semitone, pitch_to_scale_offset,
    resolve_midi_note_with_accidental,
};

const TPQ: u16 = 480; // ticks per quarter note

pub(crate) fn resolve_midi_note(pitch: &NumberedPitch, octave: i8, key: &KeyChange) -> u8 {
    let root = 12 * (key.note.octave as i32 + 1)
        + note_name_to_semitone(&key.note.name)
        + accidental_offset(&key.note.accidental);
    let midi = root + pitch_to_scale_offset(pitch) + (octave as i32) * 12;
    midi.clamp(0, 127) as u8
}

/// Converts a duration in quarter-beats (sixteenth-note units) to MIDI ticks.
///
/// `quarter_beats` is read directly off a `GroupedNote`/`GroupedRest`/etc., which for a
/// tuplet-rescaled measure has already been multiplied by that measure's
/// `resolution_multiplier` (see `GroupedMeasure::resolution_multiplier`) — `multiplier`
/// divides that scaling back out so a tuplet note converts to the same tick count as the
/// equivalent plain note. `multiplier` is `1` for every non-tuplet measure, making this a
/// no-op division.
///
/// Known limitation: `TPQ` (480) only divides evenly by multipliers built from small prime
/// factors (2, 3, 5); a multiplier that pulls in a larger prime (e.g. `7` for a septuplet)
/// doesn't divide `quarter_beats * TPQ` evenly, so the result rounds down to the nearest
/// tick. This is accepted rounding error, not bumped away by raising `TPQ` — see the
/// **Tuplet** glossary entry in `ARCHITECTURE.md`.
pub(crate) fn duration_to_ticks(quarter_beats: u32, multiplier: u32) -> u32 {
    quarter_beats * (TPQ as u32) / (4 * multiplier)
}
