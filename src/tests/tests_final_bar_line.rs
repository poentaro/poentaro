use super::*;
use crate::compiler::types::BarLineKind;
use crate::grid_layout::types::{GridContent, GridPage};

/// Lays `input` out on pages of `page_height_pt` and returns every bar line's
/// kind in page/row/element order.
fn bar_line_kinds_by_page(input: &str, page_height_pt: f32) -> Vec<Vec<BarLineKind>> {
    let score = compile(input, "test", &[]).unwrap();
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let compile_result = consolidator::consolidate(compiler::compile(&score));
    let pages: Vec<GridPage> = grid_layout::layout(
        &compile_result,
        &config,
        &grid_layout::types::Header {
            parts_list_columns: 3,
            ..Default::default()
        },
        595.0,
        page_height_pt,
        None,
    )
    .pages;
    pages
        .iter()
        .map(|page| {
            page.rows
                .iter()
                .flat_map(|row| &row.elements)
                .filter_map(|e| match e.content {
                    GridContent::BarLine { kind, .. } => Some(kind),
                    _ => None,
                })
                .collect()
        })
        .collect()
}

const MULTI_SYSTEM_INPUT: &str = r#"metadata { max measures per system 2 },

parts ["a": notes],

score """
    [a] 1 2 3 4

    [a] 5 6 7 1

    [a] 1 2 3 4

    [a] 5 6 7 1

    [a] 1 2 3 4
    """,
"#;

#[test]
fn only_the_score_closing_bar_line_is_final() {
    let kinds: Vec<BarLineKind> = bar_line_kinds_by_page(MULTI_SYSTEM_INPUT, 842.0)
        .into_iter()
        .flatten()
        .collect();
    let final_count = kinds.iter().filter(|k| **k == BarLineKind::Final).count();
    assert_eq!(
        final_count, 1,
        "expected exactly one final bar line: {kinds:?}"
    );
    assert_eq!(kinds.last(), Some(&BarLineKind::Final));
}

#[test]
fn final_bar_line_is_only_on_the_last_page() {
    let pages = bar_line_kinds_by_page(MULTI_SYSTEM_INPUT, 200.0);
    assert!(
        pages.len() >= 2,
        "fixture should span several pages, got {}",
        pages.len()
    );
    let (last, earlier) = pages.split_last().unwrap();
    assert!(
        earlier.iter().flatten().all(|k| *k == BarLineKind::Single),
        "no page before the last should have a final bar line: {earlier:?}"
    );
    assert_eq!(last.last(), Some(&BarLineKind::Final));
}
