mod irrecoverable;
mod recoverable_error;
mod recoverable_kind;

pub use irrecoverable::{IrrecoverableError, IrrecoverableErrorKind};
pub use recoverable_error::RecoverableError;
pub use recoverable_kind::RecoverableErrorKind;

/// One of the document's top-level sections (`sequence` is optional; the
/// others are required).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentSection {
    Metadata,
    Parts,
    Score,
    Sequence,
}

impl DocumentSection {
    pub fn header(self) -> &'static str {
        match self {
            Self::Metadata => "metadata { ... }",
            Self::Parts => "parts [ ... ]",
            Self::Score => "score \"\"\" ... \"\"\"",
            Self::Sequence => "sequence [ ... ]",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningKind {
    General,
    /// A chord symbol had an unrecognized quality/extension suffix.
    ChordUnknownSuffix,
    /// A slash-chord bass note could not be parsed.
    ChordInvalidBass,
    /// An unexpected character appeared while parsing a slash-chord bass note.
    ChordBassUnexpectedChar,
    /// A slash-chord bass note had trailing characters after the accidental.
    ChordBassTrailingChars,
    /// A note/rest duration crosses the half-bar boundary in 4/4 time.
    HalfBarBoundaryCrossed,
    /// A tie/slur group `(…)` contains fewer than 2 notes — group depth is not applied.
    GroupTooFewNotes,
    /// A system's measures' combined rod (hard minimum) width exceeds the
    /// page's usable music width — the layout overflows rather than
    /// over-compressing glyphs (see **Rod and spring** in `ARCHITECTURE.md`).
    MeasureOverflow,
    /// A preset draws staff notation (violin/cello), which ignores its text styles.
    PresetTextStylesIgnored,
    /// A note sounds outside the range of its part's instrument.
    NoteOutOfRange,
}

impl WarningKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::ChordUnknownSuffix => "chord_unknown_suffix",
            Self::ChordInvalidBass => "chord_invalid_bass",
            Self::ChordBassUnexpectedChar => "chord_bass_unexpected_char",
            Self::ChordBassTrailingChars => "chord_bass_trailing_chars",
            Self::HalfBarBoundaryCrossed => "half_bar_boundary_crossed",
            Self::GroupTooFewNotes => "group_too_few_notes",
            Self::MeasureOverflow => "measure_overflow",
            Self::PresetTextStylesIgnored => "preset_text_styles_ignored",
            Self::NoteOutOfRange => "note_out_of_range",
        }
    }
}

/// A recoverable warning: render continues and the score is still produced.
/// Displayed as an amber view zone in the editor.
#[derive(Debug, Clone)]
pub struct Warning {
    pub span: Span,
    pub message: String,
    pub kind: WarningKind,
}

impl Warning {
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            kind: WarningKind::General,
        }
    }

    pub fn half_bar_boundary_crossed(span: Span) -> Self {
        Self {
            span,
            message: "note/rest crosses the half-bar boundary (beat 2→3); use a beam group or tie to show the split".to_string(),
            kind: WarningKind::HalfBarBoundaryCrossed,
        }
    }

    pub fn note_out_of_range(span: Span, message: String) -> Self {
        Self {
            span,
            message,
            kind: WarningKind::NoteOutOfRange,
        }
    }

    pub fn group_too_few_notes(span: Span) -> Self {
        Self {
            span,
            message: "tie/slur group `(…)` must contain at least 2 notes".to_string(),
            kind: WarningKind::GroupTooFewNotes,
        }
    }

    pub fn measure_overflow(span: Span, needed_pt: f32, available_pt: f32) -> Self {
        Self {
            span,
            message: format!(
                "system needs {needed_pt:.1}pt of minimum width but only {available_pt:.1}pt \
                 is available; reduce measures per system or font size"
            ),
            kind: WarningKind::MeasureOverflow,
        }
    }

    pub fn preset_text_styles_ignored(span: Span, notation: &str) -> Self {
        Self {
            span,
            message: format!(
                "{notation} notation is drawn as a staff, which ignores this preset's text styles"
            ),
            kind: WarningKind::PresetTextStylesIgnored,
        }
    }
}

/// A per-measure diagnostic that is attached to rendered output.
/// `Warning` variants are shown as amber view zones; `Error` variants as red view zones.
#[derive(Debug, Clone)]
pub enum Diagnostic {
    Warning(Warning),
    Error(RecoverableError),
}

impl Diagnostic {
    pub fn span(&self) -> Span {
        match self {
            Self::Warning(w) => w.span,
            Self::Error(e) => e.span,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Warning(w) => w.message.clone(),
            Self::Error(e) => e.message(),
        }
    }

    /// Convert an `IrrecoverableError` that was caught on a chord line into a `Diagnostic`.
    /// Promoted kinds become `Diagnostic::Error`; others remain `Diagnostic::Warning`.
    pub fn from_chord_irrecoverable(error: &IrrecoverableError) -> Self {
        Self::Error(RecoverableError {
            span: error.span().copied().unwrap_or(Span::new(0, 0)),
            kind: RecoverableErrorKind::ChordInvalidToken {
                message: error.message(),
            },
        })
    }
}

#[cfg(test)]
mod tests;
