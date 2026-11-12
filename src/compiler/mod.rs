pub mod types;
pub use types::*;

mod beam;

mod part_slice;
use part_slice::{compile_part_slice, PartSliceInput};

mod part_slice_unit;

mod timed_unit;

mod slur_chains;
use slur_chains::{PartCrossState, PendingSlurOpen};

mod tuplet_spans;

mod rest_runs;
use rest_runs::merge_rest_runs;

mod chord_degrees;
pub use chord_degrees::ChordDegrees;
#[cfg(test)]
pub(crate) mod test_chord_degrees;

mod element_position;
mod harmonica_tabs;
mod least_movement_path;
mod letter_names;
pub use letter_names::{label_note_heads, NoteLabels};

mod guitar_chords;
mod guitar_voicing_path;

use crate::ast::grouped::{MultiPartMeasure, NoteEvent, PartRow, Score};
use crate::ast::parsed::{Accidental, KeyChange, NoteName, PartKind};
use crate::lyric_slots::LyricCursor;
use itertools::Itertools;

struct PartSliceResult {
    elements: Vec<ColumnElement>,
    final_pending_opens: Vec<Option<PendingSlurOpen>>,
    final_tie: bool,
    final_tie_column: Option<u32>,
    final_tie_measure: Option<usize>,
    final_tie_note_id: Option<usize>,
    final_next_note_id: usize,
}

pub fn compile(score: &Score) -> CompileResult {
    let max_parts = score
        .measures
        .iter()
        .map(|m| m.parts.len())
        .max()
        .unwrap_or(0);
    let mut cross_states: Vec<PartCrossState> =
        (0..max_parts).map(|_| PartCrossState::new()).collect();

    let mut slur_spans: Vec<SlurSpan> = Vec::new();
    let mut tuplet_spans: Vec<TupletSpan> = Vec::new();
    let blocks: Vec<MeasureBlock> = score
        .measures
        .iter()
        .enumerate()
        .map(|(measure_index, measure)| {
            compile_measure(
                measure,
                measure_index + 1,
                measure_index,
                &mut cross_states,
                &mut slur_spans,
                &mut tuplet_spans,
            )
        })
        .collect();

    let (blocks, measure_to_block) = merge_rest_runs(&score.measures, blocks);
    for span in &mut slur_spans {
        let (Some(&from), Some(&to)) = (
            measure_to_block.get(span.from_measure),
            measure_to_block.get(span.to_measure),
        ) else {
            continue;
        };
        span.from_measure = from;
        span.to_measure = to;
    }
    for span in &mut tuplet_spans {
        let Some(&mapped) = measure_to_block.get(span.measure_index) else {
            continue;
        };
        span.measure_index = mapped;
    }

    CompileResult {
        lyric_links: lyric_links(score),
        blocks,
        slur_spans,
        tuplet_spans,
    }
}

/// Every lyric part's link to the part it is drawn under, read from the part
/// level (`LyricsSlice::drawn_under`) and named by the same `RowId`s the rows get.
fn lyric_links(score: &Score) -> Vec<LyricLink> {
    score
        .measures
        .iter()
        .flat_map(|measure| measure.parts.iter().enumerate())
        .filter_map(|(part_idx, part_row)| {
            part_row.slice().lyrics.as_ref().map(|lyrics| LyricLink {
                lyric_row_id: part_row_id(part_row, part_idx),
                target_row_id: RowId(lyrics.drawn_under.clone()),
            })
        })
        .unique()
        .collect()
}

fn part_row_id(part_row: &PartRow, part_idx: usize) -> RowId {
    RowId(
        part_row
            .name()
            .cloned()
            .unwrap_or_else(|| format!("__anon_{part_idx}")),
    )
}

