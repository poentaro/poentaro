use super::beat_padding::validate_and_pad_beats;
use super::errors::invariant;
use super::{lyrics_accumulator_mut, BarGroupContext, SlotAction, TrackAccumulator};
use crate::ast::parsed::{ParsedMeasureSlot, PartKind, ScoreEvent};
use crate::desugar::SourceLine;
use crate::error::{
    Diagnostic, IrrecoverableError, IrrecoverableErrorKind, RecoverableError, Span, Spanned,
};
use crate::parser::lyrics::tokenize_bracketed_lyrics;
use crate::parser::score::token_parser;
use crate::utils::{count_lyric_slots_in_events, tokenize_lyrics_with_errors};

fn is_recoverable_chord_line_error(_kind: &IrrecoverableErrorKind) -> bool {
    false
}

/// Tags every `Rest` event in `events` as `implicit_fill` — used when the whole
/// line they came from was synthesized to stand in for a part not mentioned
/// in this measure (see `SourceLine::is_implicit_fill`), so the rest renders
/// with a distinct glyph instead of an ordinary written `0`.
fn tag_implicit_rests(events: &mut [Spanned<ScoreEvent>]) {
    for event in events {
        if let ScoreEvent::Rest(rest) = &mut event.value {
            rest.implicit_fill = true;
        }
    }
}

/// A single column's source line, bundled to keep downstream
/// `process_*_column_line` functions under clippy's argument-count limit.
#[derive(Clone, Copy)]
struct ColumnLine<'a> {
    text: &'a str,
    offset: usize,
    /// True when this line was synthesized to fill a part not mentioned in
    /// this measure (see `SourceLine::is_implicit_fill`) — every `Rest`
    /// event parsed from it is tagged `ParsedRest::implicit_fill` to match.
    is_implicit_fill: bool,
}

pub(super) fn process_padded_columns(
    padded_data: &[SourceLine],
    beats_expected: u32,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), IrrecoverableError> {
    for (i, line) in padded_data.iter().enumerate() {
        process_column_line(
            i,
            ColumnLine {
                text: &line.content,
                offset: line.offset,
                is_implicit_fill: line.is_implicit_fill,
            },
            beats_expected,
            ctx,
        )?;
    }
    Ok(())
}

fn process_lyrics_column_line(
    track_index: usize,
    line: ColumnLine<'_>,
    line_span: Span,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), IrrecoverableError> {
    let ColumnLine {
        text: line,
        is_implicit_fill,
        ..
    } = line;
    let mut lyrics_parse_error = if line.is_empty() {
        Some(RecoverableError::lyrics_line_empty(line_span))
    } else {
        None
    };
    let is_chords_target = ctx
        .declarations
        .get(track_index)
        .and_then(|decl| match decl.kind {
            PartKind::Lyrics { target_part_index } => ctx.declarations.get(target_part_index),
            _ => None,
        })
        .is_some_and(|target| target.kind == PartKind::Chords);
    // Treat empty lines as `_`: no syllables for this measure.
    let syllables = if line.is_empty() || line == "_" {
        Vec::new()
    } else if is_chords_target {
        // Under a chord part, `( … )` items carry the timing (see
        // `grouper::lyrics_pairing`). The first bracket mistake becomes the
        // measure's lyric error, which also suppresses the bracket tally there
        // (it would only repeat the same problem); any others are reported
        // document-wide.
        let bracketed = tokenize_bracketed_lyrics(line, line_span.start);
        let mut errors = bracketed.errors.into_iter();
        lyrics_parse_error = errors.next();
        ctx.extra_document_errors.extend(errors);
        bracketed.syllables
    } else {
        let tokens = tokenize_lyrics_with_errors(line, line_span.start);
        ctx.extra_document_errors.extend(tokens.errors);
        tokens.syllables
    };

    let acc = ctx.accumulators.get_mut(track_index).ok_or_else(|| {
        invariant(
            line_span,
            "internal error: track accumulator index out of range",
        )
    })?;
    let Some(lyrics) = lyrics_accumulator_mut(acc)? else {
        return Err(invariant(
            line_span,
            "internal error: lyric line routed to a track that is not a lyric part",
        ));
    };
    let Some(current_measure) = lyrics.syllables.last_mut() else {
        return Err(invariant(
            line_span,
            "internal error: no measure bucket to push lyric line into",
        ));
    };
    *current_measure = syllables;
    // `line_starts`/`line_ends` already have a placeholder entry for this bar
    // group (pushed once per group in `process_bar_group`, alongside this same
    // track's syllables bucket) — overwrite it in place rather than pushing
    // again, so the two stay 1:1 with the target's measures. A line left to
    // implicit fill keeps the placeholder: its synthesized offset is not a
    // real position in the source.
    if !is_implicit_fill {
        if let Some(start) = lyrics.line_starts.last_mut() {
            *start = line_span.start;
        }
        if let Some(end) = lyrics.line_ends.last_mut() {
            *end = line_span.end;
        }
    }

    let acc = ctx.accumulators.get_mut(track_index).ok_or_else(|| {
        invariant(
            line_span,
            "internal error: track accumulator index out of range",
        )
    })?;
    let TrackAccumulator::Timed {
        per_measure_lyrics_errors,
        ..
    } = acc;
    per_measure_lyrics_errors.push(lyrics_parse_error);
    Ok(())
}

