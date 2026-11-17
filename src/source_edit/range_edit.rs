//! Shared plumbing for the editor toolbar's selection-scoped source edits
//! ("Octave up"/"Octave down", "Slur/Unslur"): the selection's byte ranges,
//! the edits an action produces, and remapping the selection through those
//! edits so the editor can restore it afterward.

use crate::error::Span;

/// One `[start_byte, end_byte)` span of the editor's selection. A Monaco
/// multicursor selection (e.g. one produced by clicking a part label, which
/// selects that part's notes across every measure in the system) surfaces as
/// several disjoint `ByteRange`s rather than one contiguous span — collapsing
/// them to a single min/max span would sweep in unrelated notes/parts sitting
/// between the disjoint pieces (e.g. another part's line in between two
/// selected measures of this one).
#[derive(Debug, Clone, Copy)]
pub struct ByteRange {
    pub start_byte: u32,
    pub end_byte: u32,
}

impl ByteRange {
    /// Whether `span` shares at least one byte with this range.
    pub(super) fn overlaps(&self, span: Span) -> bool {
        self.overlaps_bytes(span.start, span.end)
    }

    /// Whether `[start, end)` shares at least one byte with this range. A
    /// zero-width range (a bare caret) overlaps nothing.
    pub fn overlaps_bytes(&self, start: usize, end: usize) -> bool {
        start < self.end_byte as usize && end > self.start_byte as usize
    }
}

/// Whether `span` overlaps *any* of `ranges`.
pub(super) fn overlaps_any(ranges: &[ByteRange], span: Span) -> bool {
    ranges.iter().any(|range| range.overlaps(span))
}

/// One replacement of `span` in the source with `replacement`. A zero-width
/// `span` is a pure insertion; an empty `replacement` is a pure deletion.
#[derive(Debug, Clone)]
pub(super) struct SourceEdit {
    pub span: Span,
    pub replacement: String,
}

/// A selection-scoped edit's return value: the rewritten source, plus the
/// caller's own input `ranges` remapped forward through the edits — the
/// editor toolbar re-selects these afterward.
#[derive(Debug, Clone)]
pub struct RangeEditResult {
    pub source: String,
    /// The caller's input `ranges`, in the same order and count, each
    /// remapped to its new position in the post-edit text. A range's length
    /// can change (e.g. a two-note contiguous range selection whose notes'
    /// `'`/`,` marker runs each grow or shrink by a different amount) but its
    /// *shape* — how many ranges there are, contiguous vs. disjoint — is
    /// always preserved, so re-selecting these restores exactly what the
    /// caller had selected, just shifted onto the new text. Re-applying the
    /// caller's *pre-edit* `ranges` to the new text directly wouldn't work:
    /// each range's byte offsets only stay valid for text preceding every
    /// edit that lands before it. Empty when nothing was edited.
    pub ranges: Vec<ByteRange>,
}

impl RangeEditResult {
    pub(super) fn unchanged(source: &str) -> Self {
        Self {
            source: source.to_string(),
            ranges: Vec::new(),
        }
    }

    /// Applies `edits` to `source` and remaps `ranges` through them, or
    /// returns [`Self::unchanged`] when there are no edits.
    pub(super) fn from_edits(source: &str, ranges: &[ByteRange], edits: Vec<SourceEdit>) -> Self {
        if edits.is_empty() {
            return Self::unchanged(source);
        }
        Self {
            ranges: remap_ranges(ranges, &edits),
            source: apply_edits(source, edits),
        }
    }
}

/// Applies `edits` to `source`, rewriting from the end backwards so earlier
/// spans stay valid.
pub(super) fn apply_edits(source: &str, mut edits: Vec<SourceEdit>) -> String {
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.span.start));

    let mut result = source.to_string();
    for edit in edits {
        result.replace_range(edit.span.start..edit.span.end, &edit.replacement);
    }
    result
}

/// Which end of an input range [`remap_offset`] is resolving, controlling how
/// it behaves when the offset falls *inside* an edit's old span (rather than
/// cleanly before or after it): a range's start biases left to the edit's new
/// start, its end biases right to the edit's new end, so a range that begins
/// or ends mid-note still comes back covering that note's full new text
/// instead of collapsing to a zero-width point inside it.
#[derive(Clone, Copy)]
enum Bias {
    Left,
    Right,
}

/// Remaps each of the caller's input `ranges` forward through `edits` to its
/// new `[start, end)` byte span in the post-edit text, preserving the input
/// ranges' count and shape (unlike remapping each *edit's own* span forward,
/// which would turn one contiguous multi-note selection into one disjoint
/// range per edited note).
fn remap_ranges(ranges: &[ByteRange], edits: &[SourceEdit]) -> Vec<ByteRange> {
    let mut ascending: Vec<&SourceEdit> = edits.iter().collect();
    ascending.sort_by_key(|edit| edit.span.start);

    ranges
        .iter()
        .map(|range| ByteRange {
            start_byte: remap_offset(range.start_byte, &ascending, Bias::Left),
            end_byte: remap_offset(range.end_byte, &ascending, Bias::Right),
        })
        .collect()
}

/// Maps one byte offset in the pre-edit source to its position in the
/// post-edit text, given `edits_ascending` (sorted by span start). Walks the
/// edits in source order, accumulating how much the total byte length has
/// drifted (grown or shrunk) from every edit fully preceding `offset`; an
/// edit whose old span straddles `offset` is resolved via `bias` instead of
/// arithmetic, since `offset` has no well-defined position inside text that
/// got replaced wholesale.
fn remap_offset(offset: u32, edits_ascending: &[&SourceEdit], bias: Bias) -> u32 {
    let mut byte_length_delta: i64 = 0;
    for edit in edits_ascending {
        let (span, replacement) = (edit.span, &edit.replacement);
        if span.end as u32 <= offset {
            byte_length_delta += replacement.len() as i64 - (span.end - span.start) as i64;
            continue;
        }
        if span.start as u32 <= offset {
            let new_start = (span.start as i64 + byte_length_delta) as u32;
            return match bias {
                Bias::Left => new_start,
                Bias::Right => new_start + replacement.len() as u32,
            };
        }
        break;
    }
    (offset as i64 + byte_length_delta) as u32
}
