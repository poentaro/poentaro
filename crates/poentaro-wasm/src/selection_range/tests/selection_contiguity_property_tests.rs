//! Property test: whatever two notes or lyric syllables a click-and-click
//! gesture lands on, the resolved selection is contiguous.
//!
//! Scores are generated as real `.poentaro` source (several `notes` parts in
//! 4/4 with mixed durations, rests and ties, some with a lyric part of CJK
//! and Latin syllables) and run through the same `list_note_spans_response`/
//! `list_lyric_spans_response` the web app calls, so the spans carry every
//! quirk the real pipeline gives them (tie continuations reusing their
//! note's id, rests as spans, lyric parts reordered under their target,
//! parts with different rhythms in one measure).
//!
//! The expected selection is every note and syllable, in every row from the
//! upper endpoint's down to the lower one's, that sounds during the window
//! from the earlier endpoint's onset to the later one's end. Holes are also
//! checked on their own, since they're what a user sees:
//! - **horizontally**: an unselected cell with a selected one before it and
//!   after it in its own row, or
//! - **vertically**: an unselected cell with a selected one in a row above
//!   it and in a row below it, all three sounding at some shared instant
//!   inside the window. (A selected note can ring on past the window's end,
//!   so an instant after it doesn't count.)

mod generated_score;
mod generated_source;

use itertools::Itertools;
use proptest::prelude::*;
use proptest::sample::Index;

use crate::responses::{list_lyric_spans_response, list_note_spans_response};
use crate::selection_range::resolve_selection_range_response;
use crate::selection_range::types::{ClickableElementId, ResolveSelectionRangeResponse};
use crate::types::{ListLyricSpansResponse, ListNoteSpansResponse};

use generated_score::{score_strategy, Timing};
use generated_source::to_source;

use super::test_helpers::{lyric, note};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CellKind {
    Note,
    Lyric,
}

/// One note or syllable as a selection names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Cell {
    kind: CellKind,
    /// The rendered row: parts are numbered in row order, since the parser
    /// moves every lyric part under its target.
    part: usize,
    note_id: usize,
}

impl Cell {
    fn clickable_element_id(&self) -> ClickableElementId {
        match self.kind {
            CellKind::Note => note(self.part, self.note_id),
            CellKind::Lyric => lyric(self.part, self.note_id),
        }
    }
}

/// One span placed on the score's timeline. A tied note is one cell made of
/// several spans.
#[derive(Debug, Clone, Copy)]
struct TimedSpan {
    cell: Cell,
    timing: Timing,
}

/// The selection a resolved response names.
fn selected_cells(response: &ResolveSelectionRangeResponse) -> Option<Vec<Cell>> {
    let ResolveSelectionRangeResponse::Ok {
        note_cells,
        lyric_cells,
    } = response
    else {
        return None;
    };
    Some(
        note_cells
            .iter()
            .map(|cell| Cell {
                kind: CellKind::Note,
                part: cell.source_part_index,
                note_id: cell.note_id,
            })
            .chain(lyric_cells.iter().map(|cell| Cell {
                kind: CellKind::Lyric,
                part: cell.source_part_index,
                note_id: cell.note_id,
            }))
            .collect(),
    )
}

/// The window from the earlier endpoint's onset to the later one's end,
/// counting every span of a tied endpoint.
fn endpoint_window(timed: &[TimedSpan], endpoints: &[Cell]) -> Timing {
    let endpoint_spans = timed
        .iter()
        .filter(|span| endpoints.contains(&span.cell))
        .collect_vec();
    Timing {
        onset: endpoint_spans
            .iter()
            .map(|span| span.timing.onset)
            .min()
            .unwrap_or(0),
        end: endpoint_spans
            .iter()
            .map(|span| span.timing.end)
            .max()
            .unwrap_or(0),
    }
}

/// Every cell in the endpoints' rows (and the rows between them) that
/// sounds during their window.
fn expected_cells(timed: &[TimedSpan], anchor: Cell, current: Cell, window: Timing) -> Vec<Cell> {
    let top = anchor.part.min(current.part);
    let bottom = anchor.part.max(current.part);
    timed
        .iter()
        .filter(|span| (top..=bottom).contains(&span.cell.part) && span.timing.overlaps(&window))
        .map(|span| span.cell)
        .unique()
        .collect()
}

/// Unselected cells with a selected cell before and after them in their own
/// row.
fn horizontal_holes(timed: &[TimedSpan], selected: &[Cell]) -> Vec<TimedSpan> {
    timed
        .iter()
        .into_group_map_by(|span| (span.cell.kind, span.cell.part))
        .into_values()
        .flat_map(|row| {
            let is_selected = |span: &&TimedSpan| selected.contains(&span.cell);
            let first = row.iter().position(is_selected);
            let last = row.iter().rposition(is_selected);
            match (first, last) {
                (Some(first), Some(last)) => row[first..=last]
                    .iter()
                    .filter(|span| !is_selected(span))
                    .map(|span| **span)
                    .collect_vec(),
                _ => Vec::new(),
            }
        })
        .collect()
}