/// Indices into `measure.parts` that are actually compiled/sounded for this
/// measure — i.e. `measure.parts` minus whichever resting parts
/// `hide_resting_parts` hides when at least one other part has real content.
/// Shared with `midi::timing::note_timings_seconds`, which must walk exactly
/// these same parts in the same order for its `note_id` counters to line up
/// with `ColumnElement::note_id` (see `compile_measure`, which uses this too).
pub(crate) fn visible_part_indices(measure: &MultiPartMeasure) -> Vec<usize> {
    let visible_part_count =
        if measure.hide_resting_parts && measure.parts.iter().any(|p| !is_resting(p)) {
            measure.parts.iter().filter(|p| !is_resting(p)).count()
        } else {
            measure.parts.len()
        };
    measure
        .parts
        .iter()
        .enumerate()
        .filter_map(|(part_idx, part_row)| {
            if visible_part_count < measure.parts.len() && is_resting(part_row) {
                None
            } else {
                Some(part_idx)
            }
        })
        .collect()
}

/// Determines if a part row is "resting" — either all notes are rests, or it's a
/// lyric part with no syllables. A resting part is hidden when `hide_resting_parts`
/// is on and at least one other part has real content.
pub(crate) fn is_resting(part_row: &PartRow) -> bool {
    let slice = part_row.slice();

    // A lyric part with no syllables is resting (nothing to sing)
    if let Some(lyrics) = &slice.lyrics {
        return lyrics.syllables.is_empty();
    }

    // A non-lyric part is resting if all its notes are rests
    let events = slice.timing_events();
    !events.is_empty() && events.iter().all(|e| matches!(e, NoteEvent::Rest(_)))
}

/// Whether a lyric part's note ids advance past this measure: when the part
/// it sings along to is compiled here (see `visible_part_indices`), so the
/// ids match that part's. A target hidden by the track filter is gone from
/// `measure.parts` altogether, with no note ids to match; the ids still
/// advance then, or every measure's syllables would repeat the same ids.
pub(crate) fn lyric_target_visible(
    measure: &MultiPartMeasure,
    visible: &[usize],
    target_name: &str,
) -> bool {
    let target_index = measure
        .parts
        .iter()
        .position(|part_row| part_row.name().map(String::as_str) == Some(target_name));
    target_index.is_none_or(|part_idx| visible.contains(&part_idx))
}

/// Moves every lyric part's cursor past this measure, including a lyric part
/// whose row is hidden here (see `lyric_slots::LyricCursor`).
fn advance_lyric_cursors(
    measure: &MultiPartMeasure,
    visible: &[usize],
    cross_states: &mut [PartCrossState],
) {
    for (part_row, cs) in measure.parts.iter().zip(cross_states.iter_mut()) {
        let Some(lyrics) = &part_row.slice().lyrics else {
            continue;
        };
        let cursor = LyricCursor {
            next_note_id: cs.next_note_id,
        }
        .advance(
            &lyrics.target_events,
            lyric_target_visible(measure, visible, &lyrics.target_name),
        );
        cs.next_note_id = cursor.next_note_id;
    }
}

fn update_cross_state(cs: &mut PartCrossState, result: &mut PartSliceResult) {
    cs.pending_slur_opens = std::mem::take(&mut result.final_pending_opens);
    cs.prev_tie = result.final_tie;
    cs.prev_tie_column = result.final_tie_column;
    cs.prev_tie_measure = result.final_tie_measure;
    cs.prev_tie_note_id = result.final_tie_note_id;
    cs.next_note_id = result.final_next_note_id;
}

