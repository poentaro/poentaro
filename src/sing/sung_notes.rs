//! A singing part's notes in playback order, each with the syllable its
//! lyric part puts on it.

use itertools::Itertools;
use poentaro_singer::{SungNote, SungSyllable};

use crate::ast::grouped::{NoteEvent, PartSlice, Score};
use crate::ast::parsed::{KeyChange, Syllable};
use crate::error::IrrecoverableError;
use crate::lyric_slots::{measure_lyric_slots, MeasureTime};
use crate::midi::{measure_start_times_seconds, resolve_midi_note_with_accidental};

/// Default tempo before any `bpm=`, matching the MIDI writer.
const DEFAULT_BPM: u32 = 120;

/// The notes `singer` (a notes part with a voice) sings across `score`
/// (already expanded to playback order), with the syllables of the lyric part
/// sung on it, each syllable on the note that starts when the part the
/// syllables are paired with takes it. A tie onto the same pitch lengthens
/// the note before it; a tie onto another pitch, or a slur onto a note no
/// syllable starts on, holds the syllable before.
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
    let mut slurred = false;
    // The paired part's last note was tied into the next measure.
    let mut paired_tied = false;
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
            slurred = false;
            continue;
        };
        let lyrics = MeasureLyrics::sung_on(&slices, singer_slice, &mut paired_tied);
        let seconds_per_unit =
            60.0 / bpm as f64 / 4.0 / singer_slice.resolution_multiplier.max(1) as f64;
        let mut offset_units = 0u32;

        for event in &singer_slice.notes.events {
            let column = offset_units;
            let start_seconds = measure_start + offset_units as f64 * seconds_per_unit;
            let end_seconds = start_seconds + event.duration() as f64 * seconds_per_unit;
            offset_units += event.duration();
            let NoteEvent::Note(note) = event else {
                tied = false;
                slurred = false;
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
                        lyrics.syllable_at(column, slurred && notes.last().is_some())
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
            slurred = note.slur;
        }
    }
    Ok(notes)
}

/// The syllables a singer sings in one measure, and when.
struct MeasureLyrics<'a> {
    syllables: &'a [Syllable],
    /// When each syllable is sung: at the lyric slots of the part the
    /// syllables are paired with, which may be another part of the choir.
    times: Vec<MeasureTime>,
    /// The singer's measure length, in its own columns.
    singer_length: u32,
}

impl<'a> MeasureLyrics<'a> {
    /// The lyrics among `slices` sung on `singer`. `paired_tied` carries the
    /// paired part's tie into the next measure.
    fn sung_on(slices: &[&'a PartSlice], singer: &PartSlice, paired_tied: &mut bool) -> Self {
        let singer_length = singer.notes.events.iter().map(NoteEvent::duration).sum();
        let Some(lyrics) = slices
            .iter()
            .filter_map(|slice| slice.lyrics.as_ref())
            .find(|lyrics| {
                lyrics
                    .sung_on
                    .iter()
                    .any(|name| Some(name) == singer.name.as_ref())
            })
        else {
            return MeasureLyrics {
                syllables: &[],
                times: Vec::new(),
                singer_length,
            };
        };
        let slots = measure_lyric_slots(&lyrics.target_events, *paired_tied);
        *paired_tied = slots.exits_tied;
        MeasureLyrics {
            syllables: &lyrics.syllables,
            times: slots
                .slots
                .iter()
                .map(|slot| MeasureTime {
                    column: slot.column,
                    length: slots.end_column,
                })
                .collect(),
            singer_length,
        }
    }

    /// The syllable the singer's note starting at `column` sings. A note no
    /// syllable starts on holds the one before when it's `slurred_on` to.
    fn syllable_at(&self, column: u32, slurred_on: bool) -> SungSyllable {
        let time = MeasureTime {
            column,
            length: self.singer_length,
        };
        let index = self
            .times
            .iter()
            .position(|syllable| syllable.same_as(time));
        match index.and_then(|index| self.syllables.get(index)) {
            Some(syllable) if syllable.text == "-" => SungSyllable::Held,
            Some(syllable) if !syllable.text.is_empty() => {
                SungSyllable::Word(syllable.text.clone())
            }
            Some(_) => SungSyllable::Silent,
            None if index.is_none() && slurred_on => SungSyllable::Held,
            None => SungSyllable::Silent,
        }
    }
}

#[cfg(test)]
#[path = "sung_notes_tests.rs"]
mod tests;
