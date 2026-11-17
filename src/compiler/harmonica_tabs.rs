//! Harmonica tabs for the harmonica-tab view: each note head's pitch as the
//! hole and breath that plays it on a chromatic harmonica in C — `4` blow,
//! `(4)` draw, and an arrow (`←`) above when the slide button is pressed in,
//! which raises the note a semitone (`(6)` with `←` is F♯).
//!
//! The harmonica is always in C, whatever the score's key, and every four
//! holes repeat the same layout an octave up (blow C E G C, draw D F A B).
//! So a song in D plays its F♯ as `(6)` with the slide. A part naming
//! `harmonica16` plays a 16-hole one, whose hole 1 blow is `C3`; every other
//! part plays a 12-hole one, whose hole 1 blow is middle C (`C4`). Holes are
//! numbered from the low end, so the 16-hole's middle C is hole 5.
//!
//! Each four-hole group's last blow is the same C as the next group's first
//! blow (holes 4 and 5, 8 and 9), and the same goes for C♯ with the slide.
//! Those notes go on whichever hole the melody around them moves the least
//! to reach (see `least_movement_path`), the lower hole on a tie.

use std::collections::HashMap;

use itertools::Itertools;

use super::element_position::{apply_picks, positioned_elements, ElementPosition};
use super::least_movement_path::{least_movement_path, PathCosts};
use super::letter_names::block_keys;
use super::types::{ColumnElement, CompileResult, ElementContent, NoteHeadLabel};
use crate::ast::grouped::Score;
use crate::ast::parsed::{Accidental, KeyChange, NoteName, NumberedPitch, RangeInstrument};

/// A chromatic harmonica in C: how many holes it has and where hole 1 blow
/// sits.
#[derive(Debug, Clone, Copy)]
struct Harmonica {
    holes: u8,
    /// Hole 1 blow, as `key_semitones` counts.
    lowest_c: i32,
}

impl Harmonica {
    /// The harmonica a part naming `instrument` plays.
    fn for_instrument(instrument: Option<RangeInstrument>) -> Self {
        match instrument {
            Some(RangeInstrument::Harmonica16) => Self {
                holes: 16,
                lowest_c: 36,
            },
            _ => Self {
                holes: 12,
                lowest_c: 48,
            },
        }
    }
}

/// What a pitch with no hole on the harmonica reads as.
const UNPLAYABLE: &str = "?";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Breath {
    Blow,
    Draw,
}

/// One way to play a pitch: a hole, a breath, and whether the slide button
/// is pressed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Fingering {
    hole: u8,
    breath: Breath,
    slide: bool,
}

impl Fingering {
    /// Semitones above hole 1 blow.
    fn semitones(self) -> i32 {
        let group = i32::from((self.hole - 1) / 4);
        // The hole's reed, in semitones above its four-hole group's C:
        // blow C E G C, draw D F A B.
        let reed = match (self.breath, (self.hole - 1) % 4) {
            (Breath::Blow, 0) => 0,
            (Breath::Blow, 1) => 4,
            (Breath::Blow, 2) => 7,
            (Breath::Blow, _) => 12,
            (Breath::Draw, 0) => 2,
            (Breath::Draw, 1) => 5,
            (Breath::Draw, 2) => 9,
            (Breath::Draw, _) => 11,
        };
        12 * group + reed + i32::from(self.slide)
    }

    fn label(self) -> NoteHeadLabel {
        NoteHeadLabel {
            slide_button: self.slide,
            ..NoteHeadLabel::playable(match self.breath {
                Breath::Blow => self.hole.to_string(),
                Breath::Draw => format!("({})", self.hole),
            })
        }
    }
}

/// Every fingering on `harmonica` for `pitch` (as `key_semitones` counts),
/// lowest hole first. A pitch playable without the slide is never given one
/// with it (F is `(2)`, not `2` with the slide).
fn fingerings(harmonica: Harmonica, pitch: i32) -> Vec<Fingering> {
    let all = (1..=harmonica.holes)
        .cartesian_product([Breath::Blow, Breath::Draw])
        .cartesian_product([false, true])
        .map(|((hole, breath), slide)| Fingering {
            hole,
            breath,
            slide,
        })
        .filter(|fingering| fingering.semitones() == pitch - harmonica.lowest_c)
        .collect_vec();
    match all.iter().any(|fingering| !fingering.slide) {
        true => all
            .into_iter()
            .filter(|fingering| !fingering.slide)
            .collect(),
        false => all,
    }
}

