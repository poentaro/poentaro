use super::{merged_block, no_header, simple_block};
use crate::compiler::types::MeasureBlock;
use crate::grid_layout::click_targets::{
    compute_all_bar_line_click_targets, compute_all_bar_number_click_targets,
};
use crate::grid_layout::layout::{block_column_width, LABEL_COLS, MUSIC_START_COL};
use std::collections::HashMap;

fn block_with_directive(col_count: u32, label: Option<&str>, bar_number: u32) -> MeasureBlock {
    let mut block = simple_block(col_count);
    block.decorations = vec![crate::compiler::types::Decoration::DirectiveLine {
        label: label.map(str::to_string),
        bar_number: Some(bar_number),
        key: None,
        bpm: None,
        time_signature: None,
    }];
    block
}

#[test]
fn bar_number_click_target_sits_in_the_directive_row_above_the_musical_rows() {
    // A block's bar number is drawn in its system's shared directive row
    // (see `make_decoration_row`), which sits above the musical rows
    // `MeasureClickTarget` covers — so its own click target needs its own
    // row, not `MeasureClickTarget::row_start`/`row_end`.
    //
    // `no_header()` has no title/subtitle/author, so `make_header_rows`
    // emits no rows and the directive line (bar number) is row 0.
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![block_with_directive(4, None, 1)]]];

    let targets = compute_all_bar_number_click_targets(
        &page_systems,
        &HashMap::new(),
        &no_header(),
        20.0,
        false,
    );

    let (_, target) = targets.first().expect("one bar-number click target");
    assert_eq!(target.row, 0, "should sit in the directive row");
    assert_eq!(
        target.column, LABEL_COLS,
        "the system's first block's bar number sits at its leading barline column"
    );
    assert_eq!(target.measure_index, 0);
    assert_eq!(target.measure_index_end, 0);
}

#[test]
fn bar_number_click_target_skips_a_later_block_with_no_directive_change() {
    // `make_decoration_row` only draws a later block's bar number when it
    // also changes a label/key/bpm/time-signature — an otherwise-plain
    // mid-system measure's bar number is never drawn, so it shouldn't get a
    // click target either.
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> = vec![vec![vec![
        block_with_directive(4, None, 1),
        block_with_directive(4, None, 2),
    ]]];

    let targets = compute_all_bar_number_click_targets(
        &page_systems,
        &HashMap::new(),
        &no_header(),
        20.0,
        false,
    );

    let measure_indices: Vec<usize> = targets.iter().map(|(_, t)| t.measure_index).collect();
    assert_eq!(
        measure_indices,
        vec![0],
        "only the system's first block draws a bar number here"
    );
}

#[test]
fn bar_number_click_target_includes_a_later_block_with_a_label() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> = vec![vec![vec![
        block_with_directive(4, None, 1),
        block_with_directive(4, Some("B"), 2),
    ]]];

    let targets = compute_all_bar_number_click_targets(
        &page_systems,
        &HashMap::new(),
        &no_header(),
        20.0,
        false,
    );

    let measure_indices: Vec<usize> = targets.iter().map(|(_, t)| t.measure_index).collect();
    assert_eq!(
        measure_indices,
        vec![0, 1],
        "the second block draws its own bar number since it also carries a label"
    );
    let (_, second) = &targets[1];
    assert_eq!(
        second.column,
        LABEL_COLS + block_column_width(&simple_block(4)),
        "the second block's bar number sits at its own leading barline column, \
         past the first block's width"
    );
}

#[test]
fn bar_line_click_targets_mark_leading_interior_and_closing_edges() {
    // Two measures in one system: 3 bar lines — leading (no `prev`),
    // interior (both), closing (no `next`).
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];

    let targets = compute_all_bar_line_click_targets(
        &page_systems,
        &HashMap::new(),
        &no_header(),
        20.0,
        false,
    );

    let pairs: Vec<(Option<usize>, Option<usize>)> = targets
        .iter()
        .map(|(_, t)| (t.measure_index_prev, t.measure_index_next))
        .collect();
    assert_eq!(
        pairs,
        vec![(None, Some(0)), (Some(0), Some(1)), (Some(1), None)],
        "leading bar line has no prev, closing bar line has no next"
    );
}

#[test]
fn bar_line_click_target_columns_line_up_with_adjacent_measures() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];

    let targets = compute_all_bar_line_click_targets(
        &page_systems,
        &HashMap::new(),
        &no_header(),
        20.0,
        false,
    );

    let block_width = block_column_width(&simple_block(4)) as f32;
    let columns: Vec<f32> = targets.iter().map(|(_, t)| t.column).collect();
    // Mirrors `measure_column_bounds`'s own boundary padding (see
    // `first_block_in_single_system_has_correct_column_range`/
    // `second_block_column_start_follows_first_block_width` in the parent
    // module): the leading bar line sits a full column left of
    // `MUSIC_START_COL`, the internal one between the two blocks half a
    // column left of the running total, and the closing one flush with it
    // — matching where each bar line's glyph is actually drawn, not the raw
    // unpadded accumulator.
    assert_eq!(
        columns,
        vec![
            MUSIC_START_COL as f32 - 1.0,
            MUSIC_START_COL as f32 + block_width - 0.5,
            MUSIC_START_COL as f32 + block_width * 2.0,
        ],
        "each bar line's column matches the padded boundary between blocks"
    );
}

#[test]
fn bar_line_click_target_is_adjacent_to_a_merged_multi_measure_rest_block() {
    // Blocks: simple(0), merged(1..=3), simple(4) — mirrors the
    // `merged_block_click_target_carries_full_represented_range` test above.
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> = vec![vec![vec![
        simple_block(4),
        merged_block(4, 3),
        simple_block(4),
    ]]];

    let targets = compute_all_bar_line_click_targets(
        &page_systems,
        &HashMap::new(),
        &no_header(),
        20.0,
        false,
    );

    let pairs: Vec<(Option<usize>, Option<usize>)> = targets
        .iter()
        .map(|(_, t)| (t.measure_index_prev, t.measure_index_next))
        .collect();
    assert_eq!(
        pairs,
        vec![
            (None, Some(0)),
            (Some(0), Some(1)),
            (Some(3), Some(4)),
            (Some(4), None),
        ],
        "the bar line after the merged block should carry its last represented \
         measure index (3), not its own block index (1)"
    );
}
