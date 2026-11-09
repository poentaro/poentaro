use crate::ast::grouped::{Score, SequenceSpan};
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};
use crate::parser;

/// Find the index of the measure whose `source_span` contains `byte_offset`.
///
/// Returns `None` when `byte_offset` falls outside all measure spans
/// (e.g. in `metadata`, `parts`, or a directive line).
pub fn find_measure_at_byte_offset(score: &Score, byte_offset: usize) -> Option<usize> {
    score
        .measures
        .iter()
        .position(|m| m.source_span.start <= byte_offset && byte_offset <= m.source_span.end)
}

/// Source byte ranges for a measure in the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasureSourceSpan {
    /// Inclusive start of note content (for cursor/selection mapping).
    pub start: usize,
    /// Exclusive end of measure content in source.
    pub end: usize,
    /// Byte offset of the first source line in this measure group, for view zones.
    pub view_zone_start: usize,
    /// Section label from `label="..."` directive, if present on this measure.
    pub section_label: Option<String>,
    /// 1-indexed first line of this measure (inclusive).
    pub start_line: usize,
    /// 1-indexed last line of this measure (inclusive).
    pub end_line: usize,
}

/// Result of [`list_measure_spans_from_source`].
pub struct MeasureSpansResult {
    /// Source byte span of every measure, in source order, 1-to-1 with measures.
    pub spans: Vec<MeasureSourceSpan>,
    /// Resolved playback order from a `sequence` section, if present and valid.
    pub sequence: Option<Vec<SequenceSpan>>,
}

/// Return the source byte span of every measure in the compiled score, along
/// with the score's resolved `sequence` spans (if any).
///
/// Spans are in source order and correspond 1-to-1 with measures.
pub fn list_measure_spans_from_source(
    source: &str,
    filename: &str,
) -> Result<MeasureSpansResult, IrrecoverableError> {
    let score_text = parser::read_document(source, &[]).score;
    let (score_content, score_offset) = (score_text.text, score_text.offset);
    let base_line = source[..score_offset.min(source.len())]
        .bytes()
        .filter(|&b| b == b'\n')
        .count()
        + 1;
    let group_bounds =
        parser::score::measure_group::collect_group_bounds(&score_content, score_offset, base_line);

    let score = crate::compile(source, filename, &[])?;
    if group_bounds.len() != score.measures.len() {
        return Err(IrrecoverableError::new(
            IrrecoverableErrorKind::internal_invariant(
                Span::new(0, 0),
                format!(
                    "measure group bounds ({}) and measures ({}) out of sync",
                    group_bounds.len(),
                    score.measures.len()
                ),
            ),
        ));
    }

    let sequence = score.sequence.clone();
    let spans = score
        .measures
        .iter()
        .zip(group_bounds)
        .map(|(measure, bounds)| MeasureSourceSpan {
            start: measure.source_span.start,
            end: measure.source_span.end,
            view_zone_start: bounds.view_zone_start,
            section_label: measure.label.clone(),
            start_line: bounds.start_line,
            end_line: bounds.end_line,
        })
        .collect();

    Ok(MeasureSpansResult { spans, sequence })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_end_line_does_not_bleed_into_next_measure_when_implicit_parts_present() {
        // Regression: [G] (notes-only part with no score lines) received an implicit fill
        // "0 0 0 0" anchored at the offset of the last real line ([C] 1).  The fill
        // string is 7 bytes while "[C] 1" is only 5 bytes, so the synthetic span's end
        // overshot into the blank separator line and then into the first line of measure 2.
        // start_line/end_line are now derived from raw line counting in the score section,
        // not from the compiled source_span, so implicit fills cannot affect them.
        let source = r#"metadata { title "", author "" },

parts [
    "Alto 1 & Tenor" ["A1,T"]: notes,
    "Alto 1 & Tenor lyrics" ["A1,Tv1"]: lyrics ["A1,T"],
    "Alto 2" [A2]: follow ["A1,T"],
    "Soprano 1" [S1]: follow ["A1,T"],
    "Soprano 2" [S2]: follow [S1],
    "Chord" [C]: chords,
    "Guzheng" [G]: notes,
],

score """
    bpm=80 key=C4 time=4/4 label="Verse 1"
    [A1,T] 5_ 5_ 5_ 5= 5= 5_ 3_ 2_ (3_
    [A1,Tv1] la la la la la la la la la
    [C] 1

    [A1,T] 3_) (1_1-) 0_ 1= 1=
    [A1,Tv1] la la la
    [C] 6m/3
    """,
"#;
        let spans = list_measure_spans_from_source(source, "test.poentaro")
            .unwrap()
            .spans;
        assert_eq!(spans.len(), 2);
        assert_eq!(
            spans[0].end_line, 17,
            "measure 0 should end at line 17 ([C] 1)"
        );
        assert_eq!(spans[1].start_line, 19, "measure 1 should start at line 19");
        assert!(
            spans[0].end_line < spans[1].start_line,
            "measure 0 end_line ({}) must be before measure 1 start_line ({})",
            spans[0].end_line,
            spans[1].start_line,
        );
    }

    #[test]
    fn section_label_is_populated_for_bare_directive() {
        let source = r#"metadata { title "test" },

parts ["M": notes],

score """
    time=4/4 key=C4 bpm=120 label="A"
    1 2 3 4

    5 6 7 1'

    label="B"
    1' 7 6 5

    4 3 2 1
    """,
"#;
        let spans = list_measure_spans_from_source(source, "test.poentaro")
            .unwrap()
            .spans;
        assert_eq!(spans.len(), 4);
        assert_eq!(spans[0].section_label.as_deref(), Some("A"));
        assert_eq!(spans[1].section_label, None);
        assert_eq!(spans[2].section_label.as_deref(), Some("B"));
        assert_eq!(spans[3].section_label, None);
    }
}
