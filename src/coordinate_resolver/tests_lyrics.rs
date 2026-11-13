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
use crate::grid_layout::types::{
    GridContent, GridElement, GridPage, GridRow, HAlign, MeasureColumnLayout, VAlign,
};

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
fn lyric_syllable_halign_center_is_independent_of_column_weight() {
    // Like `note_head_halign_center_is_independent_of_column_weight` (tests.rs)
    // but for a lyric syllable: under flush-left anchoring, its anchor must land
    // at the same offset from the column's left edge regardless of how much
    // unrelated weight (e.g. a wide chord symbol) inflates the column's width.
    let make_page = |column_weight: f32| -> GridPage {
        let el = GridElement {
            column: 2,
            column_span: 1,
            halign: HAlign::Center,
            valign: VAlign::Center,
            content: la_syllable(),
        };
        GridPage {
            width_pt: 595.0,
            height_pt: 842.0,
            rows: vec![GridRow {
                height_pt: 30.0,
                column_count: 3,
                has_label_region: true,
                measure_layout: vec![MeasureColumnLayout {
                    start_col: 2,
                    col_count: 1,
                    weight: 1.0,
                    column_weights: vec![column_weight],
                    rod_pt: 24.0,
                    column_rods: vec![column_weight],
                    column_accidental_leads: vec![0.0],
                }],
                elements: vec![el],
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
    };
    let resolve_lyric_x = |column_weight: f32| -> f32 {
        let abs = resolve(
            &[make_page(column_weight)],
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
                ..Default::default()
            },
        )
        .unwrap();
        abs[0]
            .elements
            .iter()
            .find(|e| matches!(e.content, AbsoluteContent::Lyric { .. }))
            .expect("should have Lyric")
            .x
    };

    let narrow_x = resolve_lyric_x(1.0);
    let inflated_x = resolve_lyric_x(100.0);

    assert!(
        (narrow_x - inflated_x).abs() < 0.01,
        "narrow_x={narrow_x} inflated_x={inflated_x} should match: the syllable's anchor must \
         not depend on the column's weight"
    );
}

#[test]
fn lyric_syllable_shares_the_note_head_padding_formula() {
    // Every lyric syllable (Latin or CJK) is now bearing-corrected against
    // the CJK font `render_lyric` always draws in (see
    // `resolve::flush_left_padding`) — a Latin leading character's bearing
    // in that font is nonzero too, so a Latin syllable's `x` sits left of
    // the flat configured padding value, the same shape as
    // `cjk_lyric_syllable_compensates_its_leading_glyphs_left_bearing`.
    let note_number_width = 12.0;
    let base_font_size = 14.4;
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::LyricSyllable {
            text: "la".to_string(),
            ruby: None,
            ruby_band_pt: 0.0,
            source_part_index: 0,
            note_id: 0,
        },
    };
    let page = single_row_page(el);
    let abs = resolve(
        &[page],
        note_number_width,
        super::tests::TEST_PAGE_FRAME,
        ResolveFontSizes {
            lyric: LyricFontSizes {
                base: base_font_size,
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
    let lyric = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::Lyric { .. }))
        .expect("should have Lyric");
    let x_start = 0.0; // column 0 starts at the row's own left edge
    let bearing = crate::font_metrics::cjk_glyph_left_bearing('l', base_font_size);
    let expected_x = 25.0 + x_start + (4.0f32 - bearing).max(0.0);
    assert!(
        (lyric.x - expected_x).abs() < 0.01,
        "x={} expected={expected_x} bearing={bearing}",
        lyric.x
    );
}

#[test]
fn cjk_lyric_syllable_compensates_its_leading_glyphs_left_bearing() {
    // A CJK syllable's own leading character carries a built-in left-side
    // bearing, so its resolved x must sit `bearing` points left of what the
    // plain flat-padding formula (see
    // `lyric_syllable_shares_the_note_head_padding_formula`) would give a
    // Latin syllable at the same column.
    let note_number_width = 12.0;
    let cjk_font_size = 17.28;
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::LyricSyllable {
            text: "漢字".to_string(),
            ruby: None,
            ruby_band_pt: 0.0,
            source_part_index: 0,
            note_id: 0,
        },
    };
    let page = single_row_page(el);
    let abs = resolve(
        &[page],
        note_number_width,
        super::tests::TEST_PAGE_FRAME,
        ResolveFontSizes {
            lyric: LyricFontSizes {
                base: 14.4,
                cjk: cjk_font_size,
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
    let lyric = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::Lyric { .. }))
        .expect("should have Lyric");

    let x_start = 0.0; // column 0 starts at the row's own left edge
    let bearing = crate::font_metrics::cjk_glyph_left_bearing('漢', cjk_font_size);
    let expected_x = 25.0 + x_start + (4.0f32 - bearing).max(0.0);
    assert!(bearing > 0.0, "test is only meaningful if bearing > 0.0");
    assert!(
        (lyric.x - expected_x).abs() < 0.01,
        "x={} expected={expected_x} bearing={bearing}",
        lyric.x
    );
    assert!(
        lyric.x < 25.0 + x_start + 4.0,
        "a CJK syllable's x should sit left of the flat configured-padding x"
    );
}

fn la_syllable() -> GridContent {
    GridContent::LyricSyllable {
        text: "la".to_string(),
        ruby: None,
        ruby_band_pt: 0.0,
        source_part_index: 0,
        note_id: 0,
    }
}