/// A note head and every way to play it.
struct NoteFingerings {
    position: ElementPosition,
    source_part_index: usize,
    fingerings: Vec<Fingering>,
}

/// `result`, compiled from `score`, with every note head's `head_label` set
/// to its tab, each part's notes (in written order) on the holes that move
/// the least. A note the harmonica can't play reads `?` and is skipped by
/// the path, as if it weren't there. The hole already says which octave the
/// note is in, so octave dots are dropped.
pub(super) fn place_harmonica_tabs(result: CompileResult, score: &Score) -> CompileResult {
    let keys = block_keys(&result.blocks, score);
    let harmonica = |part_index: usize| {
        Harmonica::for_instrument(
            score
                .measures
                .iter()
                .find_map(|measure| measure.parts.get(part_index))
                .and_then(|part| part.slice().range_instrument),
        )
    };
    let (playable, unplayable): (Vec<NoteFingerings>, Vec<NoteFingerings>) =
        positioned_elements(&result.blocks)
            .filter_map(|positioned| match &positioned.element.content {
                ElementContent::NoteHead {
                    pitch,
                    accidental,
                    octave,
                    ..
                } => Some(NoteFingerings {
                    position: positioned.position,
                    source_part_index: positioned.source_part_index,
                    fingerings: fingerings(
                        harmonica(positioned.source_part_index),
                        key_semitones(keys.get(positioned.position.block)?)
                            + degree_semitones(pitch)
                            + accidental_semitones(accidental)
                            + 12 * i32::from(*octave),
                    ),
                }),
                _ => None,
            })
            .partition(|note| !note.fingerings.is_empty());
    let picks: HashMap<ElementPosition, Option<Fingering>> = playable
        .into_iter()
        .into_group_map_by(|note| note.source_part_index)
        .into_values()
        .flat_map(|part_notes| {
            let path = least_movement_path(
                &part_notes
                    .iter()
                    .map(|note| note.fingerings.clone())
                    .collect_vec(),
                &PathCosts {
                    node: |_| 0,
                    movement: |from: &Fingering, to: &Fingering| {
                        u32::from(from.hole.abs_diff(to.hole))
                    },
                },
            );
            part_notes
                .into_iter()
                .zip(path)
                .map(|(note, index)| (note.position, note.fingerings.get(index).copied()))
                .collect_vec()
        })
        .chain(unplayable.into_iter().map(|note| (note.position, None)))
        .collect();
    apply_picks(result, picks, with_tab)
}

fn with_tab(element: ColumnElement, fingering: Option<Fingering>) -> ColumnElement {
    match element.content {
        ElementContent::NoteHead {
            pitch,
            accidental,
            octave: _,
            dotted,
            double_dotted,
            head_label: _,
        } => ColumnElement {
            content: ElementContent::NoteHead {
                head_label: Some(match fingering {
                    Some(fingering) => fingering.label(),
                    None => NoteHeadLabel::unplayable(UNPLAYABLE.to_string()),
                }),
                pitch,
                accidental,
                octave: 0,
                dotted,
                double_dotted,
            },
            ..element
        },
        _ => element,
    }
}

/// The key's tonic in semitones, comparable across keys (`C4` is 48).
fn key_semitones(key: &KeyChange) -> i32 {
    let letter = match key.note.name {
        NoteName::C => 0,
        NoteName::D => 2,
        NoteName::E => 4,
        NoteName::F => 5,
        NoteName::G => 7,
        NoteName::A => 9,
        NoteName::B => 11,
    };
    12 * i32::from(key.note.octave) + letter + accidental_semitones(&key.note.accidental)
}

fn degree_semitones(pitch: &NumberedPitch) -> i32 {
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

fn accidental_semitones(accidental: &Accidental) -> i32 {
    match accidental {
        Accidental::Sharp => 1,
        Accidental::Flat => -1,
        Accidental::Natural => 0,
    }
}
