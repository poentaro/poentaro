use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::grid_layout::click_targets::compute_all_measure_click_targets;
use crate::grid_layout::highlight::compute_measure_highlights_for_range;
use crate::grid_layout::layout::compute_measure_highlight_location;
use crate::grid_layout::types::{Header, MeasureRange};
use std::collections::HashMap;

fn simple_block(col_count: u32) -> MeasureBlock {
    let elements: Vec<ColumnElement> = (0..col_count)
        .map(|c| ColumnElement {
            column: c,
            content: ElementContent::NoteHead {
                pitch: crate::ast::parsed::NumberedPitch::One,
                accidental: crate::ast::parsed::Accidental::Natural,
                octave: 0,
                dotted: false,
                double_dotted: false,
                head_label: None,
            },
            note_id: None,
        })
        .chain(std::iter::once(ColumnElement {
            column: col_count,
            content: ElementContent::BarLine,
            note_id: None,
        }))
        .collect();
    MeasureBlock {
        rows: vec![MeasureRow {
            kind: crate::ast::parsed::PartKind::Notes,
            absorbed_rows: Vec::new(),
            id: RowId("S".to_string()),
            label: String::new(),
            elements,
            source_part_index: 0,
        }],
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
        system_break: false,
        source_span: crate::error::Span::new(0, 0),
    }
}

fn merged_block(col_count: u32, represents_measures: usize) -> MeasureBlock {
    let mut block = simple_block(col_count);
    block.represents_measures = represents_measures;
    block
}

fn no_header() -> Header {
    Header {
        title: None,
        subtitle: None,
        author: None,
        part_list: vec![],
        parts_list_columns: 3,
        sequence: None,
        title_font_size: 36.0,
        subtitle_font_size: 19.0,
        author_font_size: 14.0,
        sequence_font_size: 12.0,
        part_legend_font_size: 12.0,
        ..Default::default()
    }
}

#[test]
fn returns_none_for_out_of_range_measure_index() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let result = compute_measure_highlight_location(
        &page_systems,
        &HashMap::new(),
        2,
        &no_header(),
        20.0,
        false,
    );
    assert!(result.is_none());
}

#[test]
fn first_block_in_single_system_has_correct_column_range() {
    // LABEL_COLS = 1, MUSIC_START_COL = 2 (the leading bar line gets its own
    // dedicated column at LABEL_COLS), block_column_width(4-note block) = 5
    // (4 notes + 1 bar line). The system-leading bar line is flush against
    // the start of its column (HAlign::Start), so column_start = 2 - 1 = 1.0.
    // This isn't the system's last block, so its ending bar line is still
    // centered in its own column: column_end = (2 + 5) - 0.5 = 6.5.
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let result = compute_measure_highlight_location(
        &page_systems,
        &HashMap::new(),
        0,
        &no_header(),
        20.0,
        false,
    )
    .expect("should find measure 0");
    let (_, highlight) = result;
    assert_eq!(
        highlight.column_start, 1.0,
        "column_start should match the start-aligned position of the leading bar line"
    );
    assert_eq!(
        highlight.column_end, 6.5,
        "column_end should match the centered position of the ending bar line"
    );
}

#[test]
fn second_block_column_start_follows_first_block_width() {
    // measure 1's left edge is the previous bar line, centered at column 7, i.e. 6.5.
    // It's the system's last block, so its own ending bar line is flush against the
    // end of its column (HAlign::End): column_end = 7 + 5 = 12.0.
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let result = compute_measure_highlight_location(
        &page_systems,
        &HashMap::new(),
        1,
        &no_header(),
        20.0,
        false,
    )
    .expect("should find measure 1");
    let (_, highlight) = result;
    assert_eq!(highlight.column_start, 6.5);
    assert_eq!(highlight.column_end, 12.0);
}

#[test]
fn measure_on_second_page_returns_correct_page_index() {
    // page 0: system with measure 0; page 1: system with measure 1
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4)]], vec![vec![simple_block(4)]]];
    let result = compute_measure_highlight_location(
        &page_systems,
        &HashMap::new(),
        1,
        &no_header(),
        20.0,
        false,
    )
    .expect("should find measure 1");
    let (page_idx, _) = result;
    assert_eq!(page_idx, 1, "measure 1 is on page 1");
}