fn compile_measure(
    measure: &MultiPartMeasure,
    bar_number: usize,
    measure_index: usize,
    cross_states: &mut Vec<PartCrossState>,
    slur_spans: &mut Vec<SlurSpan>,
    tuplet_spans: &mut Vec<TupletSpan>,
) -> MeasureBlock {
    while cross_states.len() < measure.parts.len() {
        cross_states.push(PartCrossState::new());
    }

    let visible = visible_part_indices(measure);

    let decorations = collect_decorations(measure, bar_number);
    let mut rows: Vec<MeasureRow> = Vec::new();
    for (part_idx, part_row) in measure.parts.iter().enumerate() {
        if !visible.contains(&part_idx) {
            continue;
        }
        let Some(cs) = cross_states.get(part_idx) else {
            continue;
        };
        // Drop any incoming cross-measure tie/slur arc when this slice has errors (#28).
        // A lyric part draws no arcs, and its ties only decide which notes take a
        // syllable, which the grouper already settled from the notes alone.
        let drops_incoming_arcs =
            part_row.slice().has_error && !matches!(part_row.slice().kind, PartKind::Lyrics { .. });
        let (init_pending_opens, init_tie, init_tie_column, init_tie_measure, init_tie_note_id) =
            if drops_incoming_arcs {
                (vec![], false, None, None, None)
            } else {
                (
                    cs.clone_pending_opens(),
                    cs.prev_tie,
                    cs.prev_tie_column,
                    cs.prev_tie_measure,
                    cs.prev_tie_note_id,
                )
            };
        let init_next_note_id = cs.next_note_id;

        let mut slice_result = compile_part_slice(
            part_row.slice(),
            PartSliceInput {
                pending_opens: init_pending_opens,
                prev_tie: init_tie,
                prev_tie_column: init_tie_column,
                prev_tie_measure: init_tie_measure,
                prev_tie_note_id: init_tie_note_id,
                next_note_id: init_next_note_id,
                measure_index,
                part_index: part_idx,
            },
            slur_spans,
            tuplet_spans,
        );

        // A lyric part's cursor advances after the loop, hidden or not.
        if part_row.slice().lyrics.is_none() {
            let Some(cs) = cross_states.get_mut(part_idx) else {
                continue;
            };
            update_cross_state(cs, &mut slice_result);
        }

        let label = part_row.name().cloned().unwrap_or_default();
        let id = part_row_id(part_row, part_idx);
        let kind = part_row.slice().kind;
        rows.push(MeasureRow {
            id,
            label,
            kind,
            elements: slice_result.elements,
            source_part_index: part_idx,
            absorbed_rows: Vec::new(),
        });
    }

    advance_lyric_cursors(measure, &visible, cross_states);

    MeasureBlock {
        rows,
        decorations,
        diagnostics: measure.diagnostics.clone(),
        represents_measures: 1,
        merge_duplicate_measures_across_parts: measure.merge_duplicate_measures_across_parts,
        system_break: measure.system_break,
        source_span: measure.source_span,
    }
}

fn format_key(key: &KeyChange) -> String {
    let name = match key.note.name {
        NoteName::A => "A",
        NoteName::B => "B",
        NoteName::C => "C",
        NoteName::D => "D",
        NoteName::E => "E",
        NoteName::F => "F",
        NoteName::G => "G",
    };
    let accidental = match key.note.accidental {
        Accidental::Natural => "",
        Accidental::Sharp => "\u{266f}",
        Accidental::Flat => "\u{266d}",
    };
    format!("1={name}{accidental}")
}

fn collect_decorations(measure: &MultiPartMeasure, bar_number: usize) -> Vec<Decoration> {
    vec![Decoration::DirectiveLine {
        label: measure.label.clone(),
        bar_number: Some(bar_number as u32),
        key: measure.key.as_ref().map(format_key),
        bpm: measure.bpm,
        time_signature: measure
            .time_signature
            .as_ref()
            .map(|ts| (ts.numerator as u32, ts.denominator as u32)),
    }]
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_all_rest_notes_measures;
#[cfg(test)]
mod tests_blank_lyric_rows;
#[cfg(test)]
mod tests_directive_mid_score;
#[cfg(test)]
mod tests_harmonica_tabs;
#[cfg(test)]
mod tests_implicit_fill_rest;
#[cfg(test)]
mod tests_letter_names;
#[cfg(test)]
mod tests_lyric_cursor_properties;
#[cfg(test)]
mod tests_lyrics_and_diagnostics;
#[cfg(test)]
mod tests_multi_measure_rest;
#[cfg(test)]
mod tests_row_part_kind;
#[cfg(test)]
mod tests_slur;
#[cfg(test)]
mod tests_source_span;
#[cfg(test)]
mod tests_system_break;
#[cfg(test)]
mod tests_tuplets;
