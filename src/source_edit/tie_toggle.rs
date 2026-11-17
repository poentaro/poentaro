//! The editor toolbar's "Tie/Untie" action: a single toggle that either
//! removes the `~` ties the selection touches, or ties the selected
//! same-pitch notes together with new `~` suffixes.

use itertools::Itertools;

use super::range_edit::{overlaps_any, ByteRange, RangeEditResult, SourceEdit};
use crate::ast::parsed::{ParsedMeasureSlot, ParsedTimedTrack, ParsedTrack, ScoreEvent};
use crate::error::{Span, Spanned};
use crate::parser;

/// Toggles ties over the selection `ranges`:
///
/// - **Untie** — if any selected note/chord/percussion hit (in any part)
///   carries a `~`, removes the `~` of every such event.
/// - **Tie** — otherwise, in each part, ties every selected event into the
///   event immediately after it (no rest in between) when both carry the
///   same pitch (a tie between different pitches is an error). The last
///   selected event is only tied into its successor when it is the part's
///   *only* selected event, so selecting a single note ties it into the
///   next one, while a multi-note selection never ties out of itself.
///
/// Infallible / best-effort like [`super::toggle_range_slur`]: empty
/// `ranges`, a parse failure, or nothing to toggle all return `source`
/// unchanged with no remapped ranges.
pub fn toggle_range_tie(source: &str, ranges: &[ByteRange]) -> RangeEditResult {
    if ranges.is_empty() {
        return RangeEditResult::unchanged(source);
    }

    let Ok(document) = parser::parse(source, "input.poentaro", &[]) else {
        return RangeEditResult::unchanged(source);
    };

    let tracks = document
        .tracks
        .iter()
        .map(|ParsedTrack::Timed(track)| sounding_events(track))
        .collect_vec();

    let untie_edits = tracks
        .iter()
        .flatten()
        .filter(|event| overlaps_any(ranges, event.span))
        .filter_map(|event| tie_to_next_span(&event.value))
        .map(|tie_span| SourceEdit {
            span: tie_span,
            replacement: tie_removal_replacement(source, tie_span).to_string(),
        })
        .collect_vec();

    let edits = if untie_edits.is_empty() {
        tracks
            .iter()
            .flat_map(|events| tie_edits(events, ranges))
            .collect_vec()
    } else {
        untie_edits
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

/// `track`'s notes, chords, percussion hits and rests, in source order —
/// the events a tie can start from, end at, or be interrupted by.
fn sounding_events(track: &ParsedTimedTrack) -> Vec<&Spanned<ScoreEvent>> {
    track
        .measure_slots
        .iter()
        .filter_map(|slot| match slot {
            ParsedMeasureSlot::Real { events } => Some(events),
            ParsedMeasureSlot::EmptyNote { .. } => None,
        })
        .flatten()
        .filter(|spanned| {
            matches!(
                spanned.value,
                ScoreEvent::Note(_)
                    | ScoreEvent::Chord(_)
                    | ScoreEvent::PercussionHit(_)
                    | ScoreEvent::Rest(_)
            )
        })
        .unique_by(|spanned| spanned.span.start)
        .collect_vec()
}

fn tie_to_next_span(event: &ScoreEvent) -> Option<Span> {
    match event {
        ScoreEvent::Note(note) => note.tie_to_next_span,
        ScoreEvent::Chord(chord) => chord.tie_to_next_span,
        ScoreEvent::PercussionHit(hit) => hit.tie_to_next_span,
        _ => None,
    }
}

/// What a removed `~` is replaced with: a space when the next character
/// would otherwise glue onto the tied token and change its meaning (e.g.
/// `5~_` — a note tied into its own eighth-note repeat — must become `5 _`,
/// not the eighth note `5_`), otherwise nothing.
fn tie_removal_replacement(source: &str, tie_span: Span) -> &'static str {
    match source[tie_span.end..].chars().next() {
        Some(next) if next.is_ascii_digit() || matches!(next, '_' | '=' | 'r') => " ",
        _ => "",
    }
}

/// The `~` insertions tying `events`' selected same-pitch neighbours.
fn tie_edits(events: &[&Spanned<ScoreEvent>], ranges: &[ByteRange]) -> Vec<SourceEdit> {
    let selected_count = events
        .iter()
        .filter(|event| overlaps_any(ranges, event.span))
        .count();

    events
        .iter()
        .tuple_windows()
        .filter(|(current, next)| {
            overlaps_any(ranges, current.span)
                && (selected_count == 1 || overlaps_any(ranges, next.span))
                && tie_to_next_span(&current.value).is_none()
                && same_pitch(&current.value, &next.value)
        })
        .map(|(current, _)| SourceEdit {
            span: Span::new(current.span.end, current.span.end),
            replacement: "~".to_string(),
        })
        .collect_vec()
}

/// Whether a tie from `current` into `next` is valid: identical note pitch
/// (degree, accidental and octave), identical chord symbol, or two
/// percussion hits.
fn same_pitch(current: &ScoreEvent, next: &ScoreEvent) -> bool {
    match (current, next) {
        (ScoreEvent::Note(a), ScoreEvent::Note(b)) => {
            a.pitch == b.pitch && a.accidental == b.accidental && a.octave == b.octave
        }
        (ScoreEvent::Chord(a), ScoreEvent::Chord(b)) => {
            a.degree == b.degree
                && a.accidental == b.accidental
                && a.triad == b.triad
                && a.extension == b.extension
                && a.bass == b.bass
        }
        (ScoreEvent::PercussionHit(_), ScoreEvent::PercussionHit(_)) => true,
        _ => false,
    }
}

#[cfg(test)]
#[path = "tie_toggle_tests.rs"]
mod tie_toggle_tests;
