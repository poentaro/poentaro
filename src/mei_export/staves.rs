//! Which parts get a staff, and with which clef.

use super::pitch::sounding_midi;
use crate::ast::grouped::{NoteEvent, Score};
use crate::ast::parsed::{KeyChange, PartKind};
use itertools::Itertools;
use std::collections::HashMap;

/// One staff of the output: a `notes` part, in declaration order, then the
/// hidden targets of shown lyric parts.
pub(super) struct StaffPart {
    /// The part's abbreviation, which `PartSlice::name` carries.
    pub(super) name: String,
    pub(super) label: String,
    pub(super) octave_offset: i8,
    pub(super) clef: Clef,
    /// A hidden part kept only to carry the verses of a shown lyric part
    /// that targets it: an invisible staff with invisible notes.
    pub(super) lyrics_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Clef {
    Treble,
    TrebleOctaveDown,
    Bass,
}

impl Clef {
    /// MEI `staffDef` clef attributes.
    pub(super) fn attributes(self) -> &'static str {
        match self {
            Self::Treble => r#"clef.shape="G" clef.line="2""#,
            Self::TrebleOctaveDown => {
                r#"clef.shape="G" clef.line="2" clef.dis="8" clef.dis.place="below""#
            }
            Self::Bass => r#"clef.shape="F" clef.line="4""#,
        }
    }
}

/// Which clef every staff gets: the staff view is engraved for a violin
/// (every staff in treble clef) or a cello (every staff in bass clef), while
/// the CLI's MEI export picks each part's clef from its range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClefChoice {
    Automatic,
    Violin,
    Cello,
}

/// MIDI pitch below which a part's average sits on a bass staff.
const BASS_CLEF_AVERAGE_BELOW: f32 = 53.0;

pub(super) fn collect_staves(
    score: &Score,
    display_names: &HashMap<String, String>,
    clef_choice: ClefChoice,
    omit_alternative_lines: bool,
) -> Vec<StaffPart> {
    let Some(first) = score.measures.first() else {
        return Vec::new();
    };
    let note_staves = first
        .parts
        .iter()
        .filter(|row| !omit_alternative_lines || !row.is_alternative_line())
        .map(|row| row.slice())
        .filter(|slice| matches!(slice.kind, PartKind::Notes))
        .filter_map(|slice| {
            let name = slice.name.clone()?;
            let label = display_names
                .get(&name)
                .cloned()
                .unwrap_or_else(|| name.clone());
            Some(StaffPart {
                clef: match clef_choice {
                    ClefChoice::Automatic => choose_clef(score, &name, slice.octave_offset),
                    ClefChoice::Violin => Clef::Treble,
                    ClefChoice::Cello => Clef::Bass,
                },
                label,
                octave_offset: slice.octave_offset,
                name,
                lyrics_only: false,
            })
        })
        .collect_vec();
    let lyrics_only_staves = lyric_targets(score)
        .into_iter()
        .filter(|target| note_staves.iter().all(|staff| staff.name != *target))
        .map(|name| StaffPart {
            name,
            label: String::new(),
            octave_offset: 0,
            clef: Clef::Treble,
            lyrics_only: true,
        })
        .collect_vec();
    note_staves.into_iter().chain(lyrics_only_staves).collect()
}

/// Targets of the score's lyric parts, in lyric part declaration order.
fn lyric_targets(score: &Score) -> Vec<String> {
    score
        .measures
        .iter()
        .flat_map(|measure| measure.parts.iter().map(|row| row.slice()))
        .filter_map(|slice| slice.lyrics.as_ref())
        .map(|lyrics| lyrics.target_name.clone())
        .unique()
        .collect()
}

/// Bass clef for low parts, octave-down treble for parts played an octave
/// lower than written (tenor-style), treble otherwise.
fn choose_clef(score: &Score, name: &str, octave_offset: i8) -> Clef {
    let pitches: Vec<i32> = score
        .measures
        .iter()
        .scan(KeyChange::default(), |key, measure| {
            if let Some(change) = &measure.key {
                *key = change.clone();
            }
            Some(part_pitches(measure, name, octave_offset, key))
        })
        .flatten()
        .collect();
    if pitches.is_empty() {
        return Clef::Treble;
    }
    let average = pitches.iter().sum::<i32>() as f32 / pitches.len() as f32;
    if average < BASS_CLEF_AVERAGE_BELOW {
        Clef::Bass
    } else if octave_offset < 0 {
        Clef::TrebleOctaveDown
    } else {
        Clef::Treble
    }
}

fn part_pitches(
    measure: &crate::ast::grouped::MultiPartMeasure,
    name: &str,
    octave_offset: i8,
    key: &KeyChange,
) -> Vec<i32> {
    measure
        .parts
        .iter()
        .map(|row| row.slice())
        .filter(|slice| slice.name.as_deref() == Some(name))
        .flat_map(|slice| slice.notes.events.iter())
        .filter_map(|event| match event {
            NoteEvent::Note(note) => Some(sounding_midi(
                &note.pitch,
                &note.accidental,
                i32::from(note.octave) + i32::from(octave_offset),
                key,
            )),
            NoteEvent::Rest(_) | NoteEvent::Chord(_) | NoteEvent::Percussion(_) => None,
        })
        .collect()
}
