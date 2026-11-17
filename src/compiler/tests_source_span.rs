// ── `MeasureBlock.source_span` ────────────────────────────────────────────────

use crate::compiler::compile;
use crate::grouper::group;
use crate::parser::parse;

fn score_from(source: &str) -> crate::ast::grouped::Score {
    let doc = parse(source, "test", &[]).unwrap();
    group(doc).unwrap()
}

fn notes_doc(score_content: &str) -> String {
    format!(
        r#"metadata {{ title "t", author "a" }},

parts ["S": notes],

score """
    {score_content}
    """,
"#
    )
}

/// Mirrors `combiner::tests::measure_source_span_is_nonzero_after_combine`:
/// `MeasureBlock.source_span` (populated in `compile_measure` from the
/// grouped `MultiPartMeasure`'s own `source_span`, which `combiner` already
/// computes as a real span into the source text) should carry that span
/// forward rather than defaulting to the dummy `(0, 0)`.
#[test]
fn measure_block_source_span_is_nonzero_after_compile() {
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1 2 3 4\n"));
    let result = compile(&score);
    assert_eq!(result.blocks.len(), 1);
    assert!(
        result.blocks[0].source_span.end > 0,
        "source_span.end should be > 0, got {:?}",
        result.blocks[0].source_span
    );
}

/// A run of consecutive all-rest measures collapses into one
/// `MultiMeasureRest` block (see `compiler::merge_rest_run`); its
/// `source_span` should still cover the whole run — the union (min start,
/// max end) of every merged measure's own span — not just the first
/// measure's, so a diagnostic anchored to the merged block can still point
/// at all of the source text it stands in for.
#[test]
fn merged_rest_run_source_span_covers_the_whole_run() {
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120\n[S] 0 0 0 0\n\n[S] 0 0 0 0\n\n[S] 1 2 3 4\n",
    ));
    let result = compile(&score);
    // The two rest measures collapse into one `MultiMeasureRest` block,
    // followed by the plain third measure.
    assert_eq!(result.blocks.len(), 2);
    let rest_block = &result.blocks[0];
    assert_eq!(rest_block.represents_measures, 2);
    let note_block = &result.blocks[1];
    assert!(
        rest_block.source_span.end <= note_block.source_span.start,
        "merged rest run's span {:?} should end before the following \
         measure's span {:?} starts",
        rest_block.source_span,
        note_block.source_span
    );
    assert!(
        rest_block.source_span.end > rest_block.source_span.start,
        "merged rest run's span should cover real source text, got {:?}",
        rest_block.source_span
    );
}
