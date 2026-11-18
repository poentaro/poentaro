use super::depth_event::DepthEvent;
use super::duration::parse_duration_suffixes;
use super::groups::{apply_closed_group_depth, GroupStack, TupletStack};
use super::timed_lexer::TimedLexToken;
use super::{EventAttrs, ParseHeadError, TimedUnitHead};
use crate::ast::parsed::{ScoreEvent, TupletInfo};
use crate::error::{Diagnostic, IrrecoverableError, RecoverableError, Span, Spanned};

#[path = "timed_recursive_descent_parser/group_and_repeat.rs"]
mod group_and_repeat;

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// What closing token (if any) should end the current `parse_atoms` call and return
/// control to its caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StopAt {
    None,
    RParen,
    RBrace,
}

pub struct TimedRecursiveDescentParser<'a, H: TimedUnitHead> {
    source: &'a str,
    base_offset: usize,
    tokens: &'a [Spanned<TimedLexToken>],
    pos: usize,
    stack: &'a mut GroupStack,
    /// Tuplets never span lines (unlike groups), so this stack is always fresh per line.
    tuplet_stack: TupletStack,
    /// Staging area: events with their pending depth accumulators.
    staging: Vec<DepthEvent>,
    chord_errors: Vec<Diagnostic>,
    head: std::marker::PhantomData<H>,
}

type TimedLineParseResult = (Vec<Spanned<ScoreEvent>>, Vec<Diagnostic>);

impl<'a, H: TimedUnitHead> TimedRecursiveDescentParser<'a, H> {
    pub fn parse_line(
        source: &'a str,
        base_offset: usize,
        tokens: &'a [Spanned<TimedLexToken>],
        stack: &'a mut GroupStack,
    ) -> Result<TimedLineParseResult, IrrecoverableError> {
        // Frames carried over from a previous bar have segment_start values that
        // refer to the old staging vec.  Reset them to 0 so they cover all events
        // produced in this new call.
        for frame in stack.frames.iter_mut() {
            frame.segment_start = 0;
        }

        let mut parser = Self {
            source,
            base_offset,
            tokens,
            pos: 0,
            stack,
            tuplet_stack: TupletStack::default(),
            staging: Vec::new(),
            chord_errors: Vec::new(),
            head: std::marker::PhantomData,
        };
        std::hint::black_box(parser.head);
        parser.parse_atoms(StopAt::None)?;
        parser.finalize_open_frames()?;
        let events = parser
            .staging
            .into_iter()
            .map(|d| d.into_spanned())
            .collect();
        Ok((events, parser.chord_errors))
    }

    // -----------------------------------------------------------------------
    // Token stream helpers
    // -----------------------------------------------------------------------

    fn peek(&self) -> Option<&TimedLexToken> {
        self.tokens.get(self.pos).map(|s| &s.value)
    }

    fn peek_span(&self) -> Option<&Span> {
        self.tokens.get(self.pos).map(|s| &s.span)
    }

    fn bump(&mut self) -> Option<&Spanned<TimedLexToken>> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn current_span(&self) -> Span {
        self.peek_span()
            .cloned()
            .unwrap_or_else(|| Span::new(self.base_offset, self.base_offset))
    }

    // -----------------------------------------------------------------------
    // Core recursive methods
    // -----------------------------------------------------------------------

    fn parse_atoms(&mut self, stop_at: StopAt) -> Result<(), IrrecoverableError> {
        loop {
            match self.peek() {
                None => return Ok(()),
                Some(TimedLexToken::RParen) => {
                    if stop_at == StopAt::RParen {
                        return Ok(());
                    }
                    self.close_group()?;
                }
                Some(TimedLexToken::LParen) => {
                    self.open_group()?;
                }
                Some(TimedLexToken::LBrace { num, den }) => {
                    let (num, den) = (*num, *den);
                    self.open_tuplet(num, den)?;
                }
                Some(TimedLexToken::RBrace) => {
                    if stop_at == StopAt::RBrace {
                        return Ok(());
                    }
                    // No cross-line tuplets, so there is no "still open" leniency like
                    // groups get: a `}` with no matching `{` on this line is dropped.
                    let span = self.current_span();
                    self.bump();
                    self.chord_errors
                        .push(Diagnostic::Error(RecoverableError::general(
                            span,
                            "unexpected '}' — no open tuplet; '}' ignored",
                        )));
                }
                Some(TimedLexToken::Extension {
                    dotted,
                    double_dotted,
                }) => self.parse_extension(*dotted, *double_dotted),
                Some(TimedLexToken::Tilde) => self.parse_tilde()?,
                Some(TimedLexToken::HeadStart { offset }) => {
                    let offset = *offset;
                    self.parse_timed_unit(offset)?;
                }
                Some(TimedLexToken::Repeat { offset }) => {
                    let offset = *offset;
                    self.parse_repeat_unit(offset)?;
                }
                Some(TimedLexToken::Bpm(bpm)) => {
                    let bpm = *bpm;
                    let span = self.current_span();
                    self.bump();
                    self.staging.push(DepthEvent::new(Spanned::new(
                        ScoreEvent::BpmChange(bpm),
                        span,
                    )));
                }
                Some(TimedLexToken::KeyChange(key)) => {
                    let key = key.clone();
                    let span = self.current_span();
                    self.bump();
                    self.staging.push(DepthEvent::new(Spanned::new(
                        ScoreEvent::KeyChange(key),
                        span,
                    )));
                }
                Some(TimedLexToken::TimeSignature { num, den }) => {
                    let numerator = *num;
                    let denominator = *den;
                    let span = self.current_span();
                    self.bump();
                    self.staging.push(DepthEvent::new(Spanned::new(
                        ScoreEvent::TimeSignatureChange {
                            numerator,
                            denominator,
                        },
                        span,
                    )));
                }
            }
        }
    }

