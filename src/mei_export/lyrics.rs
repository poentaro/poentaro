//! Lyric parts as MEI verses on their target part's notes.

use super::escape;
use crate::ast::grouped::{LyricsSlice, Score};
use crate::ast::parsed::{PartKind, Syllable};
use itertools::Itertools;
use std::collections::HashMap;

/// One lyric part's syllables, per measure.
pub(super) struct Verse {
    measures: Vec<MeasureSyllables>,
}

/// One measure of a verse: syllables keyed by the target event they sit on.
#[derive(Clone, Default)]
pub(super) struct MeasureSyllables {
    by_event_index: HashMap<usize, Syllable>,
}

/// Verses per target part abbreviation, in lyric part declaration order.
pub(super) fn collect_verses(score: &Score) -> HashMap<String, Vec<Verse>> {
    let lyric_part_names = score
        .measures
        .first()
        .map(|measure| {
            measure
                .parts
                .iter()
                .map(|row| row.slice())
                .filter(|slice| matches!(slice.kind, PartKind::Lyrics { .. }))
                .filter_map(|slice| slice.name.clone())
                .collect_vec()
        })
        .unwrap_or_default();
    lyric_part_names
        .iter()
        .flat_map(|lyric_name| verses_of(score, lyric_name))
        .into_group_map_by(|targeted| targeted.target.clone())
        .into_iter()
        .map(|(target, verses)| {
            let verses = verses.into_iter().map(|targeted| targeted.verse).collect();
            (target, verses)
        })
        .collect()
}

struct TargetedVerse {
    target: String,
    verse: Verse,
}

/// A lyric part's verse on each part it pairs with (see
/// `lyric_paired_target`): each holds the measures paired with that part.
fn verses_of(score: &Score, lyric_name: &str) -> Vec<TargetedVerse> {
    let slices = score
        .measures
        .iter()
        .map(|measure| {
            measure
                .parts
                .iter()
                .map(|row| row.slice())
                .find(|slice| slice.name.as_deref() == Some(lyric_name))
                .and_then(|slice| slice.lyrics.as_ref())
        })
        .collect_vec();
    slices
        .iter()
        .flatten()
        .map(|lyrics| lyrics.target_name.clone())
        .unique()
        .map(|target| {
            let measures = slices
                .iter()
                .map(|lyrics| match lyrics {
                    Some(lyrics) if lyrics.target_name == target => measure_syllables_of(lyrics),
                    _ => MeasureSyllables::default(),
                })
                .collect();
            TargetedVerse {
                target,
                verse: Verse { measures },
            }
        })
        .collect()
}

fn measure_syllables_of(lyrics: &LyricsSlice) -> MeasureSyllables {
    let slots =
        crate::lyric_slots::measure_lyric_slots(&lyrics.target_events, lyrics.entering_tied);
    let by_event_index = slots
        .slots
        .iter()
        .zip(lyrics.syllables.iter())
        .map(|(slot, syllable)| (slot.event_index, syllable.clone()))
        .collect();
    MeasureSyllables { by_event_index }
}

/// Each verse's syllables for one measure.
pub(super) fn measure_syllables(verses: &[Verse], measure_index: usize) -> Vec<MeasureSyllables> {
    verses
        .iter()
        .map(|verse| {
            verse
                .measures
                .get(measure_index)
                .cloned()
                .unwrap_or_default()
        })
        .collect()
}

/// `<verse>` elements for the note at `event_index`.
pub(super) fn verses_xml(verses: &[MeasureSyllables], event_index: usize) -> String {
    verses
        .iter()
        .enumerate()
        .filter_map(|(verse_index, verse)| {
            let text = verse.by_event_index.get(&event_index)?.text.trim();
            (!text.is_empty() && text != "-" && text != "_").then(|| {
                format!(
                    "<verse n=\"{}\"><syl>{}</syl></verse>",
                    verse_index + 1,
                    escape(text)
                )
            })
        })
        .join("")
}
