use crate::compositor::types::AbsoluteContent;
use crate::coordinate_resolver::resolve::{
    resolve, ElementPaddings, LabelFontSizes, LyricFontSizes, ResolveFontSizes,
};

/// Shared default padding used across this file's `ResolveFontSizes` literals, factored out to keep each test under clippy's line-count cap.
const DEFAULT_PADDINGS: ElementPaddings = ElementPaddings {
    notes: 4.0,
    chords: 4.0,
    lyrics: 4.0,
    note_dash: 4.0,
};
use crate::grid_layout::types::{GridContent, GridElement, GridPage, GridRow, HAlign, VAlign};

fn single_row_page(element: GridElement) -> GridPage {
    GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![GridRow {
            height_pt: 30.0,
            column_count: 10,
            has_label_region: false,
            measure_layout: vec![],
            elements: vec![element],
        }],
        measure_highlights: vec![],
        error_highlights: vec![],
        measure_click_targets: vec![],
        bar_number_click_targets: vec![],
        bar_line_click_targets: vec![],
        playback_cursor_targets: vec![],
        part_label_click_targets: vec![],
        lyric_click_targets: vec![],
        lyric_label_click_targets: vec![],
    }
}

#[test]
fn multi_measure_rest_resolves_width_from_column_span_inset_by_glyph_left_padding() {
    // usable = 595 - 50 = 545, col_width = 545/10 = 54.5
    // column=0, column_span=4 → x_start = 25.0, span_width = 4*54.5 = 218.0,
    // then inset by the configured notes padding on both ends (see
    // `resolve_multi_measure_rest`) so the drawn bar doesn't render flush
    // against the enclosing measure dividers.
    let el = GridElement {
        column: 0,
        column_span: 4,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::MultiMeasureRest { count: 5 },
    };
    let page = single_row_page(el);
    let abs = resolve(
        &[page],
        12.0,
        super::tests::TEST_PAGE_FRAME,
        ResolveFontSizes {
            lyric: LyricFontSizes {
                base: 14.4,
                cjk: 17.28,
            },
            notes: 12.0,
            chords: 12.0,
            labels: LabelFontSizes {
                measure_number: 10.0,
                section_label: 12.0,
                section_label_vertical_padding_pt: 0.0,
                part_label: 12.0,
                ..Default::default()
            },
            paddings: DEFAULT_PADDINGS,
            page_number_vertical_padding_pt: 0.0,
            ..Default::default()
        },
    )
    .unwrap();
    let rest = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::MultiMeasureRest { .. }))
        .expect("should have MultiMeasureRest");
    let col_width = (595.0 - 50.0) / 10.0; // 54.5
    let padding = 4.0;
    let x_start = 25.0 + padding;
    assert!(
        (rest.x - x_start).abs() < 0.01,
        "x={} expected={x_start}",
        rest.x
    );
    if let AbsoluteContent::MultiMeasureRest { count, width } = rest.content {
        assert_eq!(count, 5);
        let expected_width = col_width * 4.0 - padding * 2.0;
        assert!(
            (width - expected_width).abs() < 0.01,
            "width={width} expected={expected_width}"
        );
    } else {
        panic!("expected MultiMeasureRest content");
    }
}
