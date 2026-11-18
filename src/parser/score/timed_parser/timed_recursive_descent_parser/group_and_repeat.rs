use super::super::depth_event::{annotate_slur_close_via_extension, DepthEvent};
use super::super::groups::{
    apply_closed_group_depth, apply_open_group_depth, implicit_tuplet_ratio,
    validate_group_note_count,
};
use super::super::timed_lexer::TimedLexToken;
use super::super::TimedUnitHead;
use super::{StopAt, TimedRecursiveDescentParser};
use crate::ast::parsed::ScoreEvent;
use crate::error::{
    Diagnostic, IrrecoverableError, IrrecoverableErrorKind, RecoverableError, Span, Spanned,
};

impl<'a, H: TimedUnitHead> TimedRecursiveDescentParser<'a, H> {
    /// Parse a repeat atom (`r`, bare `_`, or bare `=`) starting at `offset`, which repeats
    /// the last pitched note/chord (skipping rests) as a fresh, standalone attack.
    pub(super) fn parse_repeat_unit(&mut self, offset: usize) -> Result<(), IrrecoverableError> {
        let rel = offset - self.base_offset;
        let ch = self.source[rel..].chars().next().unwrap_or('r');
        let len = ch.len_utf8();
        let span = Span::new(offset, offset + len);
        let duration = match ch {
            'r' => 4,
            '_' => 2,
            _ => 1, // '='
        };

        let Some(mut event) = self.stack.last_pitched_event.clone() else {
            self.chord_errors
                .push(Diagnostic::Error(RecoverableError::repeat_no_prior_note(
                    span,
                )));
            self.bump();
            return Ok(());
        };

        let tuplet = self.current_tuplet();

        match &mut event {
            ScoreEvent::Note(note) => {
                note.duration = duration;
                note.dotted = false;
                note.tie_to_next_span = None;
                note.group_membership = 0;
                note.group_continuation = 0;
                note.slur = false;
                note.slur_group_close_at_duration = None;
                note.tuplet = tuplet;
            }
            ScoreEvent::Chord(chord) => {
                chord.duration = duration;
                chord.dotted = false;
                chord.tie_to_next_span = None;
                chord.group_membership = 0;
                chord.group_continuation = 0;
                chord.slur = false;
                chord.slur_group_close_at_duration = None;
                chord.tuplet = tuplet;
            }
            ScoreEvent::PercussionHit(hit) => {
                hit.duration = duration;
                hit.dotted = false;
                hit.tie_to_next_span = None;
                hit.group_membership = 0;
                hit.group_continuation = 0;
                hit.slur = false;
                hit.slur_group_close_at_duration = None;
                hit.tuplet = tuplet;
            }
            _ => {}
        }

        self.stack.mark_slur_continuation(&mut event);
        self.stack.last_pitched_event = Some(event.clone());
        self.staging
            .push(DepthEvent::new(Spanned::new(event, span)));
        self.stack.increment_note_count();
        self.tuplet_stack.increment_note_count();
        self.bump();

        Ok(())
    }
    /// Handle `(` — push a new frame and recurse into the inner atom sequence.
    pub(super) fn open_group(&mut self) -> Result<(), IrrecoverableError> {
        let lparen_span = self.current_span();
        self.bump(); // consume LParen

        let segment_start = self.staging.len();
        self.stack.push(segment_start, lparen_span);

        // Parse inner atoms until `)` or end of token stream.
        self.parse_atoms(StopAt::RParen)?;

        // Now we should see `)` or end of stream.
        match self.peek() {
            Some(TimedLexToken::RParen) => {
                // Closed group — consume and apply closed-group depth.
                let rparen_span = self.current_span();
                self.bump();

                let frame = self.stack.close(rparen_span).ok_or_else(|| {
                    IrrecoverableError::new(IrrecoverableErrorKind::internal_invariant(
                        rparen_span,
                        "open_group: stack empty after push",
                    ))
                })?;

                let note_count = frame.note_count;
                if let Some(warning) = validate_group_note_count(note_count, &rparen_span) {
                    self.chord_errors.push(Diagnostic::Warning(warning));
                } else if let Some(slice) = self.staging.get_mut(frame.segment_start..) {
                    apply_closed_group_depth(slice);
                }
            }
            _ => {
                // No closing paren — treat as an open (cross-line) group: apply open depth.
                // The frame stays on the stack for `finalize_open_frames`.
            }
        }

        Ok(())
    }