pub(super) fn push_skipped_notes_measure(
    ctx: &mut BarGroupContext<'_>,
    track_index: usize,
    line_span: Span,
    lex_error: Option<RecoverableError>,
) -> Result<(), IrrecoverableError> {
    let acc = ctx.accumulators.get_mut(track_index).ok_or_else(|| {
        invariant(
            line_span,
            "internal error: notes accumulator index out of range",
        )
    })?;
    let TrackAccumulator::Timed {
        per_measure_beat_errors,
        per_measure_dotted_eighth_errors,
        per_measure_lex_errors,
        per_measure_chord_errors,
        measure_slots,
        ..
    } = acc;
    per_measure_beat_errors.push(None);
    per_measure_dotted_eighth_errors.push(vec![]);
    per_measure_lex_errors.push(lex_error);
    per_measure_chord_errors.push(vec![]);
    measure_slots.push(ParsedMeasureSlot::EmptyNote { span: line_span });
    Ok(())
}

fn process_notes_column_line(
    track_index: usize,
    line: ColumnLine<'_>,
    beats_expected: u32,
    line_span: Span,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), IrrecoverableError> {
    let ColumnLine {
        text: line,
        offset: line_offset,
        is_implicit_fill,
    } = line;
    if line == "_" {
        return push_skipped_notes_measure(ctx, track_index, line_span, None);
    }
    let group_state = ctx
        .group_states
        .get_mut(track_index)
        .ok_or_else(|| invariant(line_span, "internal error: group state index out of range"))?;
    let is_percussion = ctx
        .declarations
        .get(track_index)
        .is_some_and(|decl| decl.kind == PartKind::Percussion);
    let notes_parse = if is_percussion {
        token_parser::parse_percussion_line(line, ctx.base_offset + line_offset, group_state)?
    } else {
        token_parser::parse_notes_line(line, ctx.base_offset + line_offset, group_state)?
    };
    let lex_error = notes_parse.lex_errors.into_iter().next();
    let mut padded = validate_and_pad_beats(
        notes_parse.events,
        beats_expected,
        *ctx.time_num,
        *ctx.time_den,
        line_span,
    )?;
    if is_implicit_fill {
        tag_implicit_rests(&mut padded.events);
    }
    if let Some(tie_state) = ctx.lyric_tie_states.get_mut(track_index) {
        let slots = count_lyric_slots_in_events(&padded.events, tie_state);
        if let Some(bar_slot) = ctx.bar_lyric_slots.get_mut(track_index) {
            *bar_slot = Some(slots);
        }
    }
    let acc = ctx.accumulators.get_mut(track_index).ok_or_else(|| {
        invariant(
            line_span,
            "internal error: notes accumulator index out of range",
        )
    })?;
    let TrackAccumulator::Timed {
        measure_slots,
        pending_events,
        per_measure_beat_errors,
        per_measure_dotted_eighth_errors,
        per_measure_lex_errors,
        per_measure_chord_errors,
        ..
    } = acc;
    let mut slot_events = std::mem::take(pending_events);
    slot_events.extend(padded.events);
    per_measure_beat_errors.push(padded.beat_overflow_error);
    per_measure_dotted_eighth_errors.push(padded.dotted_eighth_errors);
    per_measure_lex_errors.push(lex_error);
    per_measure_chord_errors.push(notes_parse.chord_errors);
    measure_slots.push(ParsedMeasureSlot::Real {
        events: slot_events,
    });
    Ok(())
}