    fn parse_extension(&mut self, dotted: bool, double_dotted: bool) {
        let span = self.current_span();
        self.bump();
        self.staging.push(DepthEvent::new(Spanned::new(
            ScoreEvent::Extension {
                dotted,
                double_dotted,
            },
            span,
        )));
    }

    /// Handle a `~` token: it ties the most recently pushed event to the timed unit that
    /// immediately follows it (if any), then, unless that pair forms a note/chord tie (which
    /// carries its own arc via `tie_to_next_span`), applies closed-group depth to the pair.
    fn parse_tilde(&mut self) -> Result<(), IrrecoverableError> {
        if self.staging.is_empty() {
            self.bump();
            return Ok(());
        }
        let group_start = self.staging.len() - 1;
        let tilde_span = self.current_span();
        self.bump();
        if let Some(TimedLexToken::HeadStart { offset }) = self.peek() {
            let offset = *offset;
            self.parse_timed_unit(offset)?;
        }
        if let Some(slice) = self.staging.get_mut(group_start..) {
            // Notes and chords use tie_to_next_span (set by duration parser); applying
            // group depth here would create a spurious slur arc in addition to the tie arc.
            let is_note_tie = slice
                .iter()
                .any(|e| matches!(e.spanned.value, ScoreEvent::Note(_)));
            let is_chord_tie = slice
                .iter()
                .any(|e| matches!(e.spanned.value, ScoreEvent::Chord(_)));
            let is_percussion_tie = slice
                .iter()
                .any(|e| matches!(e.spanned.value, ScoreEvent::PercussionHit(_)));
            if is_note_tie || is_chord_tie || is_percussion_tie {
                // A `~` glued directly after a repeat atom (`r`/`_`/`=`) never goes through
                // `parse_duration_suffixes` — that's where `tie_to_next_span` normally gets
                // recorded while scanning a note's own suffix characters — so the tied event
                // here may not carry a tie span yet. Fill it in from the `~` token itself.
                if let Some(tied) = slice.first_mut() {
                    match &mut tied.spanned.value {
                        ScoreEvent::Note(note) if note.tie_to_next_span.is_none() => {
                            note.tie_to_next_span = Some(tilde_span);
                        }
                        ScoreEvent::Chord(chord) if chord.tie_to_next_span.is_none() => {
                            chord.tie_to_next_span = Some(tilde_span);
                        }
                        ScoreEvent::PercussionHit(hit) if hit.tie_to_next_span.is_none() => {
                            hit.tie_to_next_span = Some(tilde_span);
                        }
                        _ => {}
                    }
                }
            } else {
                apply_closed_group_depth(slice);
            }
        }
        Ok(())
    }

    /// Collect the recoverable diagnostics attached to a parsed `DurationParse` into
    /// `self.chord_errors`.
    fn collect_duration_suffix_diagnostics(&mut self, duration_meta: &super::DurationParse) {
        let errors = [
            &duration_meta.unexpected_char_error,
            &duration_meta.mixed_octave_markers_error,
            &duration_meta.cannot_dot_quarter_beat_error,
            &duration_meta.cannot_double_dot_error,
            &duration_meta.tie_on_rest_error,
        ];
        self.chord_errors
            .extend(errors.into_iter().flatten().cloned().map(Diagnostic::Error));
    }

