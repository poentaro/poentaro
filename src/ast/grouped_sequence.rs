//! A score's resolved `sequence`: split out of `grouped.rs` to keep it
//! under the line limit, and re-exported from there.

/// A labeled section's resolved measure range, in written-score order: from
/// the labeled measure up to (but not including) the next labeled measure,
/// or the end of the score.
#[derive(Clone)]
pub struct SequenceSpan {
    pub label: String,
    /// Inclusive index into `Score.measures`.
    pub start: usize,
    /// Inclusive index into `Score.measures`.
    pub end: usize,
    /// Individual part abbreviations to omit from this occurrence's MIDI/WAV
    /// playback (from a `sequence` entry's `(-abbrev ...)` omit suffix, or
    /// the complement of an `(abbrev ...)` only suffix against `parts`).
    pub omit_parts: Vec<String>,
    /// The entry's `(-abbrev ...)` / `(abbrev ...)` suffix's abbreviations as
    /// written, plus whether it's an omit or only suffix, for display on the
    /// SVG/PDF "Sequence: ..." summary line. `None` when the entry has no
    /// suffix.
    pub part_filter_display: Option<PartFilterDisplay>,
    /// The entry's `WITH ([w] 2, …)` choices that play a line other than the
    /// part's first, each naming an alternative line the score declares.
    pub chosen_lines: Vec<ChosenLine>,
    /// The entry's `WITH (…)` choices as written, for the "Sequence: ..."
    /// summary line.
    pub line_choices_display: Vec<ChosenLine>,
}

/// A part and the line of it a `sequence` entry plays (`[w] 2`).
#[derive(Debug, Clone, PartialEq)]
pub struct ChosenLine {
    pub part: String,
    /// 1-based; 1 is the part's own line.
    pub line: u32,
}

impl ChosenLine {
    /// The alternative line's abbreviation (`w.2`); `None` for line 1.
    pub fn alternative_abbreviation(&self) -> Option<String> {
        (self.line >= 2).then(|| {
            crate::ast::parsed::AlternativeLine {
                base: self.part.clone(),
                number: self.line,
            }
            .abbreviation()
        })
    }
}

/// A `sequence` entry's `(...)` suffix, as written, for display purposes.
#[derive(Debug, Clone, PartialEq)]
pub struct PartFilterDisplay {
    pub kind: crate::parser::sequence_parser::PartFilterKind,
    pub parts: Vec<String>,
}

/// A `sequence` entry's `WITHOUT` / `ONLY` parts as written in its
/// display suffix: `-S`, `-A2` for `WITHOUT`; `S`, `A2` for `ONLY`.
pub fn part_filter_items(
    kind: crate::parser::sequence_parser::PartFilterKind,
    parts: &[String],
) -> Vec<String> {
    use crate::parser::sequence_parser::PartFilterKind;
    let prefix = match kind {
        PartFilterKind::Omit => "-",
        PartFilterKind::Only => "",
    };
    parts.iter().map(|part| format!("{prefix}{part}")).collect()
}

/// A `sequence` entry's modifiers as one bracket after its label: its `WITH`
/// choices, then its `WITHOUT` / `ONLY` parts (from [`part_filter_items`]),
/// e.g. ` (w 2, -S, -A2)`. `None` when the entry has none. Shared by the
/// SVG/PDF "Sequence: ..." line and the web app's sequence buttons.
pub fn sequence_entry_suffix(
    lines: &[ChosenLine],
    part_filter_items: Vec<String>,
) -> Option<String> {
    let items = lines
        .iter()
        .map(|line| format!("{} {}", line.part, line.line))
        .chain(part_filter_items)
        .collect::<Vec<_>>();
    (!items.is_empty()).then(|| format!(" ({})", items.join(", ")))
}

impl SequenceSpan {
    /// The entry as one line of text, e.g. `Verse (w 2, -S)`: its label
    /// followed by the same suffix the "Sequence: ..." line shows.
    pub fn display_label(&self) -> String {
        let filter = self
            .part_filter_display
            .as_ref()
            .map(|display| part_filter_items(display.kind, &display.parts))
            .unwrap_or_default();
        let suffix = sequence_entry_suffix(&self.line_choices_display, filter).unwrap_or_default();
        format!("{}{suffix}", self.label)
    }
}