fn process_chord_column_line(
    track_index: usize,
    line: ColumnLine<'_>,
    beats_expected: u32,
    line_span: Span,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), IrrecoverableError> {
    let ColumnLine {
        text: line,
        offset: line_offset,
        is_implicit_fill,
    } = line;
    let group_state = ctx
        .group_states
        .get_mut(track_index)
        .ok_or_else(|| invariant(line_span, "internal error: group state index out of range"))?;
    let chord_result =
        token_parser::parse_chord_line(line, ctx.base_offset + line_offset, group_state);
    let (chord_events, line_chord_errors) = match chord_result {
        Ok(parsed) => (parsed.events, parsed.chord_errors),
        Err(error) if is_recoverable_chord_line_error(&error.kind) => {
            let recoverable = Diagnostic::from_chord_irrecoverable(&error);
            (vec![], vec![recoverable])
        }
        Err(error) => return Err(error),
    };
    let line_failed = chord_events.is_empty() && !line_chord_errors.is_empty();
    let mut final_padded = validate_and_pad_beats(
        chord_events,
        beats_expected,
        *ctx.time_num,
        *ctx.time_den,
        line_span,
    )?;
    if line_failed {
        final_padded.beat_overflow_error = None;
    }
    if is_implicit_fill {
        tag_implicit_rests(&mut final_padded.events);
    }
    let acc = ctx.accumulators.get_mut(track_index).ok_or_else(|| {
        invariant(
            line_span,
            "internal error: chord accumulator index out of range",
        )
    })?;
    let TrackAccumulator::Timed {
        measure_slots,
        pending_events,
        per_measure_beat_errors,
        per_measure_dotted_eighth_errors,
        per_measure_chord_errors,
        ..
    } = acc;
    let mut slot_events = std::mem::take(pending_events);
    slot_events.extend(final_padded.events);
    per_measure_beat_errors.push(final_padded.beat_overflow_error);
    per_measure_dotted_eighth_errors.push(final_padded.dotted_eighth_errors);
    per_measure_chord_errors.push(line_chord_errors);
    measure_slots.push(ParsedMeasureSlot::Real {
        events: slot_events,
    });
    Ok(())
}

fn process_column_line(
    slot_idx: usize,
    line: ColumnLine<'_>,
    beats_expected: u32,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), IrrecoverableError> {
    let line_span = Span::new(
        ctx.base_offset + line.offset,
        ctx.base_offset + line.offset + line.text.len(),
    );
    let slot_action = ctx
        .slot_actions
        .get(slot_idx)
        .ok_or_else(|| invariant(line_span, "internal error: slot index out of range"))?;
    match slot_action {
        SlotAction::Notes { track_index } => {
            process_notes_column_line(*track_index, line, beats_expected, line_span, ctx)?;
        }
        SlotAction::Lyrics { track_index } => {
            process_lyrics_column_line(*track_index, line, line_span, ctx)?;
        }
        SlotAction::Chord { track_index } => {
            if line.text == "_" {
                return Ok(());
            }
            process_chord_column_line(*track_index, line, beats_expected, line_span, ctx)?;
        }
    }
    Ok(())
}
