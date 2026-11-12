//! A singing part's notes in playback order, each with the syllable its
//! lyric part puts on it.

use itertools::Itertools;
use poentaro_singer::{SungNote, SungSyllable};

use crate::ast::grouped::{NoteEvent, Score};
use crate::ast::parsed::KeyChange;
use crate::error::IrrecoverableError;
use crate::midi::{measure_start_times_seconds, resolve_midi_note_with_accidental};

/// Default tempo before any `bpm=`, matching the MIDI writer.
const DEFAULT_BPM: u32 = 120;

/// The notes `singer` (a notes part with a voice) sings across `score`
/// (already expanded to playback order), with the syllables of the lyric part
/// sung on it. A tie onto the same pitch lengthens the note before it.
pub fn sung_notes(score: &Score, singer: &str) -> Result<Vec<SungNote>, IrrecoverableError> {
    // The instruments' length of each measure, from the MIDI writer.
    let instrument_lengths = measure_start_times_seconds(score)?
        .into_iter()
        .tuple_windows()
        .map(|(start, end)| end - start)
        .collect::<Vec<_>>();
    let mut key = KeyChange::default();
    let mut bpm = DEFAULT_BPM;
    let mut tied = false;
    let mut notes: Vec<SungNote> = Vec::new();

    let mut next_measure_start = 0.0;
    for (index, measure) in score.measures.iter().enumerate() {
        if let Some(measure_key) = &measure.key {
            key = measure_key.clone();
        }
        if let Some(measure_bpm) = measure.bpm {
            bpm = measure_bpm;
        }
        let measure_start = next_measure_start;
        next_measure_start += instrument_lengths.get(index).copied().unwrap_or(0.0);
        let slices = measure
            .parts
            .iter()
            .map(|row| row.slice())
            .collect::<Vec<_>>();
        let Some(singer_slice) = slices
            .iter()
            .find(|slice| slice.name.as_deref() == Some(singer))
        else {
            tied = false;
            continue;
        };
        let syllables = slices
            .iter()
            .filter_map(|slice| slice.lyrics.as_ref())
            .find(|lyrics| lyrics.sung_on.iter().any(|name| name == singer))
            .map_or(&[][..], |lyrics| lyrics.syllables.as_slice());
        let seconds_per_unit =
            60.0 / bpm as f64 / 4.0 / singer_slice.resolution_multiplier.max(1) as f64;
        let mut offset_units = 0u32;
        let mut slot = 0usize;

        for event in &singer_slice.notes.events {
            let start_seconds = measure_start + offset_units as f64 * seconds_per_unit;
            let end_seconds = start_seconds + event.duration() as f64 * seconds_per_unit;
            offset_units += event.duration();
            let NoteEvent::Note(note) = event else {
                tied = false;
                continue;
            };
            let midi = resolve_midi_note_with_accidental(
                &note.pitch,
                &note.accidental,
                note.octave + singer_slice.octave_offset,
                &key,
            );
            let continues_previous = tied && notes.last().is_some();
            match notes.last_mut() {
                Some(previous) if continues_previous && previous.midi == midi => {
                    previous.end_seconds = end_seconds;
                }
                _ => {
                    let syllable = if continues_previous {
                        SungSyllable::Held
                    } else {
                        slot += 1;
                        syllable_at(syllables, slot - 1)
                    };
                    notes.push(SungNote {
                        start_seconds,
                        end_seconds,
                        midi,
                        syllable,
                    });
                }
            }
            tied = note.tie_to_next();
        }
    }
    Ok(notes)
}

fn syllable_at(syllables: &[crate::ast::parsed::Syllable], slot: usize) -> SungSyllable {
    match syllables.get(slot).map(|syllable| syllable.text.as_str()) {
        Some("-") => SungSyllable::Held,
        Some("") | None => SungSyllable::Silent,
        Some(text) => SungSyllable::Word(text.to_string()),
    }
}

#[cfg(test)]
#[path = "sung_notes_tests.rs"]
mod tests;
