use crate::compiler::{compile, types::*};
use crate::grouper::group;
use crate::parser::parse;

fn score_from(source: &str) -> crate::ast::grouped::Score {
    let doc = parse(source, "test", &[]).unwrap();
    group(doc).unwrap()
}

/// Minimal one-part (notes) document. `score_content` is everything after `score\n`.
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

#[test]
fn three_consecutive_rest_measures_collapse_into_one_multi_measure_rest_block() {
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120\n[S] 1\n\n[S] 0 0 0 0\n\n[S] 0 0 0 0\n\n[S] 0 0 0 0\n",
    ));
    let result = compile(&score);
    let blocks = result.blocks;
    assert_eq!(
        blocks.len(),
        2,
        "one normal block for measure 1, one merged block for the 3-measure rest run"
    );
    let merged = &blocks[1];
    assert_eq!(merged.represents_measures, 3);
    let row = &merged.rows[0];
    let multi_rest: Vec<_> = row
        .elements
        .iter()
        .filter(|e| matches!(e.content, ElementContent::MultiMeasureRest { count: 3 }))
        .collect();
    assert_eq!(
        multi_rest.len(),
        1,
        "merged block should have exactly one MultiMeasureRest element with count=3"
    );
}

#[test]
fn lone_rest_measure_run_of_length_one_stays_uncollapsed() {
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120\n[S] 1\n\n[S] 0 0 0 0\n\n[S] 2\n",
    ));
    let result = compile(&score);
    let blocks = result.blocks;
    assert_eq!(blocks.len(), 3, "no collapsing should occur");
    for block in &blocks {
        assert_eq!(block.represents_measures, 1);
    }
    let rest_row = &blocks[1].rows[0];
    assert!(
        rest_row
            .elements
            .iter()
            .any(|e| matches!(e.content, ElementContent::Rest { .. })),
        "lone rest measure should keep its individual Rest elements"
    );
}

#[test]
fn time_signature_change_interrupts_a_rest_run() {
    // Measures 2-3 are all-rest in 4/4; measure 4 changes time signature to 3/4
    // (also all-rest) but must not be absorbed into the run; measures 5-6 (3/4
    // rest) form their own separate run.
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120\n[S] 1\n\n[S] 0 0 0 0\n\n[S] 0 0 0 0\n\ntime=3/4\n[S] 0 0 0\n\n[S] 0 0 0\n\n[S] 0 0 0\n",
    ));
    let result = compile(&score);
    let blocks = result.blocks;
    // block0: measure 1 (note); block1: merged measures 2-3 (4/4 rest run);
    // block2: measure 4 (time-sig change, uncollapsed); block3: merged measures 5-6 (3/4 rest run)
    assert_eq!(blocks.len(), 4, "blocks={blocks:#?}");
    assert_eq!(blocks[0].represents_measures, 1);
    assert_eq!(blocks[1].represents_measures, 2);
    assert_eq!(blocks[2].represents_measures, 1);
    assert!(
        blocks[2].decorations.iter().any(|d| matches!(
            d,
            Decoration::DirectiveLine {
                time_signature: Some((3, 4)),
                ..
            }
        )),
        "the time-signature-change measure must stay uncollapsed and keep its decoration"
    );
    assert_eq!(blocks[3].represents_measures, 2);
}

#[test]
fn redundant_time_signature_directive_does_not_interrupt_a_rest_run() {
    // Measure 1 implicitly starts in 4/4 (the default); measure 3 restates
    // `time=4/4` explicitly even though nothing actually changed. Since it's
    // not a real change, it must not break the rest run spanning measures
    // 2-4.
    let score = score_from(&notes_doc(
        "key=C4 bpm=120\n[S] 1\n\n[S] 0 0 0 0\n\ntime=4/4\n[S] 0 0 0 0\n\n[S] 0 0 0 0\n",
    ));
    let result = compile(&score);
    let blocks = result.blocks;
    assert_eq!(blocks.len(), 2, "blocks={blocks:#?}");
    assert_eq!(blocks[0].represents_measures, 1, "measure 1 has a note");
    assert_eq!(
        blocks[1].represents_measures, 3,
        "the redundant time=4/4 on measure 3 must not split the rest run"
    );
}

