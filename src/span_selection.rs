//! Turns the source text a selection covers into the notes and lyrics it
//! selects, against whichever parts are currently visible.
//!
//! A selection's byte ranges stay put when a part is hidden or shown, but the
//! listed spans do not: hidden parts drop out and `source_part_index` is
//! renumbered. Deriving the selected cells from the byte ranges again, against
//! the freshly listed spans, keeps the highlight and "play selection" in step
//! with what is visible.

use crate::lyric_spans::{
    group_selected_lyrics_into_contiguous_runs, LyricCell, LyricSelectionRun, LyricSourceSpan,
};
use crate::note_spans::{
    group_selected_notes_into_contiguous_runs, NoteCell, NoteSelectionRun, NoteSourceSpan,
};
use crate::source_edit::ByteRange;

/// The notes a set of byte ranges selects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteSelection {
    /// Every note, chord or rest whose token overlaps a range, in span order.
    pub cells: Vec<NoteCell>,
    /// `cells` grouped per `(part, measure)`, see
    /// [`group_selected_notes_into_contiguous_runs`].
    pub runs: Vec<NoteSelectionRun>,
}

/// The lyric syllables a set of byte ranges selects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricSelection {
    /// Every syllable whose token overlaps a range, in span order.
    pub cells: Vec<LyricCell>,
    /// `cells` grouped per `(part, measure)`, see
    /// [`group_selected_lyrics_into_contiguous_runs`].
    pub runs: Vec<LyricSelectionRun>,
}

fn overlaps_any(ranges: &[ByteRange], start: usize, end: usize) -> bool {
    ranges.iter().any(|range| range.overlaps_bytes(start, end))
}

/// Selects every note whose token overlaps one of `ranges`. Empty ranges
/// (a bare caret) select nothing.
pub fn select_notes_in_byte_ranges(
    ranges: &[ByteRange],
    note_spans: &[NoteSourceSpan],
) -> NoteSelection {
    let cells: Vec<NoteCell> = note_spans
        .iter()
        .filter(|span| match (span.start, span.end) {
            (Some(start), Some(end)) => overlaps_any(ranges, start, end),
            _ => false,
        })
        .map(|span| NoteCell {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        })
        .collect();
    let runs = group_selected_notes_into_contiguous_runs(&cells, note_spans);
    NoteSelection { cells, runs }
}

/// Selects every lyric syllable whose token overlaps one of `ranges`. Empty
/// ranges (a bare caret) select nothing.
pub fn select_lyrics_in_byte_ranges(
    ranges: &[ByteRange],
    lyric_spans: &[LyricSourceSpan],
) -> LyricSelection {
    let cells: Vec<LyricCell> = lyric_spans
        .iter()
        .filter(|span| overlaps_any(ranges, span.start, span.end))
        .map(|span| LyricCell {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        })
        .collect();
    let runs = group_selected_lyrics_into_contiguous_runs(&cells, lyric_spans);
    LyricSelection { cells, runs }
}

#[cfg(test)]
#[path = "span_selection_tests.rs"]
mod tests;
