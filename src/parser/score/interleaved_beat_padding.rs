use crate::ast::parsed::{ScoreEvent, ScoreLineSlot};
use crate::desugar::SourceLine;
use crate::error::{Diagnostic, IrrecoverableError, Span, Spanned, Warning};

pub(super) struct PaddedBeats {
    pub(super) events: Vec<Spanned<ScoreEvent>>,
    pub(super) beat_overflow_error: Option<Warning>,
    pub(super) dotted_eighth_errors: Vec<Diagnostic>,
}

pub(super) fn beats_per_measure(num: u8, den: u8) -> u32 {
    (num as u32) * (16 / den as u32)
}

pub(crate) fn timed_beats(event: &ScoreEvent) -> u32 {
    match event {
        ScoreEvent::Note(n) => n.duration,
        ScoreEvent::Chord(c) => c.duration,
        ScoreEvent::PercussionHit(p) => p.duration,
        ScoreEvent::Rest(r) => r.duration,
        ScoreEvent::Extension {
            dotted,
            double_dotted,
        } => {
            if *double_dotted {
                7
            } else if *dotted {
                6
            } else {
                4
            }
        }
        _ => 0,
    }
}

fn timed_beats_before_last(events: &[Spanned<ScoreEvent>]) -> (u32, u32) {
    let timed = events
        .iter()
        .filter_map(|e| {
            let beats = timed_beats(&e.value);
            (beats > 0).then_some(beats)
        })
        .collect::<Vec<_>>();

    let Some(&last) = timed.last() else {
        return (0, 0);
    };
    let before_last: u32 = timed.iter().take(timed.len().saturating_sub(1)).sum();
    (before_last, last)
}

fn timed_cluster_duration_at(events: &[Spanned<ScoreEvent>], start: usize) -> u32 {
    let Some(event) = events.get(start) else {
        return 0;
    };
    let mut duration = timed_beats(&event.value);
    if duration == 0 {
        return 0;
    }
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
            };
            index += 1;
        } else {
            break;
        }
    }
    duration
}

