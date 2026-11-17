use crate::compositor::types::{AbsoluteContent, FontFamily};
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

/// Shared default label font sizes used across this file's `ResolveFontSizes` literals, factored out to keep each test under clippy's line-count cap.
const DEFAULT_LABELS: LabelFontSizes = LabelFontSizes {
    measure_number: 10.0,
    section_label: 12.0,
    section_label_vertical_padding_pt: 0.0,
    part_label: 12.0,
    measure_number_bold: false,
    measure_number_italic: false,
    measure_number_underline: false,
    measure_number_font_family: FontFamily::SansSerif,
    section_label_bold: false,
    section_label_italic: false,
    section_label_underline: false,
    section_label_font_family: FontFamily::SansSerif,
    part_label_bold: false,
    part_label_italic: false,
    part_label_underline: false,
    part_label_font_family: FontFamily::SansSerif,
    sequence_bold: false,
    sequence_italic: false,
    sequence_underline: false,
    sequence_font_family: FontFamily::SansSerif,
};
use crate::grid_layout::types::{BarNumberClickTarget, GridPage, GridRow};

#[test]
fn bar_number_click_target_resolves_to_a_small_rect_sized_to_its_digits() {
    // A one-row page, no header/decoration rows involved — `row: 0` is
    // exactly where a bar number would sit if this were a real directive
    // row (see `compute_all_bar_number_click_targets`).
    let page = GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![GridRow {
            height_pt: 18.0,
            column_count: 10,
            has_label_region: false,
            measure_layout: vec![],
            elements: vec![],
        }],
        measure_highlights: vec![],
        error_highlights: vec![],
        measure_click_targets: vec![],
        bar_number_click_targets: vec![BarNumberClickTarget {
            row: 0,
            column: 2,
            measure_index: 41,
            measure_index_end: 41,
        }],
        bar_line_click_targets: vec![],
        playback_cursor_targets: vec![],
        part_label_click_targets: vec![],
        lyric_click_targets: vec![],
        lyric_label_click_targets: vec![],
    };
    let abs_pages = resolve(
        &[page],
        8.0,
        super::tests::TEST_PAGE_FRAME,
        ResolveFontSizes {
            lyric: LyricFontSizes {
                base: 14.4,
                cjk: 17.28,
            },
            notes: 12.0,
            chords: 12.0,
            labels: DEFAULT_LABELS,
            paddings: DEFAULT_PADDINGS,
            page_number_vertical_padding_pt: 0.0,
            ..Default::default()
        },
    )
    .unwrap();

    let target = abs_pages[0]
        .elements
        .iter()
        .find_map(|e| match &e.content {
            AbsoluteContent::BarNumberClickTarget {
                width,
                height,
                measure_index,
                measure_index_end,
            } => Some((
                e.x,
                e.y,
                *width,
                *height,
                *measure_index,
                *measure_index_end,
            )),
            _ => None,
        })
        .expect("expected a BarNumberClickTarget element");
    let (x, y, width, height, measure_index, measure_index_end) = target;

    // Measure 41's displayed bar number is 42 (`measure_index + 1` — see
    // `compiler::compile`), two digits wide, so the click target should be
    // noticeably narrower than the whole row, not the row's full width.
    assert!(
        width > 0.0 && width < 40.0,
        "width ({width}) should be a small, digit-sized box, not the row's full width"
    );
    assert!(
        (height - 18.0).abs() < 0.01,
        "height should be the row's own height, got {height}"
    );
    assert!(
        x > 0.0,
        "x should be positive (past the page margin), got {x}"
    );
    assert!(y >= 0.0, "y should be non-negative, got {y}");
    assert_eq!(measure_index, 41);
    assert_eq!(measure_index_end, 41);
}
