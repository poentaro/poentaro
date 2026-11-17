use crate::ast::parsed::{NumberedPitch, Offset};
use crate::compiler::types::{ColumnElement, ElementContent, MeasureRow, RowId};
use crate::grid_layout::types::GridRow;
use crate::render_config::RenderConfig;

#[test]
fn column_geometry_divides_evenly_without_label_region() {
    let row = GridRow {
        height_pt: 30.0,
        column_count: 10,
        has_label_region: false,
        measure_layout: vec![],
        elements: vec![],
    };
    let geometry = row.column_geometry(500.0, 40.0);
    assert_eq!(geometry.col_width(0.0), 50.0);
}

#[test]
fn column_geometry_gives_label_region_a_fixed_width() {
    // 1 label col + 15 musical cols = 16 total; usable=400, label=40pt fixed
    // regardless of column_count → musical cols get (400-40)/15 = 24pt each.
    let row = GridRow {
        height_pt: 30.0,
        column_count: 16,
        has_label_region: true,
        measure_layout: vec![],
        elements: vec![],
    };
    let geometry = row.column_geometry(400.0, 40.0);
    assert_eq!(geometry.col_width(0.0), 40.0); // 40pt / 1 label col
    assert_eq!(geometry.col_width(5.0), 24.0);
    assert_eq!(geometry.x_start(0.0), 0.0);
    assert_eq!(geometry.x_start(1.0), 40.0);
}

#[test]
fn column_geometry_label_width_is_independent_of_musical_density() {
    // Two systems with the same label width but different musical column
    // counts must still render the label at the same pixel width — this is
    // the fix for the part-label-width-varies-by-system bug.
    let sparse_row = GridRow {
        height_pt: 30.0,
        column_count: 3, // 1 label col + 1 barline col + 1 musical col
        has_label_region: true,
        measure_layout: vec![],
        elements: vec![],
    };
    let dense_row = GridRow {
        height_pt: 30.0,
        column_count: 21, // 1 label col + 1 barline col + 19 musical cols
        has_label_region: true,
        measure_layout: vec![],
        elements: vec![],
    };
    let sparse_geometry = sparse_row.column_geometry(545.0, 40.0);
    let dense_geometry = dense_row.column_geometry(545.0, 40.0);
    let sparse_label_width = sparse_geometry.x_start(1.0) - sparse_geometry.x_start(0.0);
    let dense_label_width = dense_geometry.x_start(1.0) - dense_geometry.x_start(0.0);
    assert_eq!(sparse_label_width, dense_label_width);
    assert_eq!(sparse_label_width, 40.0);
}

fn note_row(id: &str) -> MeasureRow {
    MeasureRow {
        kind: crate::ast::parsed::PartKind::Notes,
        absorbed_rows: Vec::new(),
        id: RowId(id.to_string()),
        label: id.to_string(),
        elements: vec![ColumnElement {
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
        }],
        source_part_index: 0,
    }
}

fn chord_row(id: &str) -> MeasureRow {
    MeasureRow {
        kind: crate::ast::parsed::PartKind::Notes,
        absorbed_rows: Vec::new(),
        id: RowId(id.to_string()),
        label: id.to_string(),
        elements: vec![ColumnElement {
            column: 0,
            content: ElementContent::ChordSymbol {
                text: "Am".to_string(),
                degrees: crate::compiler::test_chord_degrees::tonic_major(),
                guitar_voicing: None,
                dotted: false,
                double_dotted: false,
            },
            note_id: None,
        }],
        source_part_index: 0,
    }
}

fn lyric_row(id: &str) -> MeasureRow {
    MeasureRow {
        kind: crate::ast::parsed::PartKind::Lyrics {
            target_part_index: 0,
        },
        absorbed_rows: Vec::new(),
        id: RowId(id.to_string()),
        label: id.to_string(),
        elements: vec![ColumnElement {
            column: 0,
            content: ElementContent::Lyric {
                text: "la".to_string(),
                ruby: None,
                note_id: 0,
            },
            note_id: None,
        }],
        source_part_index: 1,
    }
}

use crate::compiler::types::MeasureBlock;
use crate::grid_layout::layout::{
    chord_part_sub_row_heights, is_chord_only_row, is_lyric_row, note_part_sub_row_heights,
    pack_into_systems,
};

pub(crate) fn make_block(row_id: &str, bar_col: u32) -> MeasureBlock {
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

fn cfg() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        note_number_width: 12,
        part_label_width_pt: 40,
        max_measures_per_system: 2,
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

#[test]
fn is_lyric_row_detects_lyric() {
    assert!(is_lyric_row(&lyric_row("L")));
    assert!(!is_lyric_row(&note_row("S")));
}

#[test]
fn is_chord_only_row_detects_chord() {
    assert!(is_chord_only_row(&chord_row("C")));
    assert!(!is_chord_only_row(&note_row("S")));
    assert!(!is_chord_only_row(&lyric_row("L")));
}

#[test]
fn note_part_sub_row_heights_sums_correctly() {
    let heights = note_part_sub_row_heights(30.0, 0.0);
    // tuplet_bracket + arc + above_dot + note_head + below_dot + ul + ul
    // = 30.0 + 9.0 + 7.5 + 30.0 + 7.5 + 7.5 + 7.5 = 99.0
    let sum: f32 = heights.iter().sum();
    assert!((sum - 99.0).abs() < 0.001, "sum={sum}");
    assert_eq!(heights.len(), 7);
}

#[test]
fn chord_part_sub_row_heights_has_four_rows() {
    let heights = chord_part_sub_row_heights(30.0);
    assert_eq!(heights.len(), 4);
}

#[test]
fn single_block_is_one_system() {
    let blocks = vec![make_block("S", 3)]; // 4 columns
    let systems = pack_into_systems(&blocks, &cfg(), f32::INFINITY);
    assert_eq!(systems.len(), 1);
    assert_eq!(systems[0].len(), 1);
}

#[test]
fn blocks_exceeding_max_measures_per_system_split_into_two_systems() {
    // max_measures_per_system=2 → fits 2 blocks per system
    let blocks = vec![make_block("S", 3), make_block("S", 3), make_block("S", 3)];
    let systems = pack_into_systems(&blocks, &cfg(), f32::INFINITY);
    assert_eq!(systems.len(), 2);
    assert_eq!(systems[0].len(), 2);
    assert_eq!(systems[1].len(), 1);
}

#[cfg(test)]
#[path = "tests_tuplets.rs"]
mod tests_tuplets;

#[cfg(test)]
#[path = "tests_expand_system.rs"]
mod tests_expand_system;
