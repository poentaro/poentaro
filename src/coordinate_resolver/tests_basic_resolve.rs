use crate::ast::parsed::{Accidental, NumberedPitch};
use crate::compositor::types::AbsoluteContent;
use crate::coordinate_resolver::resolve::{
    resolve, ElementPaddings, LabelFontSizes, LyricFontSizes, ResolveFontSizes,
};
use crate::grid_layout::types::{GridContent, GridElement, GridPage, GridRow, HAlign, VAlign};

/// Shared default padding used across this file's `ResolveFontSizes` literals, factored out to keep each test under clippy's line-count cap.
const DEFAULT_PADDINGS: ElementPaddings = ElementPaddings {
    notes: 4.0,
    chords: 4.0,
    lyrics: 4.0,
    note_dash: 4.0,
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
fn valign_top_places_y_at_row_top() {
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Start,
        valign: VAlign::Top,
        content: GridContent::HorizontalLine,
    };
    let page = GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![
            GridRow {
                height_pt: 10.0,
                column_count: 1,
                has_label_region: false,
                measure_layout: vec![],
                elements: vec![],
            },
            GridRow {
                height_pt: 20.0,
                column_count: 1,
                has_label_region: false,
                measure_layout: vec![],
                elements: vec![el],
            },
        ],
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
    let line = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::HorizontalLine { .. }))
        .expect("should have HorizontalLine");
    // row_y = page margin + 10.0 = 35.0; VAlign::Top → y = row_y
    assert!((line.y - 35.0).abs() < 0.01, "y={}", line.y);
}

#[test]
fn halign_end_places_x_at_right_of_column_span() {
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::End,
        valign: VAlign::Center,
        content: GridContent::Text {
            content: "Author".to_string(),
            font_size: 12.0,
            bold: false,
            italic: false,
            underline: false,
            font_family: crate::compositor::types::FontFamily::SansSerif,
        },
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
    let text = abs[0]
        .elements
        .iter()
        .find(
            |e| matches!(&e.content, AbsoluteContent::Text { content, .. } if content == "Author"),
        )
        .expect("should have Text");
    let col_width = (595.0 - 50.0) / 10.0;
    let expected_x = 25.0 + col_width; // Start + 1*col_width = 25 + 54.5
    assert!(
        (text.x - expected_x).abs() < 0.01,
        "x={} expected={expected_x}",
        text.x
    );
}

#[test]
fn low_octave_dots_resolve_to_the_same_x_as_their_note_head() {
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::LowOctaveDots {
            count: 1,
            pitch: NumberedPitch::Five,
            head_label: None,
        },
    };
    let note_head = GridElement {
        content: GridContent::NoteHead {
            pitch: NumberedPitch::Five,
            accidental: Accidental::Natural,
            octave: 0,
            dotted: false,
            double_dotted: false,
            head_label: None,
        },
        ..el
    };
    let mut page = single_row_page(el);
    page.rows[0].elements.push(note_head);
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
    let [dots, head] = abs[0].elements.as_slice() else {
        panic!(
            "expected the dots and the note head, got {:?}",
            abs[0].elements
        );
    };
    assert!(matches!(
        dots.content,
        AbsoluteContent::LowOctaveDots { count: 1 }
    ));
    assert_eq!(
        dots.x, head.x,
        "the dots must center on the digit exactly as the head's own inline dots do"
    );
}
