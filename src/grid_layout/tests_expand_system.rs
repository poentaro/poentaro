//! `expand_system_to_rows` tests — split out of `tests.rs` to keep it under
//! the max-file-lines lint.

use super::make_block;
use crate::ast::parsed::NumberedPitch;
use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::coordinate_resolver::LyricFontSizes;
use crate::grid_layout::layout::{
    expand_system_to_rows, system_lyric_height_pt, system_musical_height_pt, LyricSizing,
    SystemBands,
};
use crate::grid_layout::types::GridContent;
use std::collections::HashMap;

fn make_system_single_note_block() -> Vec<MeasureBlock> {
    vec![make_block("S", 3)] // 4 musical cols, bar at compiler col 3
}

#[test]
fn note_block_expands_to_six_sub_rows_without_tuplet() {
    let rows = expand_system_to_rows(
        &make_system_single_note_block(),
        30.0,
        &HashMap::new(),
        &HashMap::new(),
        &[],
        LyricSizing {
            font_sizes: LyricFontSizes {
                base: 18.0,
                cjk: 21.6,
            },
            click_target_padding_pt: 12.0,
            notes_vertical_padding_pt: 0.0,
        },
    );
    // 1 note part × 6 sub-rows (no `tuplet_bracket` sub-row reserved, since
    // this system has no tuplet), no lyric.
    assert_eq!(rows.len(), 6);
}

#[test]
fn note_head_element_is_in_sub_row_index_2_without_tuplet() {
    let rows = expand_system_to_rows(
        &make_system_single_note_block(),
        30.0,
        &HashMap::new(),
        &HashMap::new(),
        &[],
        LyricSizing {
            font_sizes: LyricFontSizes {
                base: 18.0,
                cjk: 21.6,
            },
            click_target_padding_pt: 12.0,
            notes_vertical_padding_pt: 0.0,
        },
    );
    let note_row = &rows[2]; // note-head sub-row (no tuplet_bracket sub-row ahead of it)
    let has_note = note_row
        .elements
        .iter()
        .any(|e| matches!(e.content, GridContent::NoteHead { .. }));
    assert!(has_note, "note head should be in sub-row 2");
}

#[test]
fn bar_line_element_has_positive_height_pt() {
    let rows = expand_system_to_rows(
        &make_system_single_note_block(),
        30.0,
        &HashMap::new(),
        &HashMap::new(),
        &[],
        LyricSizing {
            font_sizes: LyricFontSizes {
                base: 18.0,
                cjk: 21.6,
            },
            click_target_padding_pt: 12.0,
            notes_vertical_padding_pt: 0.0,
        },
    );
    let bar = rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| matches!(e.content, GridContent::BarLine { .. }));
    let bar = bar.expect("should have a BarLine element");
    if let GridContent::BarLine { height_pt, .. } = bar.content {
        assert!(height_pt > 0.0, "height_pt={height_pt}");
    }
}

fn make_block_with_lyric_part(bar_col: u32) -> MeasureBlock {
    MeasureBlock {
        rows: vec![
            MeasureRow {
                kind: crate::ast::parsed::PartKind::Notes,
                absorbed_rows: Vec::new(),
                id: RowId("note".to_string()),
                label: "note".to_string(),
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
            },
            MeasureRow {
                kind: crate::ast::parsed::PartKind::Notes,
                absorbed_rows: Vec::new(),
                id: RowId("lyric".to_string()),
                label: "lyric".to_string(),
                elements: vec![ColumnElement {
                    column: 0,
                    content: ElementContent::Lyric {
                        text: "la".to_string(),
                        ruby: None,
                        note_id: 0,
                    },
                    note_id: None,
                }],
                source_part_index: 0,
            },
        ],
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
        system_break: false,
        source_span: crate::error::Span::new(0, 0),
    }
}

#[test]
fn bar_line_height_includes_lyric_rows() {
    let base = 30.0_f32;
    let lyric_sizing = LyricSizing {
        font_sizes: LyricFontSizes {
            base: 18.0,
            cjk: 21.6,
        },
        click_target_padding_pt: 12.0,
        notes_vertical_padding_pt: 0.0,
    };
    let system = vec![make_block_with_lyric_part(3)];
    let first = system.first().unwrap();
    let expected_height = system_musical_height_pt(
        first,
        base,
        &SystemBands::default(),
        lyric_sizing.notes_vertical_padding_pt,
    ) + system_lyric_height_pt(&system, base, lyric_sizing);

    let rows = expand_system_to_rows(
        &system,
        base,
        &HashMap::new(),
        &HashMap::new(),
        &[],
        lyric_sizing,
    );
    let bar = rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| matches!(e.content, GridContent::BarLine { .. }))
        .expect("should have a BarLine element");
    let GridContent::BarLine { height_pt, .. } = bar.content else {
        panic!("expected BarLine content");
    };
    assert!(
        (height_pt - expected_height).abs() < 0.001,
        "bar height={height_pt}, expected={expected_height} (musical + lyric)"
    );
}

#[test]
fn row_label_is_in_note_head_sub_row_at_column_0_span_1() {
    let rows = expand_system_to_rows(
        &make_system_single_note_block(),
        30.0,
        &HashMap::new(),
        &HashMap::new(),
        &[],
        LyricSizing {
            font_sizes: LyricFontSizes {
                base: 18.0,
                cjk: 21.6,
            },
            click_target_padding_pt: 12.0,
            notes_vertical_padding_pt: 0.0,
        },
    );
    let note_row = &rows[2]; // no tuplet in this system, so notehead is sub-row 2
    let label = note_row
        .elements
        .iter()
        .find(|e| matches!(e.content, GridContent::RowLabel(_)));
    let label = label.expect("note-head row should have RowLabel");
    assert_eq!(label.column, 0);
    assert_eq!(label.column_span, 1);
}

#[test]
fn column_count_is_label_cols_plus_musical_cols() {
    let rows = expand_system_to_rows(
        &make_system_single_note_block(),
        30.0,
        &HashMap::new(),
        &HashMap::new(),
        &[],
        LyricSizing {
            font_sizes: LyricFontSizes {
                base: 18.0,
                cjk: 21.6,
            },
            click_target_padding_pt: 12.0,
            notes_vertical_padding_pt: 0.0,
        },
    );
    // 1 label col + 1 leading bar line col + 4 musical cols (bar at col 3 → block width=4)
    assert_eq!(rows[0].column_count, 6);
}
