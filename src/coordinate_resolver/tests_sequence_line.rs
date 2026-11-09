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
    GridContent, GridElement, GridPage, GridRow, HAlign, SequenceEntryInfo,
    SequenceEntryPartFilter, VAlign,
};
use crate::parser::sequence_parser::PartFilterKind;

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
fn sequence_line_renders_label_and_omit_parts_spans() {
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Start,
        valign: VAlign::Center,
        content: GridContent::SequenceLine {
            entries: vec![
                SequenceEntryInfo {
                    label: "Verse".to_string(),
                    part_filter: None,
                },
                SequenceEntryInfo {
                    label: "Chorus".to_string(),
                    part_filter: Some(SequenceEntryPartFilter {
                        kind: PartFilterKind::Omit,
                        parts: vec!["S".to_string(), "A2".to_string()],
                    }),
                },
            ],
            font_size: 12.0,
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
    let AbsoluteContent::DirectiveLine { spans, .. } = &abs[0].elements[0].content else {
        panic!(
            "expected a DirectiveLine, got {:?}",
            abs[0].elements[0].content
        );
    };
    let texts: Vec<&str> = spans.iter().map(|s| s.content.as_str()).collect();
    assert_eq!(
        texts,
        vec!["Sequence: ", "Verse", " \u{203a} ", "Chorus", " (-S -A2)"]
    );
    // Verse's suffix span is absent (no part filter); Chorus's isn't
    // bold/italic like the label span, since it's plain annotation text.
    let chorus_omit_span = &spans[4];
    assert!(!chorus_omit_span.bold);
    assert!(!chorus_omit_span.italic);
}

/// `Metadata::sequence`'s `bold`/`italic`/`underline` must style the label
/// spans on the `sequence` summary line — previously the line was
/// hardcoded to reuse `section_label`'s style, so `sequence`'s own toggles
/// had no rendering effect (see `syntax.md`'s "Text styles" section).
#[test]
fn sequence_line_label_span_uses_the_sequence_style() {
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Start,
        valign: VAlign::Center,
        content: GridContent::SequenceLine {
            entries: vec![SequenceEntryInfo {
                label: "Verse".to_string(),
                part_filter: None,
            }],
            font_size: 12.0,
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
                // Deliberately the opposite of `sequence_*` below, so a test
                // failure that falls back to `section_label`'s style (the
                // bug this test guards against) is unambiguous.
                section_label_bold: false,
                section_label_italic: false,
                section_label_underline: false,
                sequence_bold: true,
                sequence_italic: true,
                sequence_underline: true,
                ..Default::default()
            },
            paddings: DEFAULT_PADDINGS,
            page_number_vertical_padding_pt: 0.0,
            ..Default::default()
        },
    )
    .unwrap();
    let AbsoluteContent::DirectiveLine { spans, .. } = &abs[0].elements[0].content else {
        panic!(
            "expected a DirectiveLine, got {:?}",
            abs[0].elements[0].content
        );
    };
    let label_span = &spans[1];
    assert_eq!(label_span.content, "Verse");
    assert!(
        label_span.bold,
        "label span should be bold per sequence_bold"
    );
    assert!(
        label_span.italic,
        "label span should be italic per sequence_italic"
    );
    assert!(
        label_span.underline,
        "label span should be underlined per sequence_underline"
    );
}

#[test]
fn sequence_line_renders_only_parts_suffix_without_a_dash() {
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Start,
        valign: VAlign::Center,
        content: GridContent::SequenceLine {
            entries: vec![SequenceEntryInfo {
                label: "Chorus".to_string(),
                part_filter: Some(SequenceEntryPartFilter {
                    kind: PartFilterKind::Only,
                    parts: vec!["S".to_string()],
                }),
            }],
            font_size: 12.0,
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
    let AbsoluteContent::DirectiveLine { spans, .. } = &abs[0].elements[0].content else {
        panic!(
            "expected a DirectiveLine, got {:?}",
            abs[0].elements[0].content
        );
    };
    let texts: Vec<&str> = spans.iter().map(|s| s.content.as_str()).collect();
    assert_eq!(texts, vec!["Sequence: ", "Chorus", " (S)"]);
}
