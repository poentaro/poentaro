//! The editor toolbar's "Slur/Unslur" action: a single toggle that either
//! removes the `(…)` groups the selection touches, or wraps the selected
//! notes in a new `(…)` group.

use itertools::Itertools;

use super::range_edit::{overlaps_any, ByteRange, RangeEditResult, SourceEdit};
use crate::ast::parsed::{
    ParsedMeasureSlot, ParsedSlurGroup, ParsedTimedTrack, ParsedTrack, ScoreEvent,
};
use crate::error::Span;
use crate::parser;

/// Toggles slurs over the selection `ranges`:
///
/// - **Unslur** — if any closed `(…)` group (in any part) overlaps the
///   selection, removes the parentheses of every such group. A selection
///   touching even one note of a group removes that whole group.
/// - **Slur** — otherwise, for each part with at least two selected
///   notes/chords, wraps its first through last selected note in a new
///   `(…)` group. A part whose selected notes span several measures gets a
///   single cross-measure group, matching how a part-label click (one
///   disjoint range per measure) reads as "slur this phrase". A part with
///   fewer than two selected notes is left alone, since a one-note group is
///   a `group_too_few_notes` warning.
///
/// Infallible / best-effort like [`super::shift_range_octave`]: empty
/// `ranges`, a parse failure, or nothing to toggle all return `source`
/// unchanged with no remapped ranges.
pub fn toggle_range_slur(source: &str, ranges: &[ByteRange]) -> RangeEditResult {
    if ranges.is_empty() {
        return RangeEditResult::unchanged(source);
    }

    let Ok(document) = parser::parse(source, "input.poentaro", &[]) else {
        return RangeEditResult::unchanged(source);
    };

    let tracks = document
        .tracks
        .iter()
        .map(|ParsedTrack::Timed(track)| track)
        .collect_vec();

    let overlapping_groups = tracks
        .iter()
        .flat_map(|track| track.slur_groups.iter())
        .filter(|group| overlaps_any(ranges, whole_group_span(group)))
        .collect_vec();

    let edits = if overlapping_groups.is_empty() {
        tracks
            .iter()
            .flat_map(|track| slur_edits(track, ranges))
            .collect_vec()
    } else {
        overlapping_groups
            .into_iter()
            .flat_map(|group| [group.open_paren_span, group.close_paren_span])
            .map(|span| SourceEdit {
                span,
                replacement: String::new(),
            })
            .collect_vec()
    };

    // Desugaring (e.g. a `follow[X]` part, or a measure reused by several
    // parts) can surface the same source span on more than one track, so the
    // same edit may be collected twice — apply each only once.
    let edits = edits
        .into_iter()
        .unique_by(|edit| (edit.span.start, edit.span.end, edit.replacement.clone()))
        .collect_vec();

    RangeEditResult::from_edits(source, ranges, edits)
}

/// The span from a group's `(` through its `)`, inclusive.
fn whole_group_span(group: &ParsedSlurGroup) -> Span {
    Span::new(group.open_paren_span.start, group.close_paren_span.end)
}

/// The `(`/`)` insertions wrapping `track`'s selected notes/chords, or none
/// when fewer than two of them are selected.
fn slur_edits(track: &ParsedTimedTrack, ranges: &[ByteRange]) -> Vec<SourceEdit> {
    let selected_spans = track
        .measure_slots
        .iter()
        .filter_map(|slot| match slot {
            ParsedMeasureSlot::Real { events } => Some(events),
            ParsedMeasureSlot::EmptyNote { .. } => None,
        })
        .flatten()
        .filter(|spanned| matches!(spanned.value, ScoreEvent::Note(_) | ScoreEvent::Chord(_)))
        .map(|spanned| spanned.span)
        .filter(|span| overlaps_any(ranges, *span))
        .unique_by(|span| span.start)
        .sorted_by_key(|span| span.start)
        .collect_vec();

    let (Some(first), Some(last)) = (selected_spans.first(), selected_spans.last()) else {
        return Vec::new();
    };
    if selected_spans.len() < 2 {
        return Vec::new();
    }

    vec![
        SourceEdit {
            span: Span::new(first.start, first.start),
            replacement: "(".to_string(),
        },
        SourceEdit {
            span: Span::new(last.end, last.end),
            replacement: ")".to_string(),
        },
    ]
}

#[cfg(test)]
#[path = "slur_toggle_tests.rs"]
mod slur_toggle_tests;
