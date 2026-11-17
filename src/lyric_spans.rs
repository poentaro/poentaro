use crate::ast::grouped::{LyricsSlice, NoteEvent};
use crate::chord_lyric_placement::place_chord_syllables;
use crate::compiler::{lyric_target_visible, visible_part_indices};
use crate::error::IrrecoverableError;
use crate::lyric_slots::{measure_lyric_slots, LyricCursor};
use crate::note_spans::MeasureTiming;
use itertools::Itertools;

/// Source byte range of one lyric syllable — see `ast::parsed::Syllable::span`.
/// Keyed the same way the compiled SVG's `data-part-index`/`data-note-id`
/// attributes are (see `renderer::new_renderer::render_lyric_click_target`),
/// so a click hit-test on the SVG can be mapped straight back to source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricSourceSpan {
    /// Index into `MultiPartMeasure::parts` for this syllable's lyric part,
    /// matching the compiled `part_index`/`source_part_index` used throughout
    /// the renderer.
    pub source_part_index: usize,
    /// Abbreviation of the part at `source_part_index` — see
    /// `note_spans::NoteSourceSpan::part_abbreviation`.
    pub part_abbreviation: Option<String>,
    /// Same id the syllable's underlying note (of the lyric part's target)
    /// carries in `ColumnElement::note_id` (see
    /// `compiler::types::ElementContent::Lyric`).
    pub note_id: usize,
    /// Index into `Score.measures`.
    pub measure_index: usize,
    /// Inclusive start byte of this syllable's own token in the original source.
    pub start: usize,
    /// Exclusive end byte of this syllable's own token in the original source.
    pub end: usize,
    /// Where this syllable is sung within its measure: for its note's whole
    /// tie chain in the measure, or, under a chord part, from its column to
    /// the next syllable's (or the bar line).
    pub timing: MeasureTiming,
}

/// One syllable placed in a measure, before its span is built.
struct LyricPlacement {
    syllable_index: usize,
    note_id: usize,
    timing: MeasureTiming,
}

/// Where each syllable of one lyric part's measure sits, starting from
/// `cursor`. A syllable lasts as long as its note and every note tied to it
/// in the measure; under a chord part, until the next syllable (or the bar
/// line).
fn measure_placements(lyrics: &LyricsSlice, cursor: LyricCursor) -> Vec<LyricPlacement> {
    let events = &lyrics.target_events;
    if !lyrics.targets_chords {
        return measure_lyric_slots(events, lyrics.entering_tied)
            .slots
            .iter()
            .enumerate()
            .map(|(syllable_index, slot)| LyricPlacement {
                syllable_index,
                note_id: cursor.next_note_id + slot.event_index,
                timing: MeasureTiming {
                    offset: slot.column,
                    duration: events
                        .iter()
                        .skip(slot.event_index)
                        .take_while_inclusive(
                            |event| matches!(event, NoteEvent::Note(note) if note.tie_to_next()),
                        )
                        .map(NoteEvent::duration)
                        .sum(),
                },
            })
            .collect();
    }
    let placed = place_chord_syllables(events, &lyrics.syllables);
    let columns = placed
        .iter()
        .map(|placed| placed.column)
        .sorted()
        .collect_vec();
    let measure_end = events.iter().map(NoteEvent::duration).sum::<u32>();
    placed
        .iter()
        .map(|placed| {
            let next_column = columns
                .iter()
                .copied()
                .find(|&column| column > placed.column)
                .unwrap_or(measure_end.max(placed.column + 1));
            LyricPlacement {
                syllable_index: placed.syllable_index,
                note_id: placed.note_id(cursor.next_note_id),
                timing: MeasureTiming {
                    offset: placed.column,
                    duration: next_column - placed.column,
                },
            }
        })
        .collect()
}

/// Result of [`list_lyric_spans_from_source`].
pub struct LyricSpansResult {
    /// Source byte span of every lyric syllable, in score order (measure,
    /// then lyric part, then note).
    pub spans: Vec<LyricSourceSpan>,
}

