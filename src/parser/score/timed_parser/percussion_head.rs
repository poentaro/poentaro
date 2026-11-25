use super::{EventAttrs, ParseHeadError, TimedUnitHead};
use crate::ast::parsed::{ParsedPercussionHit, ParsedRest, ScoreEvent};
use crate::error::{Diagnostic, RecoverableError, RecoverableErrorKind, Span};

#[path = "percussion_head_tests.rs"]
#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct PercussionHead {
    is_rest: bool,
}

impl TimedUnitHead for PercussionHead {
    fn parse_head(
        chars: &[char],
        start: usize,
        span: &Span,
    ) -> Result<(Self, usize, bool, Vec<Diagnostic>), ParseHeadError> {
        let Some(&head_char) = chars.get(start) else {
            return Err(ParseHeadError::Recoverable(Some(Diagnostic::Error(
                RecoverableError {
                    span: *span,
                    kind: RecoverableErrorKind::PercussionExpectedHitOrRest { ch: '\0' },
                },
            ))));
        };
        match head_char {
            '0' => Ok((
                PercussionHead { is_rest: true },
                start + 1,
                true,
                Vec::new(),
            )),
            'x' => Ok((
                PercussionHead { is_rest: false },
                start + 1,
                false,
                Vec::new(),
            )),
            _ => {
                let pos = span.start + byte_offset_at_char_index_from_chars(chars, start);
                Err(ParseHeadError::Recoverable(Some(Diagnostic::Error(
                    RecoverableError {
                        span: Span::new(pos, pos + head_char.len_utf8()),
                        kind: RecoverableErrorKind::PercussionExpectedHitOrRest { ch: head_char },
                    },
                ))))
            }
        }
    }

    fn head_boundary(chars: &[char], i: usize) -> bool {
        matches!(chars.get(i), Some('0' | 'x'))
    }

    fn allows_octave_suffixes() -> bool {
        false
    }

    fn to_event(head: &Self, attrs: EventAttrs) -> ScoreEvent {
        let EventAttrs {
            duration,
            dotted,
            double_dotted,
            fermata: _,
            group_membership,
            group_continuation,
            tuplet,
            ..
        } = attrs;
        if head.is_rest {
            ScoreEvent::Rest(ParsedRest {
                duration,
                dotted,
                double_dotted,
                fermata: false,
                group_membership: 0,
                group_continuation: 0,
                tuplet,
                implicit_fill: false,
            })
        } else {
            ScoreEvent::PercussionHit(ParsedPercussionHit {
                duration,
                slur: group_continuation > 0,
                tie_to_next_span: None,
                group_membership,
                group_continuation,
                dotted,
                double_dotted,
                slur_group_close_at_duration: None,
                tuplet,
            })
        }
    }
}

fn byte_offset_at_char_index_from_chars(chars: &[char], char_index: usize) -> usize {
    chars
        .get(..char_index)
        .map(|slice| slice.iter().map(|c| c.len_utf8()).sum())
        .unwrap_or(0)
}