#[test]
fn labeled_measure_starts_a_new_rest_run_instead_of_joining_the_previous_one() {
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120\n[S] 1\n\n[S] 0 0 0 0\n\n[S] 0 0 0 0\n\nlabel=\"Verse 1\"\n[S] 0 0 0 0\n\n[S] 0 0 0 0\n",
    ));
    let result = compile(&score);
    let blocks = result.blocks;
    assert_eq!(blocks.len(), 3, "blocks={blocks:#?}");
    assert_eq!(blocks[0].represents_measures, 1, "measure 1 has a note");
    assert_eq!(
        blocks[1].represents_measures, 2,
        "measures 2-3 (rest) merge before the labeled measure"
    );
    assert!(
        !blocks[1]
            .decorations
            .iter()
            .any(|d| matches!(d, Decoration::DirectiveLine { label: Some(_), .. })),
        "the pre-label run carries no label"
    );
    assert_eq!(
        blocks[2].represents_measures, 2,
        "the label breaks off a new run rather than joining the previous one, but still \
         merges forward with the trailing rest measure"
    );
    assert!(
        blocks[2].decorations.iter().any(|d| matches!(
            d,
            Decoration::DirectiveLine { label: Some(l), .. } if l == "Verse 1"
        )),
        "the label must remain visible on the merged block it heads, blocks={blocks:#?}"
    );
}

#[test]
fn hide_resting_parts_change_interrupts_a_rest_run() {
    // Measures 2-3 are all-rest; measure 4 changes hide_resting_parts but is
    // also all-rest, so it must not be absorbed into the preceding run.
    // Measures 4-5 (both under the new setting) then form their own run.
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120\n[S] 1\n\n[S] 0 0 0 0\n\n[S] 0 0 0 0\n\nhide_resting_parts=no\n[S] 0 0 0 0\n\n[S] 0 0 0 0\n",
    ));
    let result = compile(&score);
    let blocks = result.blocks;
    assert_eq!(blocks.len(), 3, "blocks={blocks:#?}");
    assert_eq!(blocks[0].represents_measures, 1);
    assert_eq!(
        blocks[1].represents_measures, 2,
        "measures 2-3 merge before the setting change"
    );
    assert_eq!(
        blocks[2].represents_measures, 2,
        "measures 4-5 merge separately under the new setting"
    );
}

#[test]
fn merge_duplicate_measures_across_parts_change_interrupts_a_rest_run() {
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120\n[S] 1\n\n[S] 0 0 0 0\n\n[S] 0 0 0 0\n\nmerge_duplicate_measures_across_parts=no\n[S] 0 0 0 0\n\n[S] 0 0 0 0\n",
    ));
    let result = compile(&score);
    let blocks = result.blocks;
    assert_eq!(blocks.len(), 3, "blocks={blocks:#?}");
    assert_eq!(blocks[0].represents_measures, 1);
    assert_eq!(
        blocks[1].represents_measures, 2,
        "measures 2-3 merge before the setting change"
    );
    assert_eq!(
        blocks[2].represents_measures, 2,
        "measures 4-5 merge separately under the new setting"
    );
}

#[test]
fn hiding_a_track_lets_plain_rest_measures_from_hidden_track_collapse() {
    let source = r#"metadata { title "t", author "a" },

parts ["m": chords, "n": notes],

score """
    [n] 1

    [n] 1
    """,
"#;
    let mut score = {
        let doc = parse(source, "test", &[]).unwrap();
        group(doc).unwrap()
    };
    crate::filters::filter_tracks(&mut score, &["m".to_string()]);
    let result = compile(&score);
    let blocks = result.blocks;
    assert_eq!(
        blocks.len(),
        1,
        "the two now-all-rest `m` measures should collapse into a single block, blocks={blocks:#?}"
    );
    assert_eq!(blocks[0].represents_measures, 2);
}

