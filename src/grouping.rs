use crate::ast::parsed::ScoreEvent;
use crate::error::{Diagnostic, IrrecoverableError, RecoverableError, Span, Spanned, Warning};

const HALF_BAR_BOUNDARY: u32 = 8;

struct TimedBeatFields {
    dotted: bool,
    duration: u32,
    group_membership: u8,
    tie_to_next: bool,
}

fn timed_beat_fields(event: &ScoreEvent) -> Option<TimedBeatFields> {
    match event {
        ScoreEvent::Note(note) => Some(TimedBeatFields {
            dotted: note.dotted,
            duration: note.duration,
            group_membership: note.group_membership,
            tie_to_next: note.tie_to_next(),
        }),
        ScoreEvent::Chord(chord) => Some(TimedBeatFields {
            dotted: chord.dotted,
            duration: chord.duration,
            group_membership: chord.group_membership,
            tie_to_next: chord.tie_to_next(),
        }),
        ScoreEvent::PercussionHit(hit) => Some(TimedBeatFields {
            dotted: hit.dotted,
            duration: hit.duration,
            group_membership: hit.group_membership,
            tie_to_next: hit.tie_to_next(),
        }),
        ScoreEvent::Rest(rest) => Some(TimedBeatFields {
            dotted: rest.dotted,
            duration: rest.duration,
            group_membership: 0,
            tie_to_next: false,
        }),
        _ => None,
    }
}

struct HalfBarCrossingCheck {
    group_membership: u8,
    tied_from_previous: bool,
    pos: u32,
    head_duration: u32,
    half_bar_boundary: u32,
    span: Span,
}

fn push_half_bar_crossing_warning(
    check: &HalfBarCrossingCheck,
    recoverable_errors: &mut Vec<Diagnostic>,
) {
    if check.group_membership == 0
        && !check.tied_from_previous
        && check.pos > 0
        && check.pos < check.half_bar_boundary
        && check.pos + check.head_duration > check.half_bar_boundary
    {
        recoverable_errors.push(Diagnostic::Warning(Warning::half_bar_boundary_crossed(
            check.span,
        )));
    }
}

#[derive(Clone, Copy)]
struct TimedClusterAdvance<'a> {
    events: &'a [Spanned<ScoreEvent>],
    index: usize,
    fields: &'a TimedBeatFields,
    span: &'a Span,
    multiplier: u32,
}

fn advance_timed_cluster(
    advance: &TimedClusterAdvance<'_>,
    pos: &mut u32,
    recoverable_errors: &mut Vec<Diagnostic>,
) -> Result<usize, IrrecoverableError> {
    let TimedClusterAdvance {
        events,
        index,
        fields,
        span,
        multiplier,
    } = *advance;
    if is_dotted_eighth_at_beat_start(fields.dotted, fields.duration, *pos, multiplier) {
        let next_timed = next_timed_index(events, index);
        if let Some(error) = validate_dotted_eighth_tail(events, next_timed, span, multiplier)? {
            recoverable_errors.push(error);
        }
        *pos += fields.duration + multiplier;
        return Ok(next_timed.map(|next| next + 1).unwrap_or(events.len()));
    }

    *pos += timed_cluster_duration(events, index, multiplier);
    Ok(index + timed_cluster_len(events, index))
}

/// Validates half-bar-boundary crossing and dotted-eighth-tail rules for one measure's
/// events, scaled by `multiplier` (see `GroupedMeasure::resolution_multiplier`).
///
/// The only call site (`interleaved_beat_padding::validate_and_pad_beats`) now passes
/// the measure's real tuplet-rescale multiplier — computed at parse time via
/// `crate::tuplet::resolution_multiplier_of`, the same math the grouper-stage rescale
/// pass uses — instead of always `1`; see the `crate::tuplet`
/// docs. Every threshold below is expressed as `BASE_CONST * multiplier` so
/// this function is tuplet-rescale-correct; it is also exercised directly with a non-1
/// multiplier, as the unit tests in `grouping_tuplet_tests.rs` do.
pub fn validate_measure_grouping(
    events: &[Spanned<ScoreEvent>],
    time_num: u8,
    time_den: u8,
    multiplier: u32,
) -> Result<Vec<Diagnostic>, IrrecoverableError> {
    if time_num != 4 || time_den != 4 {
        return Ok(vec![]);
    }

    let half_bar_boundary = HALF_BAR_BOUNDARY * multiplier;
    let mut pos = 0u32;
    let mut index = 0usize;
    let mut tied_from_previous = false;
    let mut recoverable_errors = Vec::new();
    while index < events.len() {
        let Some(event) = events.get(index) else {
            break;
        };

        match &event.value {
            ScoreEvent::Note(_)
            | ScoreEvent::Chord(_)
            | ScoreEvent::PercussionHit(_)
            | ScoreEvent::Rest(_) => {
                let Some(fields) = timed_beat_fields(&event.value) else {
                    index += 1;
                    continue;
                };
                let current_tie_to_next = fields.tie_to_next;
                push_half_bar_crossing_warning(
                    &HalfBarCrossingCheck {
                        group_membership: fields.group_membership,
                        tied_from_previous,
                        pos,
                        head_duration: timed_head_duration(events, index),
                        half_bar_boundary,
                        span: event.span,
                    },
                    &mut recoverable_errors,
                );
                index = advance_timed_cluster(
                    &TimedClusterAdvance {
                        events,
                        index,
                        fields: &fields,
                        span: &event.span,
                        multiplier,
                    },
                    &mut pos,
                    &mut recoverable_errors,
                )?;
                tied_from_previous = current_tie_to_next;
            }
            _ => index += 1,
        }
    }

    Ok(recoverable_errors)
}

