use crate::ast::parsed::{NumberedPitch, Offset};
use crate::compiler::types::{
    ColumnElement, CompileResult, Decoration, ElementContent, MeasureBlock, MeasureRow, RowId,
};
use crate::grid_layout::layout::{layout, LABEL_COLS};
use crate::grid_layout::types::{GridContent, Header};
use crate::render_config::RenderConfig;

// ── decoration row helpers ────────────────────────────────────────────────────

fn make_block_with_decorations(decorations: Vec<Decoration>) -> MeasureBlock {
    MeasureBlock {
        rows: vec![MeasureRow {
            kind: crate::ast::parsed::PartKind::Notes,
            absorbed_rows: Vec::new(),
            id: RowId("S".to_string()),
            label: "S".to_string(),
            elements: vec![
                ColumnElement {
                    column: 0,
                    content: ElementContent::NoteHead {
                        pitch: NumberedPitch::One,
                        accidental: crate::ast::parsed::Accidental::Natural,
                        octave: 0,
                        dotted: false,
                        double_dotted: false,
                        head_label: None,
                    },
                    note_id: None,
                },
                ColumnElement {
                    column: 3,
                    content: ElementContent::BarLine,
                    note_id: None,
                },
            ],
            source_part_index: 0,
        }],
        decorations,
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
        system_break: false,
        source_span: crate::error::Span::new(0, 0),
    }
}

fn directive_line(
    label: Option<&str>,
    bar_number: Option<u32>,
    key: Option<&str>,
    bpm: Option<u32>,
    time_signature: Option<(u32, u32)>,
) -> Decoration {
    Decoration::DirectiveLine {
        label: label.map(|s| s.to_string()),
        bar_number,
        key: key.map(|s| s.to_string()),
        bpm,
        time_signature,
    }
}

fn hdr() -> Header {
    Header {
        title: Some("Song".to_string()),
        subtitle: None,
        author: Some("Me".to_string()),
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

fn cfg_wide() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        note_number_width: 12,
        part_label_width_pt: 40,
        max_measures_per_system: 48,
        lyrics_font_size: 18,
        notes_font_size: 18,
        note_dash_font_size: 18,
        chords_font_size: 18,
        hide_system_dividers: false,
        directive_row_offset: Offset::default(),
        measure_number_font_size: 10,
        section_label_font_size: 12,
        part_label_font_size: 12,
        page_number_font_size: 18,
        lyric_click_target_padding_pt: 12,
        notes_vertical_padding_pt: 0,
        section_label_vertical_padding_pt: 0,
        page_number_vertical_padding_pt: 0,
        notes_horizontal_padding_pt: 4,
        chords_horizontal_padding_pt: 4,
        lyrics_horizontal_padding_pt: 4,
        note_dash_horizontal_padding_pt: 4,
        ..Default::default()
    }
}

