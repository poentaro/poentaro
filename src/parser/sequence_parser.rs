use crate::error::Span;

/// Whether a `sequence` entry's `(...)` suffix names parts to omit or the
/// only parts to keep, for that one occurrence's playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartFilterKind {
    /// `(-abbrev -abbrev ...)` — omit these parts from this occurrence.
    Omit,
    /// `(abbrev abbrev ...)` — play only these parts on this occurrence.
    Only,
}

/// A `sequence` entry's optional `(...)` suffix: either a `(-abbrev ...)`
/// list of parts to omit, or an `(abbrev ...)` list of the only parts to
/// keep, for that one occurrence's playback (e.g. `Chorus(-S -A2)` or
/// `Chorus(S A2)`).
#[derive(Debug, Clone, PartialEq)]
pub struct PartFilter {
    pub kind: PartFilterKind,
    pub parts: Vec<String>,
    /// Byte span of each `parts` abbreviation (excluding any leading `-`),
    /// parallel to `parts`, used by rename-symbol to locate these references.
    pub part_spans: Vec<Span>,
}

/// One label reference inside a `sequence` section.
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceEntry {
    pub label: String,
    /// Byte span of just the label substring (not the `(...)` suffix), used
    /// by rename-symbol to locate this reference site.
    pub label_span: Span,
    /// Optional `(-abbrev ...)` / `(abbrev ...)` suffix naming part
    /// abbreviations to omit from, or restrict this occurrence's playback to.
    pub part_filter: Option<PartFilter>,
    pub span: Span,
}

/// The parsed `sequence` section: an ordered list of section-label
/// references defining playback order.
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceSection {
    pub entries: Vec<SequenceEntry>,
}