fn timed_head_duration(events: &[Spanned<ScoreEvent>], start: usize) -> u32 {
    match events.get(start).map(|e| &e.value) {
        Some(ScoreEvent::Note(note)) => note.duration,
        Some(ScoreEvent::Chord(chord)) => chord.duration,
        Some(ScoreEvent::PercussionHit(hit)) => hit.duration,
        Some(ScoreEvent::Rest(rest)) => rest.duration,
        _ => 0,
    }
}

fn timed_cluster_duration(events: &[Spanned<ScoreEvent>], start: usize, multiplier: u32) -> u32 {
    let Some(event) = events.get(start) else {
        return 0;
    };
    let mut duration = match &event.value {
        ScoreEvent::Note(note) => note.duration,
        ScoreEvent::Chord(chord) => chord.duration,
        ScoreEvent::PercussionHit(hit) => hit.duration,
        ScoreEvent::Rest(rest) => rest.duration,
        _ => return 0,
    };

    let mut index = start + 1;
    while let Some(event) = events.get(index) {
        if let ScoreEvent::Extension {
            dotted,
            double_dotted,
        } = event.value
        {
            duration += if double_dotted {
                7
            } else if dotted {
                6
            } else {
                4
            } * multiplier;
            index += 1;
        } else {
            break;
        }
    }

    duration
}

fn timed_cluster_len(events: &[Spanned<ScoreEvent>], start: usize) -> usize {
    let mut len = 1usize;
    let mut index = start + 1;
    while let Some(event) = events.get(index) {
        if matches!(event.value, ScoreEvent::Extension { .. }) {
            len += 1;
            index += 1;
        } else {
            break;
        }
    }
    len
}

fn next_timed_index(events: &[Spanned<ScoreEvent>], start: usize) -> Option<usize> {
    let mut index = start + timed_cluster_len(events, start);
    while index < events.len() {
        if let Some(event) = events.get(index) {
            if matches!(
                event.value,
                ScoreEvent::Note(_)
                    | ScoreEvent::Chord(_)
                    | ScoreEvent::PercussionHit(_)
                    | ScoreEvent::Rest(_)
            ) {
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

fn is_dotted_eighth_at_beat_start(dotted: bool, duration: u32, pos: u32, multiplier: u32) -> bool {
    dotted && duration == 3 * multiplier && pos % (4 * multiplier) == 0
}

fn validate_dotted_eighth_tail(
    events: &[Spanned<ScoreEvent>],
    next_timed: Option<usize>,
    span: &Span,
    multiplier: u32,
) -> Result<Option<Diagnostic>, IrrecoverableError> {
    let Some(next_index) = next_timed else {
        return Ok(Some(Diagnostic::Error(
            RecoverableError::dotted_eighth_needs_sixteenth(*span),
        )));
    };
    let Some(event) = events.get(next_index) else {
        return Ok(Some(Diagnostic::Error(
            RecoverableError::dotted_eighth_needs_sixteenth(*span),
        )));
    };

    let tail_duration = match &event.value {
        ScoreEvent::Note(note) => note.duration,
        ScoreEvent::Chord(chord) => chord.duration,
        ScoreEvent::PercussionHit(hit) => hit.duration,
        ScoreEvent::Rest(rest) => rest.duration,
        _ => {
            return Ok(Some(Diagnostic::Error(
                RecoverableError::dotted_eighth_needs_sixteenth(*span),
            )))
        }
    };

    if tail_duration == multiplier {
        Ok(None)
    } else {
        Ok(Some(Diagnostic::Error(
            RecoverableError::dotted_eighth_needs_sixteenth(*span),
        )))
    }
}

#[cfg(test)]
#[path = "grouping_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "grouping_tuplet_tests.rs"]
mod grouping_tuplet_tests;
