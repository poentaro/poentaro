//! The sections of a document, and a preset's name.

use crate::error::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    Metadata,
    Parts,
    Score,
    Sequence,
    Preset,
}

/// The name in a `# preset "Name"` header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresetName {
    /// The name without its quotes.
    pub text: String,
    /// The quoted name, quotes included.
    pub span: Span,
}
