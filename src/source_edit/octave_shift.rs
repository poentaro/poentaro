//! Bulk-rewrites the `'`/`,` octave markers on every note belonging to one
//! part, so a part transcribed an octave too high/low can be corrected
//! without editing each note by hand.
//!
//! Infallible / best-effort, mirroring [`super::update_part_declaration`]'s
//! "not found -> unchanged" convention: an unknown abbreviation, a
//! `follow[X]` part (which has no notes of its own), or any parse failure
//! returns `source` unchanged.

use super::range_edit::{apply_edits, overlaps_any, ByteRange, RangeEditResult, SourceEdit};
use crate::ast::parsed::{ParsedMeasureSlot, ParsedTrack, ScoreEvent};
use crate::error::Span;
use crate::parser;

/// Shifts every note in the part named `abbreviation` by `delta` octaves,
/// rewriting each note's `'`/`,` marker run in place.
pub fn shift_part_octave(source: &str, abbreviation: &str, delta: i8) -> String {
    if delta == 0 {
        return source.to_string();
    }

    let Ok(document) = parser::parse(source, "input.poentaro", &[]) else {
        return source.to_string();
    };

    let is_follow = document
        .declarations
        .iter()
        .find(|decl| decl.abbreviation == abbreviation)
        .is_none_or(|decl| decl.follow_target.is_some());
    if is_follow {
        return source.to_string();
    }

    let Some(ParsedTrack::Timed(track)) = document.tracks.iter().find(|track| {
        let ParsedTrack::Timed(track) = track;
        track.abbreviation == abbreviation
    }) else {
        return source.to_string();
    };

    let edits = collect_octave_shift_edits(source, std::iter::once(track), delta, |_| true);
    apply_edits(source, edits)
}

/// Shifts every note whose span overlaps *any* of `ranges` by `delta`
/// octaves, across every part — the "selection octave" toolbar action,
/// distinct from [`shift_part_octave`]'s whole-part scope.
///
/// Takes a *list* of ranges rather than one `[start_byte, end_byte)` pair
/// because a multicursor selection (e.g. clicking a part label, which
/// selects that part's notes across every measure in its system) is
/// generally disjoint: collapsing it to a single min/max span would sweep in
/// unrelated notes/parts sitting between the disjoint pieces (e.g. another
/// part's line in between two selected measures of this one).
///
/// Notes belonging to a `follow[X]` part have no events of their own (see
/// [`shift_part_octave`]'s doc comment), so they're naturally left alone
/// without any extra check here.
pub fn shift_range_octave(source: &str, ranges: &[ByteRange], delta: i8) -> RangeEditResult {
    if delta == 0 || ranges.is_empty() {
        return RangeEditResult::unchanged(source);
    }

    let Ok(document) = parser::parse(source, "input.poentaro", &[]) else {
        return RangeEditResult::unchanged(source);
    };

    let tracks = document
        .tracks
        .iter()
        .map(|ParsedTrack::Timed(track)| track);

    let edits =
        collect_octave_shift_edits(source, tracks, delta, |span| overlaps_any(ranges, span));
    RangeEditResult::from_edits(source, ranges, edits)
}

/// Gathers the edits for every note across `tracks` that satisfies
/// `span_matches`, shifted by `delta` octaves. Shared between
/// [`shift_part_octave`] (called with a single track and an always-true
/// predicate) and [`shift_range_octave`] (every track, filtered by byte
/// overlap).
fn collect_octave_shift_edits<'a>(
    source: &str,
    tracks: impl Iterator<Item = &'a crate::ast::parsed::ParsedTimedTrack>,
    delta: i8,
    span_matches: impl Fn(Span) -> bool,
) -> Vec<SourceEdit> {
    tracks
        .flat_map(|track| track.measure_slots.iter())
        .filter_map(|slot| match slot {
            ParsedMeasureSlot::Real { events } => Some(events),
            ParsedMeasureSlot::EmptyNote { .. } => None,
        })
        .flatten()
        .filter_map(|spanned| {
            let ScoreEvent::Note(note) = &spanned.value else {
                return None;
            };
            if !span_matches(spanned.span) {
                return None;
            }
            let new_octave = note.octave.saturating_add(delta);
            if new_octave == note.octave {
                return None;
            }
            let text = source.get(spanned.span.start..spanned.span.end)?;
            // A tied continuation written as a bare repeat atom (`_`/`=`/`r`,
            // see `parse_repeat_unit`) copies its pitch *and* octave from the
            // note it repeats, with no pitch digit or `'`/`,` marker of its
            // own in the source — there's nowhere in its span to attach a
            // marker. Leave it untouched: re-parsing after the anchor note's
            // own marker is rewritten picks up the new octave automatically.
            if !text.starts_with(|c: char| c.is_ascii_digit()) {
                return None;
            }
            Some(SourceEdit {
                span: spanned.span,
                replacement: rewrite_octave_marker(text, new_octave),
            })
        })
        .collect()
}

/// Rewrites one note token's `'`/`,` octave marker to reflect `new_octave`,
/// preserving every other suffix character (duration/tie/dot) and their
/// relative order. The marker is re-inserted immediately after the
/// pitch+accidental head, matching the convention used throughout this
/// codebase's `.poentaro` sources (octave marker before duration/tie suffixes).
fn rewrite_octave_marker(text: &str, new_octave: i8) -> String {
    let split_at = text
        .char_indices()
        .find(|(_, c)| matches!(c, '_' | '=' | '-' | '.' | '~' | '\'' | ','))
        .map_or(text.len(), |(index, _)| index);
    let (head, remainder) = text.split_at(split_at);

    let remainder_without_octave: String = remainder
        .chars()
        .filter(|c| !matches!(c, '\'' | ','))
        .collect();

    let marker = match new_octave.cmp(&0) {
        std::cmp::Ordering::Greater => "'".repeat(new_octave as usize),
        std::cmp::Ordering::Less => ",".repeat((-new_octave) as usize),
        std::cmp::Ordering::Equal => String::new(),
    };

    format!("{head}{marker}{remainder_without_octave}")
}

#[cfg(test)]
#[path = "octave_shift_tests.rs"]
mod octave_shift_tests;