    /// Handle `)` when encountered outside of `parse_atoms(stop_at_rparen=true)`.
    /// This closes the innermost frame that was left open from a previous call.
    pub(super) fn close_group(&mut self) -> Result<(), IrrecoverableError> {
        let rparen_span = self.current_span();
        self.bump(); // consume RParen

        let Some(frame) = self.stack.close(rparen_span) else {
            self.chord_errors.push(Diagnostic::Error(RecoverableError {
                span: rparen_span,
                kind: crate::error::RecoverableErrorKind::GroupUnexpectedCloseParen,
            }));
            return Ok(());
        };

        let note_count = frame.note_count;
        if let Some(warning) = validate_group_note_count(note_count, &rparen_span) {
            self.chord_errors.push(Diagnostic::Warning(warning));
        } else if let Some(slice) = self.staging.get_mut(frame.segment_start..) {
            apply_closed_group_depth(slice);
            annotate_slur_close_via_extension(slice);
        }

        Ok(())
    }

    /// Handle `N:{` / `N:M:{` — push a new tuplet frame, resolve its ratio, and recurse
    /// into the inner atom sequence until the matching `}`.
    pub(super) fn open_tuplet(
        &mut self,
        num: u32,
        den: Option<u32>,
    ) -> Result<(), IrrecoverableError> {
        let lbrace_span = self.current_span();
        self.bump(); // consume LBrace

        let resolved_den = den.or_else(|| implicit_tuplet_ratio(num));
        if resolved_den.is_none() {
            self.chord_errors
                .push(Diagnostic::Error(RecoverableError::tuplet_ambiguous_ratio(
                    lbrace_span,
                    num,
                )));
        }
        // Fall back to an identity ratio (no rescale) so parsing/note-counting can still
        // proceed sanely after the error above has been reported.
        let den = resolved_den.unwrap_or(num);

        let segment_start = self.staging.len();
        self.tuplet_stack.open_tuplet(segment_start, num, den);

        // Parse inner atoms until `}` or end of token stream.
        self.parse_atoms(StopAt::RBrace)?;

        match self.peek() {
            Some(TimedLexToken::RBrace) => {
                let rbrace_span = self.current_span();
                self.bump();

                let frame = self.tuplet_stack.close_tuplet().ok_or_else(|| {
                    IrrecoverableError::new(IrrecoverableErrorKind::internal_invariant(
                        rbrace_span,
                        "open_tuplet: stack empty after push",
                    ))
                })?;

                if frame.note_count != frame.num as usize {
                    self.chord_errors.push(Diagnostic::Error(
                        RecoverableError::tuplet_note_count_mismatch(
                            rbrace_span,
                            frame.num,
                            frame.note_count,
                        ),
                    ));
                }
            }
            _ => {
                // No cross-line tuplets: an unclosed `{` at end of line is a hard parse
                // error, unlike `(` groups which get silently treated as still-open.
                self.tuplet_stack.close_tuplet();
                self.chord_errors
                    .push(Diagnostic::Error(RecoverableError::general(
                        lbrace_span,
                        "unclosed '{' tuplet at end of line",
                    )));
            }
        }

        Ok(())
    }

    /// At end-of-line, any frames still on the stack represent open (cross-line) groups.
    /// Apply open-group depth to the events that belong to those frames.
    pub(super) fn finalize_open_frames(&mut self) -> Result<(), IrrecoverableError> {
        // Iterate from outermost to innermost (bottom of stack to top).
        for frame in &self.stack.frames {
            if let Some(slice) = self.staging.get_mut(frame.segment_start..) {
                apply_open_group_depth(slice);
            }
        }
        Ok(())
    }
}