    /// Parse one timed unit (note/rest/chord head + duration suffixes) starting at `digit_offset`
    /// (which is an absolute byte offset into `self.source`).
    fn parse_timed_unit(&mut self, digit_offset: usize) -> Result<(), IrrecoverableError> {
        // Relative byte offset from the start of `source`.
        let rel = digit_offset - self.base_offset;

        // Slice from the head offset to the end of the current non-whitespace word.
        // Duration suffixes are never whitespace, so the unit ends at the first whitespace char.
        let raw_text = self.source.get(rel..).unwrap_or_default();
        let text = raw_text
            .find(|c: char| c.is_whitespace() || matches!(c, '(' | ')' | '{' | '}'))
            .map(|ws_pos| &raw_text[..ws_pos])
            .unwrap_or(raw_text);

        let chars: Vec<char> = text.chars().collect();

        // Build a span that covers the single head character (will be refined after suffixes).
        let head_span = Span::new(digit_offset, digit_offset + 1);

        // Parse the head (note digit / chord symbol).
        let (head, head_end, is_rest, head_errors) = match H::parse_head(&chars, 0, &head_span) {
            Ok(parsed) => parsed,
            Err(ParseHeadError::Recoverable(maybe_diagnostic)) => {
                if let Some(diagnostic) = maybe_diagnostic {
                    self.chord_errors.push(diagnostic);
                }
                self.bump();
                self.skip_head_starts_before(self.unit_end_abs(digit_offset));
                return Ok(());
            }
            Err(ParseHeadError::Irrecoverable(error)) => return Err(error),
        };
        self.chord_errors.extend(head_errors);

        // Parse duration suffixes.
        let duration_meta = parse_duration_suffixes::<H>(&chars, 0, head_end, is_rest, &head_span)?;
        self.collect_duration_suffix_diagnostics(&duration_meta);

        let octave = if duration_meta.octave_up > 0 {
            duration_meta.octave_up
        } else {
            -duration_meta.octave_down
        };

        // Calculate the actual byte length covered by this unit.
        let unit_byte_len: usize = chars
            .get(..duration_meta.next_index)
            .unwrap_or_default()
            .iter()
            .map(|c| c.len_utf8())
            .sum();
        let unit_end_abs = digit_offset + unit_byte_len;
        let unit_span = Span::new(digit_offset, unit_end_abs);

        let mut event = H::to_event(
            &head,
            EventAttrs {
                duration: duration_meta.duration,
                dotted: duration_meta.dotted,
                double_dotted: duration_meta.double_dotted,
                octave,
                group_membership: 0,
                group_continuation: 0,
                tuplet: self.current_tuplet(),
            },
        );
        Self::attach_tie_to_next(&mut event, duration_meta.tie_to_next_span);
        self.stack.mark_slur_continuation(&mut event);
        if matches!(
            event,
            ScoreEvent::Note(_) | ScoreEvent::Chord(_) | ScoreEvent::PercussionHit(_)
        ) {
            self.stack.last_pitched_event = Some(event.clone());
        }

        self.staging
            .push(DepthEvent::new(Spanned::new(event, unit_span)));

        // Increment note count in the innermost open group frame(s) and tuplet frame(s).
        self.stack.increment_note_count();
        self.tuplet_stack.increment_note_count();

        // Consume the HeadStart token for this unit.
        self.bump();

        // Skip any HeadStart tokens that fall within the byte range of the unit we just parsed.
        // This happens when the lexer emits a HeadStart for a digit inside a multi-char symbol
        // (e.g. the `7` in chord `1m7`).
        self.skip_head_starts_before(unit_end_abs);

        Ok(())
    }

    /// The ratio of the innermost currently-open `{...}` tuplet, if any.
    fn current_tuplet(&self) -> Option<TupletInfo> {
        self.tuplet_stack.frames.last().map(|frame| TupletInfo {
            num: frame.num,
            den: frame.den,
            id: frame.id,
        })
    }

    /// Attach a `~` tie span to whichever event variant carries one.
    fn attach_tie_to_next(event: &mut ScoreEvent, tie_span: Option<Span>) {
        let Some(tie_span) = tie_span else {
            return;
        };
        match event {
            ScoreEvent::Note(note) => note.tie_to_next_span = Some(tie_span),
            ScoreEvent::Chord(chord) => chord.tie_to_next_span = Some(tie_span),
            ScoreEvent::PercussionHit(hit) => hit.tie_to_next_span = Some(tie_span),
            _ => {}
        }
    }

    fn unit_end_abs(&self, digit_offset: usize) -> usize {
        let rel = digit_offset.saturating_sub(self.base_offset);
        let raw_text = self.source.get(rel..).unwrap_or("");
        raw_text
            .find(|c: char| c.is_whitespace() || matches!(c, '(' | ')' | '{' | '}'))
            .map(|ws_pos| digit_offset + ws_pos)
            .unwrap_or_else(|| self.base_offset + self.source.len())
    }

    fn skip_head_starts_before(&mut self, unit_end_abs: usize) {
        while let Some(TimedLexToken::HeadStart { offset }) = self.peek() {
            if *offset < unit_end_abs {
                self.bump();
            } else {
                break;
            }
        }
    }
}