#[test]
fn range_with_single_index_returns_one_highlight_matching_location() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let highlights = compute_measure_highlights_for_range(
        &page_systems,
        &HashMap::new(),
        &[MeasureRange { start: 0, end: 0 }],
        &no_header(),
        20.0,
        false,
    );
    assert_eq!(highlights.len(), 1);
    let (page_idx, h) = highlights
        .into_iter()
        .next()
        .expect("should have one highlight");
    assert_eq!(page_idx, 0);
    assert_eq!(h.column_start, 1.0);
    assert_eq!(h.column_end, 6.5);
}

#[test]
fn range_spanning_two_measures_returns_two_highlights() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let highlights = compute_measure_highlights_for_range(
        &page_systems,
        &HashMap::new(),
        &[MeasureRange { start: 0, end: 1 }],
        &no_header(),
        20.0,
        false,
    );
    assert_eq!(highlights.len(), 2);
    let mut iter = highlights.into_iter();
    let (_, first_h) = iter.next().expect("first highlight");
    let (_, second_h) = iter.next().expect("second highlight");
    assert_eq!(first_h.column_start, 1.0);
    assert_eq!(second_h.column_start, 6.5);
}

#[test]
fn range_out_of_bounds_returns_empty_vec() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let highlights = compute_measure_highlights_for_range(
        &page_systems,
        &HashMap::new(),
        &[MeasureRange { start: 5, end: 5 }],
        &no_header(),
        20.0,
        false,
    );
    assert!(highlights.is_empty());
}

#[test]
fn range_spanning_two_pages_reports_correct_page_indices() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4)]], vec![vec![simple_block(4)]]];
    let highlights = compute_measure_highlights_for_range(
        &page_systems,
        &HashMap::new(),
        &[MeasureRange { start: 0, end: 1 }],
        &no_header(),
        20.0,
        false,
    );
    assert_eq!(highlights.len(), 2);
    let mut iter = highlights.into_iter();
    let (first_page, _) = iter.next().expect("first highlight");
    let (second_page, _) = iter.next().expect("second highlight");
    assert_eq!(first_page, 0);
    assert_eq!(second_page, 1);
}

#[test]
fn global_measure_index_accounts_for_a_merged_block() {
    // 5 source measures: measure 0 is a normal block, measures 1-3 are
    // collapsed into one merged block (represents_measures = 3), measure 4
    // is a normal block again. Its global_measure_index must be 4, not 2
    // (which is what a naive "+1 per block" count would produce, since
    // there are only 3 blocks in this system).
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> = vec![vec![vec![
        simple_block(4),
        merged_block(4, 3),
        simple_block(4),
    ]]];
    let result = compute_measure_highlight_location(
        &page_systems,
        &HashMap::new(),
        4,
        &no_header(),
        20.0,
        false,
    );
    assert!(
        result.is_some(),
        "measure index 4 should resolve to the block after the merged run"
    );

    let targets = compute_all_measure_click_targets(
        &page_systems,
        &HashMap::new(),
        &no_header(),
        20.0,
        false,
    );
    let measure_indices: Vec<usize> = targets.iter().map(|(_, t)| t.measure_index).collect();
    assert_eq!(
        measure_indices,
        vec![0, 1, 4],
        "click targets should carry global_measure_index 0, 1 (merged block's start), 4 (not 2)"
    );
    let measure_index_ends: Vec<usize> = targets.iter().map(|(_, t)| t.measure_index_end).collect();
    assert_eq!(
        measure_index_ends,
        vec![0, 3, 4],
        "the merged block's click target should span its whole represented range (1..=3)"
    );
}

#[cfg(test)]
#[path = "tests_highlight_pages.rs"]
mod tests_highlight_pages;

#[cfg(test)]
#[path = "tests_highlight_disjoint_ranges.rs"]
mod tests_highlight_disjoint_ranges;

#[cfg(test)]
#[path = "tests_highlight_click_targets.rs"]
mod tests_highlight_click_targets;
