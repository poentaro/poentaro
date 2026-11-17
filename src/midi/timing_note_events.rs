use std::collections::HashMap;

use crate::ast::grouped::{MultiPartMeasure, NoteEvent, Score};
use crate::compiler::{visible_part_indices, MeasureBlock, MeasureRow};

use super::midi_notes::duration_to_ticks;

/// Per-part state carried across measures while walking note events in
/// `note_timings_seconds`'s playback pass. `note_id`s themselves come from
/// `build_written_note_id_lookup` (computed once over the *written* score,
/// matching `compiler::slur_chains::PartCrossState`); this cursor only
/// tracks tie continuation within the *expanded* (playback-order) walk.
pub(super) struct PartTimingCursor {
    /// `(note_id, index into the in-progress results vec)` of a note this
    /// part is currently tied into, so the next tied event can extend that
    /// entry's `end_tick` instead of starting a new one.
    pub(super) open_tie: Option<(usize, usize)>,
    /// `(block_index, index into the in-progress results vec)` of a merged
    /// `MultiMeasureRest` run this part is currently inside, so the next
    /// written measure belonging to the same compiled block can extend that
    /// entry's `end_tick` instead of starting a new one. Mirrors `open_tie`,
    /// but keyed by compiled block identity rather than `note_id` since every
    /// measure in the run shares one `note_id` from the start (see
    /// `compiler::merge_rest_run`).
    pub(super) open_rest_run: Option<(usize, usize)>,
}

impl PartTimingCursor {
    pub(super) fn new() -> Self {
        Self {
            open_tie: None,
            open_rest_run: None,
        }
    }
}

/// Computes, for every visible note/rest/chord/percussion event in the
/// *written* (un-expanded) `score`, the `note_id` the compiler assigns its
/// `ColumnElement` — keyed by `(written_measure_index, written_part_index,
/// event_index_within_that_part's_events)`. A tie continuation is keyed to
/// the same `note_id` as the note it continues from, mirroring
/// `compiler::part_slice::compile_timed_unit`'s `is_tie_continuation` reuse.
///
/// This is computed once over the written score (so it agrees with
/// `ColumnElement::note_id`) and then reused for every playback occurrence
/// of a given written event when walking the `sequence`/D.C.-expanded
/// timeline in `note_timings_seconds`, since a repeated or reordered measure
/// is still the same written notes, just sounding again at a different time.
pub(super) fn build_written_note_id_lookup(score: &Score) -> HashMap<(usize, usize, usize), usize> {
    struct Cursor {
        next_note_id: usize,
        open_tie_note_id: Option<usize>,
    }
    let max_parts = score
        .measures
        .iter()
        .map(|m| m.parts.len())
        .max()
        .unwrap_or(0);
    let mut cursors: Vec<Cursor> = (0..max_parts)
        .map(|_| Cursor {
            next_note_id: 0,
            open_tie_note_id: None,
        })
        .collect();
    let mut lookup = HashMap::new();

    for (measure_index, measure) in score.measures.iter().enumerate() {
        let visible = visible_part_indices(measure);
        for (part_idx, part_row) in measure.parts.iter().enumerate() {
            if !visible.contains(&part_idx) {
                continue;
            }
            while cursors.len() <= part_idx {
                cursors.push(Cursor {
                    next_note_id: 0,
                    open_tie_note_id: None,
                });
            }
            let Some(cursor) = cursors.get_mut(part_idx) else {
                continue;
            };
            for (event_idx, event) in part_row.slice().notes.events.iter().enumerate() {
                let fresh_id = cursor.next_note_id;
                cursor.next_note_id += 1;
                let (note_id, tie_to_next) = match event {
                    NoteEvent::Rest(_) => {
                        cursor.open_tie_note_id = None;
                        (fresh_id, false)
                    }
                    NoteEvent::Note(n) => (
                        cursor.open_tie_note_id.take().unwrap_or(fresh_id),
                        n.tie_to_next(),
                    ),
                    NoteEvent::Chord(c) => (
                        cursor.open_tie_note_id.take().unwrap_or(fresh_id),
                        c.tie_to_next(),
                    ),
                    NoteEvent::Percussion(p) => (
                        cursor.open_tie_note_id.take().unwrap_or(fresh_id),
                        p.tie_to_next(),
                    ),
                };
                lookup.insert((measure_index, part_idx, event_idx), note_id);
                if tie_to_next {
                    cursor.open_tie_note_id = Some(note_id);
                }
            }
        }
    }

    lookup
}