#[test]
fn soloing_a_part_with_a_labeled_first_measure_still_collapses_its_rest_run() {
    // Mirrors a real score: [a1] has no notes of its own in either measure, so
    // soloing it (filtering everything else out) turns both measures into
    // plain rests for [a1]. The label on measure 1 is a section marker for the
    // score as a whole, not a reason [a1]'s own rest run shouldn't collapse.
    let source = r#"metadata { title "t", author "a" },

parts ["Alto 1" [a1]: notes, "Snare" [sn]: percussion],

score """
    label="Intro"
    [sn]x

    [sn] x
    """,
"#;
    let mut score = {
        let doc = parse(source, "test", &[]).unwrap();
        group(doc).unwrap()
    };
    crate::filters::filter_tracks(&mut score, &["a1".to_string()]);
    let result = compile(&score);
    let blocks = result.blocks;
    assert_eq!(
        blocks.len(),
        1,
        "soloing [a1] leaves two all-rest measures for a1; despite the label on \
         measure 1, they should still merge into a single multi-measure rest block, \
         blocks={blocks:#?}"
    );
    assert_eq!(blocks[0].represents_measures, 2);
}

#[test]
fn slur_span_measure_indices_are_remapped_after_a_hidden_track_causes_merging() {
    // Mirrors `hiding_part_a_still_renders_tie_arc_on_part_b_across_merged_rest_measures`
    // in `tests_render_filtering`: [a] has plain notes in measures 1-2 and
    // measure 3; [b] only has content in measure 3, where a tie chain
    // (`6m __~_6m`) connects two chord symbols. Soloing [b] turns measures
    // 1-2 into all-rest for [b], collapsing them into one block, so the tie's
    // `SlurSpan` (originally recorded against raw measure index 2) must be
    // remapped to block index 1, not left pointing at the stale pre-merge
    // measure index.
    let source = r#"metadata { title "t", author "a" },

parts ["a": notes, "b": chords],

score """
    [a] 1

    [a]1

    [a]1
    [b] 6m __~_6m__0_
    """,
"#;
    let mut score = {
        let doc = parse(source, "test", &[]).unwrap();
        group(doc).unwrap()
    };
    crate::filters::filter_tracks(&mut score, &["b".to_string()]);
    let result = compile(&score);
    let blocks = &result.blocks;
    assert_eq!(
        blocks.len(),
        2,
        "measures 1-2 (all-rest for soloed b) merge into one block, measure 3 stays \
         standalone, blocks={blocks:#?}"
    );
    assert_eq!(blocks[0].represents_measures, 2);
    assert_eq!(blocks[1].represents_measures, 1);
    assert_eq!(
        result.slur_spans.len(),
        1,
        "expected exactly one tie span, got: {:?}",
        result.slur_spans
    );
    let span = &result.slur_spans[0];
    assert_eq!(
        span.from_measure, 1,
        "tie's opening note lives in block 1 (the standalone measure 3 block), not the \
         stale pre-merge measure index, span={span:?}"
    );
    assert_eq!(
        span.to_measure, 1,
        "tie's closing note also lives in block 1, span={span:?}"
    );
}

#[test]
fn diagnostic_bearing_measure_interrupts_a_rest_run() {
    // An error inside a measure means the source has a real problem there;
    // a lone rest measure sandwiched between two 2-measure rest runs by
    // itself would need length >= 2 to collapse, so keep this simple: a
    // rest run followed directly by a note measure that has a diagnostic
    // due to malformed input should not be folded into a following rest run.
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120\n[S] 0 0 0 0\n\n[S] 0 0 0 0\n\n[S] 1 1 1 1 1\n\n[S] 0 0 0 0\n\n[S] 0 0 0 0\n",
    ));
    let result = compile(&score);
    let blocks = result.blocks;
    let middle = blocks
        .iter()
        .find(|b| !b.diagnostics.is_empty())
        .expect("overflowing measure should carry a diagnostic");
    assert_eq!(
        middle.represents_measures, 1,
        "a measure with diagnostics must never be folded into a rest run"
    );
}