/// Return the source byte span of every lyric syllable in the compiled
/// score, one entry per syllable, with note ids matching the compiled
/// `ElementContent::Lyric::note_id` values.
///
/// `enabled_tracks` must mirror whatever the caller passed to the render
/// pipeline — see `note_spans::list_note_spans_from_source`'s doc comment
/// for why: without it, `source_part_index` here would disagree with the
/// hidden-aware SVG's compacted `data-part-index` for every part declared
/// after a hidden one.
pub fn list_lyric_spans_from_source(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
) -> Result<LyricSpansResult, IrrecoverableError> {
    let mut score = crate::compile(source, filename, &[])?;
    crate::filters::apply_track_filter(&mut score, enabled_tracks);

    let max_parts = score
        .measures
        .iter()
        .map(|m| m.parts.len())
        .max()
        .unwrap_or(0);
    // One cursor per lyric part, advanced exactly as
    // `compiler::advance_lyric_cursors` does, so the note ids here line up
    // 1-to-1 with each compiled `ElementContent::Lyric::note_id`.
    let mut cursors: Vec<LyricCursor> = vec![LyricCursor::default(); max_parts];

    let mut spans = Vec::new();
    for (measure_index, measure) in score.measures.iter().enumerate() {
        let visible = visible_part_indices(measure);
        for (part_idx, part_row) in measure.parts.iter().enumerate() {
            let Some(lyrics) = &part_row.slice().lyrics else {
                continue;
            };
            let Some(cursor) = cursors.get_mut(part_idx) else {
                continue;
            };
            let start = *cursor;
            *cursor = start.advance(
                &lyrics.target_events,
                lyric_target_visible(measure, &visible, &lyrics.target_name),
            );
            if !visible.contains(&part_idx) {
                continue;
            }
            let placements = measure_placements(lyrics, start);
            spans.extend(placements.into_iter().filter_map(|placement| {
                let syllable = lyrics.syllables.get(placement.syllable_index)?;
                Some(LyricSourceSpan {
                    source_part_index: part_idx,
                    part_abbreviation: part_row.name().cloned(),
                    note_id: placement.note_id,
                    measure_index,
                    start: syllable.span.start,
                    end: syllable.span.end,
                    timing: placement.timing,
                })
            }));
        }
    }

    Ok(LyricSpansResult { spans })
}

/// One selected `(source_part_index, note_id)` cell — see
/// `LyricSourceSpan`. Structurally identical to `note_spans::NoteCell` but
/// kept as its own type: a lyric cell and a note cell sharing the same
/// underlying note number are not interchangeable, they're just keyed by the
/// same note for convenience.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LyricCell {
    pub source_part_index: usize,
    pub note_id: usize,
}

/// One contiguous selected byte range within a single lyric part's single
/// measure, ready to become a Monaco multicursor selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricSelectionRun {
    pub source_part_index: usize,
    /// Copied from the run's `LyricSourceSpan::part_abbreviation`.
    pub part_abbreviation: Option<String>,
    pub measure_index: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// Groups a range-selected set of `(source_part_index, note_id)` cells into
/// contiguous per-`(lyric part, measure)` source byte runs. Output is sorted
/// by `(source_part_index, measure_index)`.
pub fn group_selected_lyrics_into_contiguous_runs(
    selected_cells: &[LyricCell],
    lyric_spans: &[LyricSourceSpan],
) -> Vec<LyricSelectionRun> {
    let selected: std::collections::HashSet<LyricCell> = selected_cells.iter().copied().collect();

    let mut runs_by_part_measure: std::collections::HashMap<(usize, usize), LyricSelectionRun> =
        std::collections::HashMap::new();
    for span in lyric_spans {
        let cell = LyricCell {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        };
        if !selected.contains(&cell) {
            continue;
        }

        runs_by_part_measure
            .entry((span.source_part_index, span.measure_index))
            .and_modify(|run| {
                run.start_byte = run.start_byte.min(span.start);
                run.end_byte = run.end_byte.max(span.end);
            })
            .or_insert_with(|| LyricSelectionRun {
                source_part_index: span.source_part_index,
                part_abbreviation: span.part_abbreviation.clone(),
                measure_index: span.measure_index,
                start_byte: span.start,
                end_byte: span.end,
            });
    }

    let mut runs: Vec<LyricSelectionRun> = runs_by_part_measure.into_values().collect();
    runs.sort_by_key(|run| (run.source_part_index, run.measure_index, run.start_byte));
    runs
}

#[cfg(test)]
#[path = "lyric_spans_tests.rs"]
mod tests;
