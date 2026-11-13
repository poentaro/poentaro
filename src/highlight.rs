//! Keyword highlighting for the editor: the spans of every `.poentaro`
//! keyword, found by the same lexers the parser uses, so the editor never
//! re-spells the vocabulary. See `ARCHITECTURE.md` ("Highlight token").

use crate::error::Span;
use crate::parser::section::SectionKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightKind {
    /// A score directive keyword (`bpm`, `key`, …, `break`).
    DirectiveKey,
    /// The reading after a Chinese lyric character, `(háng)` in `行(háng)`.
    LyricReading,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighlightToken {
    pub kind: HighlightKind,
    pub span: Span,
}

/// Every keyword in `source`, ordered by position. Each token lies on a
/// single line.
pub fn highlight_tokens(source: &str) -> Vec<HighlightToken> {
    anyx_highlight::highlight_tokens(source)
}

/// A known section: which one, and its header keyword's span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionHeader {
    pub kind: SectionKind,
    pub span: Span,
}

/// Every known section header in `source`, ordered by position.
pub fn section_headers(source: &str) -> Vec<SectionHeader> {
    anyx_highlight::section_headers(source)
}

mod anyx_highlight;
