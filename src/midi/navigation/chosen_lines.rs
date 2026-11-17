//! A `sequence` entry's `WITH ([w] 2)`: in that occurrence, the part plays
//! its alternative line (`w.2`) in its own place. Alternative rows never
//! play themselves (see `PartRow::is_alternative_line`).

use crate::ast::grouped::{
    ChosenLine, MultiPartMeasure, NoteEvent, PartRow, PartSlice, SequenceSpan,
};

/// One measure of the playback order: which written measure it replays, and
/// the `sequence` entry it plays under.
pub(super) struct PlaybackMeasure<'a> {
    pub written_index: usize,
    pub span: &'a SequenceSpan,
}

pub(super) fn playback_measures(spans: &[SequenceSpan]) -> Vec<PlaybackMeasure<'_>> {
    spans
        .iter()
        .flat_map(|span| {
            (span.start..=span.end).map(move |written_index| PlaybackMeasure {
                written_index,
                span,
            })
        })
        .collect()
}

/// Puts each chosen line's content in its part's row, where this measure
/// writes that line; elsewhere the part plays its first line. The part's
/// entry in `part_written_indices` then points at the line played, so
/// playback highlights it.
pub(super) fn play_chosen_lines(
    measure: &mut MultiPartMeasure,
    chosen_lines: &[ChosenLine],
    part_written_indices: &mut [usize],
) {
    for chosen in chosen_lines {
        let Some(alternative) = chosen.alternative_abbreviation() else {
            continue;
        };
        let position = |name: &str| {
            measure
                .parts
                .iter()
                .position(|row| row.name().is_some_and(|row_name| row_name == name))
        };
        let (Some(base_index), Some(alternative_index)) =
            (position(&chosen.part), position(&alternative))
        else {
            continue;
        };
        let Some(line) = measure
            .parts
            .get(alternative_index)
            .map(|row| row.slice().clone())
        else {
            continue;
        };
        if !is_written(&line) {
            continue;
        }
        let Some(base) = measure.parts.get_mut(base_index).map(PartRow::slice_mut) else {
            continue;
        };
        base.notes = line.notes;
        base.lyrics = line.lyrics;
        base.has_error = line.has_error;
        if let Some(written_index) = part_written_indices.get(alternative_index).copied() {
            if let Some(entry) = part_written_indices.get_mut(base_index) {
                *entry = written_index;
            }
        }
    }
}

/// Whether the measure writes this line, rather than filling it in.
fn is_written(line: &PartSlice) -> bool {
    match &line.lyrics {
        Some(lyrics) => !lyrics.syllables.is_empty(),
        None => line
            .notes
            .events
            .iter()
            .any(|event| !matches!(event, NoteEvent::Rest(rest) if rest.implicit_fill)),
    }
}