fn make_block(row_id: &str, bar_col: u32) -> MeasureBlock {
    MeasureBlock {
        rows: vec![MeasureRow {
            kind: crate::ast::parsed::PartKind::Notes,
            absorbed_rows: Vec::new(),
            id: RowId(row_id.to_string()),
            label: row_id.to_string(),
            elements: vec![
                ColumnElement {
                    column: 0,
                    content: ElementContent::NoteHead {
                        pitch: NumberedPitch::One,
                        accidental: crate::ast::parsed::Accidental::Natural,
                        octave: 0,
                        dotted: false,
                        double_dotted: false,
                        head_label: None,
                    },
                    note_id: None,
                },
                ColumnElement {
                    column: bar_col,
                    content: ElementContent::BarLine,
                    note_id: None,
                },
            ],
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

#[test]
fn layout_with_bpm_decoration_has_decoration_row() {
    let block =
        make_block_with_decorations(vec![directive_line(None, None, None, Some(120), None)]);
    let compile_result = CompileResult {
        blocks: vec![block],
        slur_spans: vec![],
        tuplet_spans: vec![],
        lyric_links: vec![],
    };
    let pages = layout_a4(&compile_result);
    let has_directive = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .any(|e| {
            matches!(
                &e.content,
                GridContent::DirectiveLine { bpm: Some(120), .. }
            )
        });
    assert!(
        has_directive,
        "should have DirectiveLine with bpm=120 element"
    );
}

#[test]
fn decoration_row_shares_column_count_with_music_rows() {
    let block =
        make_block_with_decorations(vec![directive_line(None, None, None, Some(120), None)]);
    let compile_result = CompileResult {
        blocks: vec![block],
        slur_spans: vec![],
        tuplet_spans: vec![],
        lyric_links: vec![],
    };
    let pages = layout_a4(&compile_result);
    let deco_row = pages[0]
        .rows
        .iter()
        .find(|r| {
            r.elements
                .iter()
                .any(|e| matches!(&e.content, GridContent::DirectiveLine { bpm: Some(_), .. }))
        })
        .expect("should have a decoration row with bpm");
    let music_row = pages[0]
        .rows
        .iter()
        .find(|r| {
            r.elements
                .iter()
                .any(|e| matches!(&e.content, GridContent::BarLine { .. }))
        })
        .expect("should have a music row with a bar line");
    assert_eq!(
        deco_row.column_count, music_row.column_count,
        "decoration row should share the music rows' column grid so labels align to measures"
    );
}

#[test]
fn decoration_items_start_at_first_measure_left_edge() {
    let block =
        make_block_with_decorations(vec![directive_line(None, None, None, Some(120), None)]);
    let compile_result = CompileResult {
        blocks: vec![block],
        slur_spans: vec![],
        tuplet_spans: vec![],
        lyric_links: vec![],
    };
    let pages = layout_a4(&compile_result);
    let directive_el = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| matches!(&e.content, GridContent::DirectiveLine { bpm: Some(_), .. }))
        .expect("should have DirectiveLine element");
    assert_eq!(
        directive_el.column, LABEL_COLS,
        "directive line should align with the leading barline of the first measure"
    );
}

#[test]
fn label_and_bpm_are_merged_into_single_directive_line() {
    // Both label and bpm should be combined into one DirectiveLine element.
    let block =
        make_block_with_decorations(vec![directive_line(Some("A"), None, None, Some(120), None)]);
    let compile_result = CompileResult {
        blocks: vec![block],
        slur_spans: vec![],
        tuplet_spans: vec![],
        lyric_links: vec![],
    };
    let pages = layout_a4(&compile_result);
    let directive_elements: Vec<_> = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .filter(|e| {
            matches!(
                &e.content,
                GridContent::DirectiveLine {
                    label: Some(_),
                    bpm: Some(_),
                    ..
                }
            )
        })
        .collect();
    assert_eq!(
        directive_elements.len(),
        1,
        "should have exactly one DirectiveLine with label and bpm"
    );
    assert_eq!(directive_elements[0].column, LABEL_COLS);
}

#[test]
fn bpm_and_time_signature_merged_into_single_directive_line_at_column_1() {
    let block = make_block_with_decorations(vec![directive_line(
        None,
        None,
        None,
        Some(120),
        Some((4, 4)),
    )]);
    let compile_result = CompileResult {
        blocks: vec![block],
        slur_spans: vec![],
        tuplet_spans: vec![],
        lyric_links: vec![],
    };
    let pages = layout_a4(&compile_result);
    let directive_el = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| {
            matches!(
                &e.content,
                GridContent::DirectiveLine {
                    bpm: Some(_),
                    time_signature: Some(_),
                    ..
                }
            )
        })
        .expect("should have DirectiveLine with bpm and time_signature");
    assert_eq!(
        directive_el.column, LABEL_COLS,
        "directive line should align with the leading barline of the first measure"
    );
}

#[test]
fn section_label_on_non_first_measure_of_system_is_rendered() {
    // Two measures in one system; only the second has a label.
    let first_block = make_block("S", 3); // bar at col 3, width = 4
    let mut second_block = make_block("S", 3);
    second_block.decorations = vec![directive_line(Some("B"), None, None, None, None)];
    let compile_result = CompileResult {
        blocks: vec![first_block, second_block],
        slur_spans: vec![],
        tuplet_spans: vec![],
        lyric_links: vec![],
    };
    let pages = layout_a4(&compile_result);
    let has_label = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .any(|e| {
            matches!(
                &e.content,
                GridContent::DirectiveLine { label: Some(s), .. } if s == "B"
            )
        });
    assert!(
        has_label,
        "DirectiveLine with label 'B' on a non-first measure should be rendered"
    );
}

#[test]
fn section_label_on_non_first_measure_is_right_of_column_1() {
    // First block has no decorations; second block has a label.
    // The directive line should appear in a column > 1.
    let first_block = make_block("S", 3);
    let mut second_block = make_block("S", 3);
    second_block.decorations = vec![directive_line(Some("B"), None, None, None, None)];
    let compile_result = CompileResult {
        blocks: vec![first_block, second_block],
        slur_spans: vec![],
        tuplet_spans: vec![],
        lyric_links: vec![],
    };
    let pages = layout_a4(&compile_result);
    let label_col = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| {
            matches!(
                &e.content,
                GridContent::DirectiveLine { label: Some(s), .. } if s == "B"
            )
        })
        .expect("should find DirectiveLine with label B")
        .column;
    assert!(
        label_col > 1,
        "directive line on 2nd measure should be right of column 1, got {label_col}"
    );
}

/// `compile_result` laid out on A4 pages with the wide test config.
fn layout_a4(compile_result: &CompileResult) -> Vec<crate::grid_layout::types::GridPage> {
    layout(
        compile_result,
        &cfg_wide(),
        &hdr(),
        crate::grid_layout::PageLayout::A4,
        None,
    )
    .pages
}