/// One note/chord/percussion event's tick span, as passed to
/// `push_or_extend_tick_span`.
struct TickSpanEvent {
    part_idx: usize,
    note_id: usize,
    start_tick: u32,
    duration: u32,
    tie_to_next: bool,
    /// This event's measure's tuplet-rescale factor (`GroupedMeasure::resolution_multiplier`,
    /// carried onto `PartSlice`), divided back out when converting `duration` to ticks. `1`
    /// for a measure with no tuplets.
    multiplier: u32,
}

/// Appends (or extends, for a tie continuation) one event's tick span onto
/// `results`, returning the tick the next event should start at.
fn push_or_extend_tick_span(
    cursor: &mut PartTimingCursor,
    results: &mut Vec<(usize, usize, u32, u32)>,
    event: &TickSpanEvent,
) -> u32 {
    let end_tick = event.start_tick + duration_to_ticks(event.duration, event.multiplier);
    if let Some((tie_note_id, result_idx)) = cursor.open_tie.take() {
        if let Some(entry) = results.get_mut(result_idx) {
            entry.3 = end_tick;
        }
        if event.tie_to_next {
            cursor.open_tie = Some((tie_note_id, result_idx));
        }
    } else {
        let result_idx = results.len();
        results.push((event.part_idx, event.note_id, event.start_tick, end_tick));
        if event.tie_to_next {
            cursor.open_tie = Some((event.note_id, result_idx));
        }
    }
    end_tick
}

/// Shared, per-measure context for [`record_measure_note_timings`]:
/// `written_measure_index` and `part_written_index` translate the
/// measure/part actually being walked (which may be a navigation-expanded
/// occurrence, or a plain written measure within a range) back to the
/// coordinates `note_id_lookup` and `block`/`block_index` (both computed
/// once over the *original whole* written score) are keyed by.
pub(super) struct MeasureTimingContext<'a, F: Fn(usize) -> usize> {
    pub(super) written_measure_index: usize,
    pub(super) part_written_index: F,
    pub(super) measure_start_tick: u32,
    pub(super) measure_end_tick: u32,
    pub(super) block_index: usize,
    pub(super) block: &'a MeasureBlock,
    pub(super) note_id_lookup: &'a HashMap<(usize, usize, usize), usize>,
}

/// One part's tick span for a `MultiMeasureRest` glyph run, as passed to
/// `record_rest_run_timing`.
#[derive(Clone, Copy)]
struct RestRunSpan {
    written_part_idx: usize,
    block_index: usize,
    measure_start_tick: u32,
    measure_end_tick: u32,
}

/// Emits (or extends) a single `NoteTiming` tuple spanning a whole
/// `MultiMeasureRest` glyph run for one part, using the glyph's own
/// `note_id` (`MeasureRow::first_note_id`).
fn record_rest_run_timing(
    block: &MeasureBlock,
    span: &RestRunSpan,
    cursors: &mut [PartTimingCursor],
    results: &mut Vec<(usize, usize, u32, u32)>,
) {
    let RestRunSpan {
        written_part_idx,
        block_index,
        measure_start_tick,
        measure_end_tick,
    } = *span;
    let note_id = block
        .rows
        .iter()
        .find(|row| row.source_part_index == written_part_idx)
        .and_then(MeasureRow::first_note_id);
    let Some(cursor) = cursors.get_mut(written_part_idx) else {
        return;
    };
    cursor.open_tie = None;
    let Some(note_id) = note_id else { return };
    match cursor.open_rest_run {
        Some((open_block_index, result_idx)) if open_block_index == block_index => {
            if let Some(entry) = results.get_mut(result_idx) {
                entry.3 = measure_end_tick;
            }
        }
        _ => {
            let result_idx = results.len();
            results.push((
                written_part_idx,
                note_id,
                measure_start_tick,
                measure_end_tick,
            ));
            cursor.open_rest_run = Some((block_index, result_idx));
        }
    }
}

/// Context for one part's event walk, as passed to
/// `record_part_note_events`.
#[derive(Clone, Copy)]
struct PartEventContext<'a> {
    written_part_idx: usize,
    written_measure_index: usize,
    measure_start_tick: u32,
    note_id_lookup: &'a HashMap<(usize, usize, usize), usize>,
    /// This part's slice's tuplet-rescale factor (`PartSlice::resolution_multiplier`),
    /// divided back out when converting event durations to ticks. `1` for a measure
    /// with no tuplets.
    multiplier: u32,
}

