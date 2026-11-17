//! Compiler-level tests confirming the beam/underline grid math in `part_slice.rs`/
//! `part_slice_unit.rs` — quarter-beat-boundary flushing and total bar-line column —
//! still comes out correct once a measure has been tuplet-rescaled
//! (`GroupedMeasure::resolution_multiplier` > 1, threaded onto `PartSlice` and consumed
//! here as `PartState::multiplier`). See `src/grouper/tests_tuplets.rs` for the
//! equivalent grouper-level test this reuses the exact same input from, and
//! `src/grouping_tuplet_tests.rs` for the half-bar-boundary/dotted-eighth-tail rules
//! (a separate, parser-facing module) exercised directly under a tuplet multiplier.

use crate::compiler::{compile, types::*};
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

/// `3:{1_1_1_} 2_ 3_ 4_ 5_ 6_ 7_` — the same eighth-note-triplet-plus-six-plain-eighths
/// measure as `measure_with_a_triplet_groups_with_correctly_rescaled_durations` in
/// `src/grouper/tests_tuplets.rs` (chosen there, and reused here, because the triplet
/// rescales to exactly one beat and the 6 plain eighth notes exactly fill the remaining
/// 3 beats — see that test's doc comment). `resolution_multiplier = lcm(3) = 3`; the 3
/// triplet notes rescale to duration 4 each, the 6 plain eighth notes rescale to
/// duration 6 each (`2 * 3`).
///
/// A quarter-beat is therefore `4 * 3 = 12` rescaled columns, not the base-scale `4` —
/// `compile_unit`'s beam-flush check (`beat_position % (4 * multiplier) == 0`) must use
/// the scaled value or the plain eighth notes here would never flush into underlines at
/// all (their raw duration of `6` never happens to be a multiple of the unscaled `4`).
#[test]
fn tuplet_measure_flushes_underlines_at_multiplier_scaled_quarter_beat_boundaries() {
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120\n[S] 3:{1_1_1_} 2_ 3_ 4_ 5_ 6_ 7_\n",
    ));
    let result = compile(&score);
    let row = &result.blocks[0].rows[0];

    let underlines: Vec<(u32, u32)> = row
        .elements
        .iter()
        .filter_map(|e| match &e.content {
            ElementContent::Underline {
                from_column,
                to_column,
                ..
            } => Some((*from_column, *to_column)),
            _ => None,
        })
        .collect();

    // The 3 triplet notes (rescaled duration 4 each) have their compression undone
    // before the underline-count check (`compile_unit` divides by `den` and multiplies
    // by `num`), recovering their written eighth-note duration scaled by the multiplier
    // (2 * 3 = 6 = 2*multiplier) — so, like plain eighth notes, they get
    // underline_count=1 and flush together into one run spanning the whole triplet
    // (column 0 to 12). The 6 plain eighth notes (rescaled duration 6 = 2*multiplier)
    // each also get underline_count=1 and flush in pairs at every scaled quarter-beat
    // boundary (column 24, 36, 48), with no leftover unpaired note.
    assert_eq!(
        underlines,
        vec![(0, 12), (12, 24), (24, 36), (36, 48)],
        "the eighth-note triplet should flush into its own underline run, followed by \
         the plain eighth notes' 3 underline runs at the multiplier-scaled \
         quarter-beat boundaries"
    );

    let bar_line = row
        .elements
        .iter()
        .find(|e| matches!(e.content, ElementContent::BarLine))
        .unwrap();
    assert_eq!(
        bar_line.column, 48,
        "bar line column should equal the rescaled total duration (3*4 + 6*6 = 48)"
    );
}

/// `3:{3 6 1} 3:{3 6 1}` — two directly-adjacent triplets with the same ratio and no
/// untagged event between them. Each `{...}` bracket should still produce its own
/// `TupletSpan`/bracket, even though nothing but the bracket boundary itself
/// distinguishes them.
#[test]
fn two_adjacent_same_ratio_tuplets_produce_two_spans() {
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120\n[S] 3:{3 6 1}  3:{3 6 1}\n",
    ));
    let result = compile(&score);

    assert_eq!(
        result.tuplet_spans.len(),
        2,
        "two adjacent same-ratio tuplet brackets should not merge into one span, got {:?}",
        result.tuplet_spans
    );
}
