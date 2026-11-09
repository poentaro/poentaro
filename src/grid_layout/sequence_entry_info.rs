//! `sequence` header-line types split out of `types.rs` to keep it under
//! the repo's max-line-count lint — re-exported via `types.rs`'s `pub use`,
//! so every existing `crate::grid_layout::types::SequenceEntryInfo`-style
//! path still resolves unchanged.

/// One `sequence` entry as rendered on the "Sequence: ..." header line:
/// a label, plus that entry's `(-abbrev ...)` / `(abbrev ...)` suffix (as
/// written), rendered parenthetically next to the label; `None` when the
/// entry has no suffix.
#[derive(Debug, Clone)]
pub struct SequenceEntryInfo {
    pub label: String,
    pub part_filter: Option<SequenceEntryPartFilter>,
}

/// The part abbreviations named in a `sequence` entry's `(...)` suffix,
/// plus whether they're an omit list (`(-abbrev ...)`) or an only list
/// (`(abbrev ...)`).
#[derive(Debug, Clone)]
pub struct SequenceEntryPartFilter {
    pub kind: crate::parser::sequence_parser::PartFilterKind,
    pub parts: Vec<String>,
}
