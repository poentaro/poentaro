use std::collections::HashMap;

use itertools::Itertools;

use crate::ast::grouped::{GroupedNote, NoteEvent, Score};
use crate::ast::parsed::{midi_note_name, CheckedRange, KeyChange, PartDecl, PitchRange};
use crate::error::{Diagnostic, Span, Warning};
use crate::pitch_description::spelling::SpelledPitch;
use crate::sounding_pitch::resolve_midi_note_with_accidental;

struct OutOfRangeNote {
    measure_index: usize,
    span: Span,
    part_abbreviation: String,
    warning: Warning,
}

/// Warns on each note whose sounding pitch (key plus the part's octave
/// offset) falls outside its part's [`CheckedRange`], attaching the
/// warning to the note's measure. A note shared by several measures or parts
/// (ditto, follow) is warned on once per part.
pub(super) fn validate_ranges(score: &mut Score, declarations: &[PartDecl]) {
    let ranges: HashMap<&str, CheckedRange> = declarations
        .iter()
        .filter_map(|declaration| {
            declaration
                .checked_range()
                .map(|range| (declaration.abbreviation.as_str(), range))
        })
        .collect();
    if ranges.is_empty() {
        return;
    }

    let keys = score
        .measures
        .iter()
        .scan(KeyChange::default(), |key, measure| {
            if let Some(changed) = &measure.key {
                *key = changed.clone();
            }
            Some(key.clone())
        })
        .collect::<Vec<_>>();

    let out_of_range_notes = score
        .measures
        .iter()
        .zip(&keys)
        .enumerate()
        .flat_map(|(measure_index, (measure, key))| {
            let ranges = &ranges;
            measure.parts.iter().flat_map(move |part_row| {
                let slice = part_row.slice();
                let abbreviation = slice.name.as_deref().unwrap_or_default();
                let range = ranges.get(abbreviation).copied();
                slice.notes.events.iter().filter_map(move |event| {
                    let NoteEvent::Note(note) = event else {
                        return None;
                    };
                    let problem = out_of_range_problem(note, slice.octave_offset, range?, key)?;
                    Some(OutOfRangeNote {
                        measure_index,
                        span: note.event_span,
                        part_abbreviation: abbreviation.to_string(),
                        warning: Warning::note_out_of_range(
                            note.event_span,
                            format!("{abbreviation}: {problem}"),
                        ),
                    })
                })
            })
        })
        .unique_by(|note| (note.span, note.part_abbreviation.clone()))
        .collect::<Vec<_>>();

    for note in out_of_range_notes {
        if let Some(measure) = score.measures.get_mut(note.measure_index) {
            measure.diagnostics.push(Diagnostic::Warning(note.warning));
        }
    }
}

/// Why `note` (shifted by `octave_offset` octaves, in `key`) falls outside
/// `range`, or `None` when it is in range.
fn out_of_range_problem(
    note: &GroupedNote,
    octave_offset: i8,
    range: CheckedRange,
    key: &KeyChange,
) -> Option<String> {
    let octave = note.octave.saturating_add(octave_offset);
    let midi = resolve_midi_note_with_accidental(&note.pitch, &note.accidental, octave, key);
    let PitchRange { lowest, highest } = range.pitches();
    let side = if midi < lowest {
        format!(
            "below {} lowest note {}",
            range.owner(),
            midi_note_name(lowest)
        )
    } else if midi > highest {
        format!(
            "above {} highest note {}",
            range.owner(),
            midi_note_name(highest)
        )
    } else {
        return None;
    };
    let sounding = SpelledPitch::key_tonic(key)
        .scale_degree(&note.pitch, &note.accidental)
        .name_with_octave(midi);
    Some(format!("note sounds as {sounding}, {side}"))
}

#[cfg(test)]
#[path = "range_validation_tests.rs"]
mod tests;
