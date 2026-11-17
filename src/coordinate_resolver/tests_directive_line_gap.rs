use crate::compositor::types::AbsoluteContent;
use crate::coordinate_resolver::resolve::{resolve, LabelFontSizes, ResolveFontSizes};
use crate::font_metrics::{span_width, DIRECTIVE_LINE_ELEMENT_GAP};
use crate::grid_layout::types::{GridContent, GridElement, GridPage, GridRow, HAlign, VAlign};

fn resolve_directive_line(bar_number: Option<u32>, key: Option<&str>) -> AbsoluteContent {
    let page = GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![GridRow {
            height_pt: 30.0,
            column_count: 10,
            has_label_region: false,
            measure_layout: vec![],
            elements: vec![GridElement {
                column: 0,
                column_span: 1,
                halign: HAlign::Start,
                valign: VAlign::Center,
                content: GridContent::DirectiveLine {
                    label: None,
                    bar_number,
                    key: key.map(str::to_string),
                    bpm: None,
                    time_signature: None,
                },
            }],
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
    };
    let font_sizes = ResolveFontSizes {
        labels: LabelFontSizes {
            measure_number: 10.0,
            section_label: 12.0,
            ..Default::default()
        },
        ..Default::default()
    };
    resolve(&[page], 12.0, super::tests::TEST_PAGE_FRAME, font_sizes).unwrap()[0].elements[0]
        .content
        .clone()
}

/// Without a section label, the key directive must start a measured gap
/// past the bar number — the spans' leading spaces are dropped by SVG, so
/// bar `3` then `1=D` would otherwise read as `31=D` (issue #68).
#[test]
fn labelless_directives_start_a_gap_past_the_bar_number() {
    let AbsoluteContent::DirectiveLine {
        bar_number,
        spans_x_offset,
        ..
    } = resolve_directive_line(Some(3), Some("1=D"))
    else {
        panic!("expected a DirectiveLine");
    };
    let bar_number_width = span_width(&bar_number.expect("expected a bar number"));
    assert_eq!(
        spans_x_offset,
        bar_number_width + DIRECTIVE_LINE_ELEMENT_GAP
    );
}

#[test]
fn labelless_directives_without_a_bar_number_start_at_the_line_start() {
    let AbsoluteContent::DirectiveLine { spans_x_offset, .. } =
        resolve_directive_line(None, Some("1=D"))
    else {
        panic!("expected a DirectiveLine");
    };
    assert_eq!(spans_x_offset, 0.0);
}
