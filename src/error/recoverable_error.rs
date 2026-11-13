use crate::error::{RecoverableErrorKind, Span};

/// A recoverable error: render continues but the affected measure is highlighted red.
/// Displayed as a red view zone in the editor.
#[derive(Debug, Clone)]
pub struct RecoverableError {
    pub span: Span,
    pub kind: RecoverableErrorKind,
}

impl RecoverableError {
    pub fn message(&self) -> String {
        self.kind.message()
    }

    pub fn anyx_invalid(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::AnyxInvalid {
                message: message.into(),
            },
        }
    }

    pub fn general(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::General {
                message: message.into(),
            },
        }
    }

    pub fn lex_unexpected_char(span: Span, ch: char) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::LexUnexpectedChar { ch },
        }
    }

    pub fn measure_no_data_lines(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MeasureNoDataLines,
        }
    }

    pub fn dotted_eighth_needs_sixteenth(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DottedEighthNeedsSixteenth,
        }
    }

    pub fn extension_no_preceding_event(span: Span, chord_track: bool) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::ExtensionNoPrecedingEvent { chord_track },
        }
    }

    pub fn duration_unexpected_char(span: Span, ch: char) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DurationUnexpectedChar { ch },
        }
    }

    pub fn duration_mixed_octave_markers(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DurationMixedOctaveMarkers,
        }
    }

    pub fn duration_cannot_dot_quarter_beat(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DurationCannotDotQuarterBeat,
        }
    }

    pub fn duration_cannot_double_dot(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DurationCannotDoubleDot,
        }
    }

    pub fn tuplet_ambiguous_ratio(span: Span, num: u32) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::TupletAmbiguousRatio { num },
        }
    }

    pub fn tuplet_note_count_mismatch(span: Span, expected: u32, got: usize) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::TupletNoteCountMismatch { expected, got },
        }
    }

    pub fn measure_directives_missing(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MeasureDirectivesMissing,
        }
    }

    pub fn source_span_missing(span: Span, index: usize) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::SourceSpanMissing { index },
        }
    }

    pub fn timed_part_measure_missing(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::TimedPartMeasureMissing,
        }
    }

    pub fn part_measure_count_mismatch(
        span: Span,
        part: impl Into<String>,
        got: usize,
        expected: usize,
    ) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartMeasureCountMismatch {
                part: part.into(),
                got,
                expected,
            },
        }
    }
}

#[path = "recoverable_error_parts.rs"]
mod recoverable_error_parts;

#[path = "recoverable_error_presets.rs"]
mod recoverable_error_presets;