/// Unselected cells sounding at the same instant inside `window` as a
/// selected cell in a row above them and a selected cell in a row below them.
fn vertical_holes(timed: &[TimedSpan], selected: &[Cell], window: Timing) -> Vec<TimedSpan> {
    let selected_spans = timed
        .iter()
        .filter(|span| selected.contains(&span.cell))
        .collect_vec();
    timed
        .iter()
        .filter(|span| !selected.contains(&span.cell))
        .filter(|hole| {
            selected_spans
                .iter()
                .filter(|above| above.cell.part < hole.cell.part)
                .cartesian_product(
                    selected_spans
                        .iter()
                        .filter(|below| below.cell.part > hole.cell.part),
                )
                .any(|(above, below)| {
                    [hole.timing, above.timing, below.timing, window]
                        .iter()
                        .map(|timing| timing.onset)
                        .max()
                        < [hole.timing, above.timing, below.timing, window]
                            .iter()
                            .map(|timing| timing.end)
                            .min()
                })
        })
        .copied()
        .collect()
}

proptest! {
    #[test]
    fn click_and_click_selection_is_contiguous(
        score in score_strategy(),
        anchor_index in any::<Index>(),
        current_index in any::<Index>(),
    ) {
        let generated = to_source(&score);
        let source = &generated.text;
        let ListNoteSpansResponse::Ok { spans: note_spans } = list_note_spans_response(source, None) else {
            return Err(TestCaseError::fail(format!("notes failed to compile:\n{source}")));
        };
        let ListLyricSpansResponse::Ok { spans: lyric_spans } = list_lyric_spans_response(source, None) else {
            return Err(TestCaseError::fail(format!("lyrics failed to compile:\n{source}")));
        };
        let timing_of = |start: Option<usize>| {
            start.and_then(|start| generated.timing_by_start_byte.get(&start).copied())
        };
        let timed = note_spans
            .iter()
            .map(|span| (CellKind::Note, span.source_part_index, span.note_id, span.start))
            .chain(lyric_spans.iter().map(|span| {
                (CellKind::Lyric, span.source_part_index, span.note_id, Some(span.start))
            }))
            .map(|(kind, part, note_id, start)| {
                timing_of(start).map(|timing| TimedSpan {
                    cell: Cell { kind, part, note_id },
                    timing,
                })
            })
            .collect::<Option<Vec<_>>>();
        let Some(timed) = timed else {
            return Err(TestCaseError::fail(format!(
                "a span points at no generated token:\n{source}\n{note_spans:?}\n{lyric_spans:?}"
            )));
        };
        prop_assume!(!timed.is_empty());
        let anchor = anchor_index.get(&timed).cell;
        let current = current_index.get(&timed).cell;

        let response = resolve_selection_range_response(
            &note_spans,
            &lyric_spans,
            &anchor.clickable_element_id(),
            &current.clickable_element_id(),
        );
        let Some(selected) = selected_cells(&response) else {
            return Err(TestCaseError::fail(format!("unresolved:\n{source}")));
        };

        let context = format!(
            "\n{source}\nanchor: {anchor:?}\ncurrent: {current:?}\nselected: {selected:?}"
        );
        prop_assert!(selected.contains(&anchor), "anchor not selected{context}");
        prop_assert!(selected.contains(&current), "current not selected{context}");
        // A range between two syllables selects only syllables.
        let timed = if anchor.kind == CellKind::Lyric && current.kind == CellKind::Lyric {
            timed.into_iter().filter(|span| span.cell.kind == CellKind::Lyric).collect_vec()
        } else {
            timed
        };
        let horizontal = horizontal_holes(&timed, &selected);
        prop_assert!(horizontal.is_empty(), "horizontal holes: {horizontal:?}{context}");
        let window = endpoint_window(&timed, &[anchor, current]);
        let vertical = vertical_holes(&timed, &selected, window);
        prop_assert!(vertical.is_empty(), "vertical holes: {vertical:?}{context}");
        let expected = expected_cells(&timed, anchor, current, window);
        let missing = expected.iter().filter(|cell| !selected.contains(cell)).collect_vec();
        prop_assert!(missing.is_empty(), "missing: {missing:?}{context}");
        let extra = selected.iter().filter(|cell| !expected.contains(cell)).collect_vec();
        prop_assert!(extra.is_empty(), "outside the rows or window: {extra:?}{context}");
    }
}