/// Records (or extends) tick spans for every visible event of one part's
/// slice of a measure.
fn record_part_note_events(
    events: &[NoteEvent],
    ctx: &PartEventContext<'_>,
    cursors: &mut [PartTimingCursor],
    results: &mut Vec<(usize, usize, u32, u32)>,
) {
    let PartEventContext {
        written_part_idx,
        written_measure_index,
        measure_start_tick,
        note_id_lookup,
        multiplier,
    } = *ctx;
    let mut tick = measure_start_tick;
    for (event_idx, event) in events.iter().enumerate() {
        let Some(cursor) = cursors.get_mut(written_part_idx) else {
            continue;
        };
        let event_note_id = note_id_lookup
            .get(&(written_measure_index, written_part_idx, event_idx))
            .copied()
            .unwrap_or(usize::MAX);

        tick = match event {
            NoteEvent::Rest(r) => {
                // Rests never continue a tie and are never tied into,
                // matching `compiler::part_slice::compile_rest`, which
                // unconditionally drops any pending tie state.
                cursor.open_tie = None;
                let end_tick = tick + duration_to_ticks(r.duration, multiplier);
                results.push((written_part_idx, event_note_id, tick, end_tick));
                end_tick
            }
            NoteEvent::Note(n) => push_or_extend_tick_span(
                cursor,
                results,
                &TickSpanEvent {
                    part_idx: written_part_idx,
                    note_id: event_note_id,
                    start_tick: tick,
                    duration: n.duration,
                    tie_to_next: n.tie_to_next(),
                    multiplier,
                },
            ),
            NoteEvent::Chord(c) => push_or_extend_tick_span(
                cursor,
                results,
                &TickSpanEvent {
                    part_idx: written_part_idx,
                    note_id: event_note_id,
                    start_tick: tick,
                    duration: c.duration,
                    tie_to_next: c.tie_to_next(),
                    multiplier,
                },
            ),
            NoteEvent::Percussion(p) => push_or_extend_tick_span(
                cursor,
                results,
                &TickSpanEvent {
                    part_idx: written_part_idx,
                    note_id: event_note_id,
                    start_tick: tick,
                    duration: p.duration,
                    tie_to_next: p.tie_to_next(),
                    multiplier,
                },
            ),
        };
    }
}

/// Records (or extends) tick spans for every visible event of one measure
/// into `results`, shared between the full-score playback walk in
/// [`super::timing::note_timings_seconds`] and the range-scoped walk in
/// [`super::timing::note_timings_seconds_for_range`].
pub(super) fn record_measure_note_timings(
    measure: &MultiPartMeasure,
    ctx: MeasureTimingContext<'_, impl Fn(usize) -> usize>,
    cursors: &mut Vec<PartTimingCursor>,
    results: &mut Vec<(usize, usize, u32, u32)>,
) {
    let MeasureTimingContext {
        written_measure_index,
        part_written_index,
        measure_start_tick,
        measure_end_tick,
        block_index,
        block,
        note_id_lookup,
    } = ctx;
    let visible = visible_part_indices(measure);
    for (part_idx, part_row) in measure.parts.iter().enumerate() {
        if !visible.contains(&part_idx) {
            continue;
        }
        let written_part_idx = part_written_index(part_idx);
        while cursors.len() <= written_part_idx {
            cursors.push(PartTimingCursor::new());
        }

        if block.represents_measures > 1 {
            // This written measure was merged into a `MultiMeasureRest`
            // glyph spanning `block.represents_measures` measures.
            record_rest_run_timing(
                block,
                &RestRunSpan {
                    written_part_idx,
                    block_index,
                    measure_start_tick,
                    measure_end_tick,
                },
                cursors,
                results,
            );
            continue;
        }
        if let Some(cursor) = cursors.get_mut(written_part_idx) {
            cursor.open_rest_run = None;
        }

        record_part_note_events(
            &part_row.slice().notes.events,
            &PartEventContext {
                written_part_idx,
                written_measure_index,
                measure_start_tick,
                note_id_lookup,
                multiplier: part_row.slice().resolution_multiplier,
            },
            cursors,
            results,
        );
    }
}
