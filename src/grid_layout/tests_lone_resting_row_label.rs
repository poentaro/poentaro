//! Verifies that a system's part-abbreviation label gets cleared when the
//! system boils down to a single row and that row is entirely rest — see
//! `layout_systems::clear_label_if_lone_resting_row`. A label only earns its
//! keep by distinguishing one row from another; a lone all-rest row has
//! nothing else in its system to distinguish itself from.

use crate::compiler::compile;
use crate::consolidator::consolidate;
use crate::grid_layout::layout::layout;
use crate::grid_layout::types::{GridContent, Header};
use crate::grouper::group;
use crate::parser::parse;
use crate::render_config::RenderConfig;

fn header() -> Header {
    Header {
        title: None,
        subtitle: None,
        author: None,
        part_list: vec![],
        parts_list_columns: 3,
        sequence: None,
        title_font_size: 45.0,
        subtitle_font_size: 24.0,
        author_font_size: 18.0,
        sequence_font_size: 12.0,
        part_legend_font_size: 12.0,
        ..Default::default()
    }
}

/// Every `RowLabel` text in the laid-out score, in row order (top to bottom,
/// `[parts]` declaration order — see `union_row_order`).
fn row_labels(source: &str) -> Vec<String> {
    let document = parse(source, "test", &[]).unwrap();
    let score = group(document).unwrap();
    let config = RenderConfig::from_metadata(&score.metadata);
    let compile_result = consolidate(compile(&score));
    let output = layout(&compile_result, &config, &header(), 2000.0, 4000.0, None);
    output
        .pages
        .iter()
        .flat_map(|page| page.rows.iter())
        .flat_map(|row| row.elements.iter())
        .filter_map(|el| match &el.content {
            GridContent::RowLabel(text) => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn lone_part_single_resting_measure_has_no_label() {
    // Only "S1" exists in this score, and it rests for its one measure. That
    // single measure doesn't meet `MIN_REST_RUN_LENGTH`, so it stays a plain
    // `Rest`, not a collapsed `MultiMeasureRest` — but the rule still applies:
    // one row, entirely rest, nothing to distinguish it from.
    let source = r#"parts ["Soprano 1" [S1]: notes],

score """
    [S1] 0 0 0 0
    """,
"#;
    assert_eq!(row_labels(source), Vec::<String>::new());
}

#[test]
fn lone_part_collapsed_multi_measure_rest_has_no_label() {
    // Only "S1" exists, and it rests for 4 consecutive measures with no
    // directive in between, so those measures collapse into a single
    // `MultiMeasureRest` block. Still just one row in the system, still
    // entirely rest, so the label is cleared.
    let source = r#"parts ["Soprano 1" [S1]: notes],

score """
    [S1] 0 0 0 0
    [S1] 0 0 0 0
    [S1] 0 0 0 0
    [S1] 0 0 0 0
    """,
"#;
    assert_eq!(row_labels(source), Vec::<String>::new());
}

#[test]
fn second_part_with_real_content_keeps_the_resting_part_labeled() {
    // "S1" rests for 4 consecutive measures (collapsing to a
    // `MultiMeasureRest`) while "A1" plays actual notes throughout — with
    // `hide_resting_parts=no` so S1's own row stays in the system instead of
    // being hidden entirely by the (default-on) resting-part hiding. Now two
    // rows share the system, so S1's label is still needed to tell the rows
    // apart, even though S1's own row is entirely rest.
    let source = r#"metadata { hide resting parts: no },

parts ["Soprano 1" [S1]: notes, "Alto 1" [A1]: notes],

score """
    [S1] 0 0 0 0
    [A1] 1 2 3 4

    [S1] 0 0 0 0
    [A1] 1 2 3 4

    [S1] 0 0 0 0
    [A1] 1 2 3 4

    [S1] 0 0 0 0
    [A1] 1 2 3 4
    """,
"#;
    assert_eq!(row_labels(source), vec!["S1".to_string(), "A1".to_string()]);
}