fn timed_cluster_len_at(events: &[Spanned<ScoreEvent>], start: usize) -> usize {
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

fn last_timed_cluster_start_and_duration(events: &[Spanned<ScoreEvent>]) -> Option<(u32, u32)> {
    let mut pos = 0u32;
    let mut index = 0usize;
    let mut last_cluster = None;
    while index < events.len() {
        let Some(event) = events.get(index) else {
            break;
        };
        if timed_beats(&event.value) > 0 {
            let duration = timed_cluster_duration_at(events, index);
            last_cluster = Some((pos, duration));
            pos += duration;
            index += timed_cluster_len_at(events, index);
        } else {
            index += 1;
        }
    }
    last_cluster
}

/// True when extending the last timed cluster by `deficit` would cross the 4/4 half-bar boundary.
fn extending_last_crosses_half_bar(events: &[Spanned<ScoreEvent>], deficit: u32) -> bool {
    let Some((start, duration)) = last_timed_cluster_start_and_duration(events) else {
        return false;
    };
    start > 0 && start < 8 && start + duration + deficit > 8
}

/// Implicit trailing `-` extensions apply only when earlier content fills whole beats
/// and the last note/rest is at least a quarter note (duration >= 4).
pub(crate) fn can_implicitly_pad(events: &[Spanned<ScoreEvent>], deficit: u32) -> bool {
    if deficit % 4 != 0 {
        return false;
    }

    let (before_last, last_beats) = timed_beats_before_last(events);
    last_beats >= 4 && before_last % 4 == 0
}

/// Validates measure capacity and pads omitted trailing `-` extensions when possible.
/// On beat overflow, truncates the events to fit and returns `Ok((truncated, Some(error), vec![]))`.
/// On underflow that cannot be implicitly padded, returns `Err`.
/// On dotted-eighth grouping violations, returns `Ok((events, None, errors))`.
fn timed_event_span(events: &[Spanned<ScoreEvent>]) -> Span {
    events
        .iter()
        .rev()
        .find(|event| {
            matches!(
                &event.value,
                ScoreEvent::Note(_)
                    | ScoreEvent::Chord(_)
                    | ScoreEvent::PercussionHit(_)
                    | ScoreEvent::Rest(_)
            )
        })
        .map(|event| event.span)
        .unwrap_or_else(|| Span::new(0, 1))
}

fn pad_beat_deficit(events: &mut Vec<Spanned<ScoreEvent>>, deficit: u32) {
    if extending_last_crosses_half_bar(events, deficit) {
        let pad_span = timed_event_span(events);
        for _ in 0..(deficit / 4) {
            events.push(Spanned::new(
                ScoreEvent::Extension {
                    dotted: false,
                    double_dotted: false,
                },
                pad_span,
            ));
        }
        return;
    }

    let Some(last_index) = events.iter().rposition(|event| {
        matches!(
            &event.value,
            ScoreEvent::Note(_)
                | ScoreEvent::Chord(_)
                | ScoreEvent::PercussionHit(_)
                | ScoreEvent::Rest(_)
        )
    }) else {
        return;
    };

    if let Some(last) = events.get_mut(last_index) {
        match &mut last.value {
            ScoreEvent::Note(note) => note.duration += deficit,
            ScoreEvent::Chord(chord) => chord.duration += deficit,
            ScoreEvent::PercussionHit(hit) => hit.duration += deficit,
            ScoreEvent::Rest(rest) => rest.duration += deficit,
            _ => {}
        }
    }
}

fn pad_incomplete_measure(
    mut events: Vec<Spanned<ScoreEvent>>,
    expected: u32,
    total: u32,
    line_span: Span,
) -> PaddedBeats {
    let deficit = expected - total;
    if !can_implicitly_pad(&events, deficit) {
        let error = Warning::new(
            line_span,
            format!(
                "incomplete measure: expected {expected} quarter-beats, got {total}; padding with rest"
            ),
        );
        let rest_span = events.last().map(|event| event.span).unwrap_or(line_span);
        events.push(Spanned::new(
            ScoreEvent::Rest(crate::ast::parsed::ParsedRest {
                duration: deficit,
                dotted: false,
                double_dotted: false,
                fermata: false,
                group_membership: 0,
                group_continuation: 0,
                tuplet: None,
                implicit_fill: false,
            }),
            rest_span,
        ));
        return PaddedBeats {
            events,
            beat_overflow_error: Some(error),
            dotted_eighth_errors: vec![],
        };
    }

    pad_beat_deficit(&mut events, deficit);
    PaddedBeats {
        events,
        beat_overflow_error: None,
        dotted_eighth_errors: vec![],
    }
}

pub(super) fn validate_and_pad_beats(
    events: Vec<Spanned<ScoreEvent>>,
    expected: u32,
    time_num: u8,
    time_den: u8,
    line_span: Span,
) -> Result<PaddedBeats, IrrecoverableError> {
    let multiplier = crate::tuplet::resolution_multiplier_of(&events);
    let padded = pad_beats(events, expected, line_span);
    if padded.beat_overflow_error.is_some() {
        return Ok(padded);
    }
    let dotted_eighth_errors =
        crate::grouping::validate_measure_grouping(&padded.events, time_num, time_den, multiplier)?;
    Ok(PaddedBeats {
        dotted_eighth_errors,
        ..padded
    })
}

/// One measure line's events as the parser lays them out against `expected`
/// quarter-beats: truncated at a beat overflow, or padded to fill a deficit.
/// Unlike [`validate_and_pad_beats`] it skips the beat-grouping checks.
pub(crate) fn padded_measure_events(
    events: Vec<Spanned<ScoreEvent>>,
    expected: u32,
) -> Vec<Spanned<ScoreEvent>> {
    pad_beats(events, expected, Span::new(0, 0)).events
}

fn pad_beats(events: Vec<Spanned<ScoreEvent>>, expected: u32, line_span: Span) -> PaddedBeats {
    let multiplier = crate::tuplet::resolution_multiplier_of(&events);
    let expected_rescaled = expected * multiplier;
    let rescaled = crate::tuplet::apply_resolution_multiplier(events.clone(), multiplier);

    let mut total_rescaled = 0u32;
    let mut truncate_at: Option<usize> = None;

    for (i, e) in rescaled.iter().enumerate() {
        let beats = timed_beats(&e.value);
        if beats > 0 {
            if total_rescaled + beats > expected_rescaled {
                truncate_at = Some(i);
                break;
            }
            total_rescaled += beats;
        }
    }

    let (events, overflow_error) = match truncate_at {
        Some(i) => {
            let error = Warning::new(
                line_span,
                format!(
                    "beat overflow: measure has {expected} quarter-beats but notes exceed that (truncated at note {})",
                    i + 1
                ),
            );
            (events.into_iter().take(i).collect(), Some(error))
        }
        None => (events, None),
    };

    if overflow_error.is_some() {
        return PaddedBeats {
            events,
            beat_overflow_error: overflow_error,
            dotted_eighth_errors: vec![],
        };
    }

    if total_rescaled < expected_rescaled {
        let deficit_written = (expected_rescaled - total_rescaled) / multiplier;
        let total_written = expected - deficit_written;
        return pad_incomplete_measure(events, expected, total_written, line_span);
    }

    PaddedBeats {
        events,
        beat_overflow_error: None,
        dotted_eighth_errors: vec![],
    }
}

fn implicit_source_line(offset: usize) -> SourceLine {
    SourceLine {
        content: "_".to_string(),
        offset,
        is_implicit_fill: false,
    }
}

pub(super) fn validate_and_pad_group_lines(
    group_lines: &[SourceLine],
    data_lines: &[SourceLine],
    slots: &[ScoreLineSlot],
    base_offset: usize,
) -> Result<Vec<SourceLine>, IrrecoverableError> {
    let group_first_span = group_lines
        .first()
        .map(|line| {
            Span::new(
                base_offset + line.offset,
                base_offset + line.offset + line.content.len(),
            )
        })
        .unwrap_or_else(|| Span::new(base_offset, base_offset));

    // These checks are defensive: desugar already normalises line counts.
    // If reached, pad or truncate silently rather than aborting parsing.
    if data_lines.is_empty() {
        return Ok(vec![implicit_source_line(group_first_span.start)]);
    }
    if data_lines.len() != slots.len() {
        let truncated: Vec<SourceLine> = data_lines
            .iter()
            .take(slots.len())
            .cloned()
            .chain(
                (data_lines.len()..slots.len())
                    .map(|_| implicit_source_line(group_first_span.start)),
            )
            .collect();
        return Ok(truncated);
    }

    Ok(data_lines.to_vec())
}
