//! A lyric part's notes in playback order, each with the syllable it sings.

use itertools::Itertools;
use poentaro_singer::{SungNote, SungSyllable};

use crate::ast::grouped::{MultiPartMeasure, NoteEvent, PartSlice, Score};
use crate::ast::parsed::KeyChange;
use crate::error::IrrecoverableError;
use crate::midi::{measure_start_times_seconds, resolve_midi_note_with_accidental};

/// Default tempo before any `bpm=`, matching the MIDI writer.
const DEFAULT_BPM: u32 = 120;

/// The notes `lyric_part` sings across `score` (already expanded to playback
/// order). A tie onto the same pitch lengthens the note before it.
pub fn sung_notes(score: &Score, lyric_part: &str) -> Result<Vec<SungNote>, IrrecoverableError> {
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
        // The lyric target's own length counts too, for when its notes are
        // filtered out of the audio.
        let measure_start = next_measure_start;
        let instrument_seconds = instrument_lengths.get(index).copied().unwrap_or(0.0);
        next_measure_start +=
            instrument_seconds.max(lyric_measure_seconds(measure, lyric_part, bpm));
        let slices = measure
            .parts
            .iter()
            .map(|row| row.slice())
            .collect::<Vec<_>>();
        let Some((lyric_slice, lyrics)) = slices
            .iter()
            .filter(|slice| slice.name.as_deref() == Some(lyric_part))
            .find_map(|slice| slice.lyrics.as_ref().map(|lyrics| (*slice, lyrics)))
            // Lyrics under a chords part have no notes to sing.
            .filter(|(_, lyrics)| !lyrics.targets_chords)
        else {
            tied = false;
            continue;
        };
        let octave_offset = target_octave_offset(&slices, &lyrics.target_name);
        let seconds_per_unit =
            60.0 / bpm as f64 / 4.0 / lyric_slice.resolution_multiplier.max(1) as f64;
        let mut offset_units = 0u32;
        let mut slot = 0usize;

        for event in &lyrics.target_events {
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
                note.octave + octave_offset,
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
                        syllable_at(&lyrics.syllables, slot - 1)
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

fn lyric_measure_seconds(measure: &MultiPartMeasure, lyric_part: &str, bpm: u32) -> f64 {
    measure
        .parts
        .iter()
        .map(|row| row.slice())
        .filter(|slice| slice.name.as_deref() == Some(lyric_part))
        .find_map(|slice| {
            let lyrics = slice.lyrics.as_ref()?;
            let units: u32 = lyrics.target_events.iter().map(NoteEvent::duration).sum();
            Some(units as f64 * 60.0 / bpm as f64 / 4.0 / slice.resolution_multiplier.max(1) as f64)
        })
        .unwrap_or(0.0)
}

fn target_octave_offset(slices: &[&PartSlice], target_name: &str) -> i8 {
    slices
        .iter()
        .find(|slice| slice.name.as_deref() == Some(target_name))
        .map_or(0, |slice| slice.octave_offset)
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
