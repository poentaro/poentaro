use super::beam::{flush_beam_buffer, BeamEntry};
use super::slur_chains::{extend_note_chains, PendingSlurOpen, SlurChainContext, SlurKey};
use super::tuplet_spans::{record_tuplet_tag, PendingTupletSpan, TupletSpanContext};
use crate::ast::parsed::TupletInfo;
use crate::compiler::types::{ColumnElement, ElementContent, SlurSpan, TupletSpan};

// ── Part slice compiler ───────────────────────────────────────────────────────

pub(super) struct PartState<'a> {
    pub(super) elements: &'a mut Vec<ColumnElement>,
    pub(super) beam_buf: &'a mut Vec<BeamEntry>,
    pub(super) pending_chains: &'a mut Vec<Vec<(u32, SlurKey)>>,
    pub(super) pending_slur_opens: &'a mut Vec<Option<PendingSlurOpen>>,
    pub(super) slur_spans: &'a mut Vec<SlurSpan>,
    /// Tuplet-bracket run currently being accumulated for this part, if any
    /// (see `tuplet_spans::record_tuplet_tag`). Never carried across
    /// measures — a fresh `None` is seeded per `compile_part_slice` call.
    pub(super) current_tuplet: &'a mut Option<PendingTupletSpan>,
    pub(super) tuplet_spans: &'a mut Vec<TupletSpan>,
    pub(super) col: &'a mut u32,
    pub(super) prev_tie: &'a mut bool,
    pub(super) prev_tie_column: &'a mut Option<u32>,
    pub(super) prev_tie_measure: &'a mut Option<usize>,
    pub(super) prev_tie_note_id: &'a mut Option<usize>,
    pub(super) next_note_id: &'a mut usize,
    pub(super) measure_index: usize,
    pub(super) part_index: usize,
    /// This measure's tuplet-rescale factor (`GroupedMeasure::resolution_multiplier`,
    /// carried onto `PartSlice`). `1` for measures with no tuplets — every
    /// quarter-beat-grid literal below (`4` per beat, underline-count thresholds
    /// `1`/`2`/`3`) is scaled by this factor so a rescaled measure still lays out
    /// correctly. Untagged (non-tuplet) notes/rests in a rescaled measure have their
    /// `duration` scaled by exactly this factor too, so the comparisons stay exact for
    /// them; a tuplet-tagged unit's `duration` is additionally ratio-compressed
    /// (`* den / num`) — `compile_unit` undoes that compression before comparing
    /// against these thresholds, so underline count still reflects the note's written
    /// duration, not its rescaled one.
    pub(super) multiplier: u32,
    /// This measure's beam-group width in quarter-beats (`PartSlice::beat_group_size`):
    /// `4` for simple meters, `6` for compound meters (6/8, 9/8, 12/8, ...). Scaled by
    /// `multiplier`, same as the other quarter-beat-grid constants, to decide when a
    /// run of beamed notes/rests flushes into a beam group.
    pub(super) beat_group_size: u32,
}

// ── Shared compile-unit abstraction ──────────────────────────────────────────

pub(super) struct CompiledUnit {
    pub(super) duration: u32,
    pub(super) dotted: bool,
    pub(super) double_dotted: bool,
    pub(super) group_membership: u8,
    pub(super) group_continuation: u8,
    pub(super) slur_close_at: Option<u32>,
    pub(super) slur_key: SlurKey,
    pub(super) tuplet: Option<TupletInfo>,
    pub(super) head: ElementContent,
}

/// Beam/underline count reflects each note or rest's *written* duration (e.g. `=` sixteenth
/// notes always get a double underline), not its tuplet-rescaled duration — a triplet
/// squeezing 3 sixteenth notes into an eighth note's space is still notated with the
/// sixteenth note's double beam underneath, with the tuplet bracket/number drawn as a
/// separate overlay (see `tuplet_spans.rs`). For tuplet-tagged units, undo the
/// `* den / num` rescale (exact, since `duration` was constructed as a multiple
/// of `den`) to recover the multiplier-scaled written duration before comparing it
/// against the thresholds below.
pub(super) fn written_underline_count(
    duration: u32,
    tuplet: Option<TupletInfo>,
    multiplier: u32,
) -> u32 {
    let scaled_written_duration = match tuplet {
        Some(TupletInfo { num, den, .. }) => duration / den * num,
        None => duration,
    };
    if scaled_written_duration == multiplier {
        2
    } else if scaled_written_duration == 2 * multiplier || scaled_written_duration == 3 * multiplier
    {
        1
    } else {
        0
    }
}

pub(super) fn compile_unit(
    state: &mut PartState<'_>,
    unit: CompiledUnit,
    measure_col_start: u32,
    note_id: usize,
) {
    record_tuplet_tag(
        &mut TupletSpanContext {
            current: state.current_tuplet,
            tuplet_spans: state.tuplet_spans,
            measure_index: state.measure_index,
            part_index: state.part_index,
        },
        *state.col,
        unit.tuplet,
    );

    let multiplier = state.multiplier;
    let underline_count = written_underline_count(unit.duration, unit.tuplet, multiplier);

    state.elements.push(ColumnElement {
        column: *state.col,
        content: unit.head,
        note_id: Some(note_id),
    });

    if underline_count == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }

    extend_note_chains(
        SlurChainContext {
            chains: state.pending_chains,
            pending_slur_opens: state.pending_slur_opens,
            slur_spans: state.slur_spans,
            measure_index: state.measure_index,
            part_index: state.part_index,
        },
        unit.group_membership,
        unit.group_continuation,
        *state.col,
        &unit.slur_key,
    );

    if let Some(close_offset) = unit.slur_close_at {
        if unit.group_membership > 0 {
            extend_note_chains(
                SlurChainContext {
                    chains: state.pending_chains,
                    pending_slur_opens: state.pending_slur_opens,
                    slur_spans: state.slur_spans,
                    measure_index: state.measure_index,
                    part_index: state.part_index,
                },
                unit.group_membership,
                0,
                *state.col + close_offset,
                &SlurKey::Rest,
            );
        }
    }

    // A dotted (or double-dotted) note's own written duration (e.g. a dotted quarter,
    // 6 quarter-beats) is one full dotted beat, so extensions past it (`-.`/`-..`) land
    // on 6-/7-quarter-beat boundaries rather than the 4-quarter-beat ones a plain note's
    // `-` extensions use.
    let beat = if unit.double_dotted {
        7 * multiplier
    } else if unit.dotted {
        6 * multiplier
    } else {
        4 * multiplier
    };
    let note_col = *state.col;
    for dash_col in (note_col + beat..note_col + unit.duration).step_by(beat as usize) {
        state.elements.push(ColumnElement {
            column: dash_col,
            content: ElementContent::NoteDash {
                dotted: unit.dotted,
                double_dotted: unit.double_dotted,
            },
            note_id: Some(note_id),
        });
    }

    if underline_count > 0 {
        state.beam_buf.push(BeamEntry {
            column: *state.col,
            underline_count,
            duration: unit.duration,
        });
    }

    *state.col += unit.duration;

    let beat_position = *state.col - measure_col_start;
    if underline_count > 0 && beat_position % (state.beat_group_size * multiplier) == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }
}
