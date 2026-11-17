// ── Proportional (density-based) measure widths ──────────────────────────────

use crate::ast::parsed::{NumberedPitch, Offset};
use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::coordinate_resolver::LyricFontSizes;
use crate::font_metrics;
use crate::grid_layout::expand::expand_system_to_rows;
use crate::grid_layout::layout::{
    build_measure_column_layout, measure_column_weights, LyricSizing,
};
use crate::render_config::RenderConfig;
use std::collections::HashMap;

fn test_config() -> RenderConfig {
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

/// Real rendered width of a single notehead/rest/percussion-hit digit glyph
/// at `config`'s notes font size — the ground-truth unit `measure_column_weights`/
/// `build_measure_column_layout` now weigh a fresh note-onset column in,
/// mirroring `layout_spacing::note_glyph_weight`.
fn notehead_weight(config: &RenderConfig) -> f32 {
    font_metrics::monospace_char_advance_width('0', config.notes_font_size())
}

/// Real rendered width of the note-dash glyph at `config`'s notes font size,
/// mirroring `layout_spacing::dash_weight`.
fn dash_weight(config: &RenderConfig) -> f32 {
    font_metrics::monospace_char_advance_width('\u{2014}', config.notes_font_size())
}

fn make_block_with_notes(row_id: &str, note_count: u32, bar_col: u32) -> MeasureBlock {
    let mut elements: Vec<ColumnElement> = (0..note_count)
        .map(|col| ColumnElement {
            column: col,
            content: ElementContent::NoteHead {
                pitch: NumberedPitch::One,
                accidental: crate::ast::parsed::Accidental::Natural,
                octave: 0,
                dotted: false,
                double_dotted: false,
                head_label: None,
            },
            note_id: None,
        })
        .collect();
    elements.push(ColumnElement {
        column: bar_col,
        content: ElementContent::BarLine,
        note_id: None,
    });
    MeasureBlock {
        rows: vec![MeasureRow {
            kind: crate::ast::parsed::PartKind::Notes,
            absorbed_rows: Vec::new(),
            id: RowId(row_id.to_string()),
            label: row_id.to_string(),
            elements,
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

fn make_block_with_dash(row_id: &str, bar_col: u32) -> MeasureBlock {
    // A single half note (`NoteHead` followed by one `NoteDash`) plus a bar line.
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
                    column: 1,
                    content: ElementContent::NoteDash {
                        dotted: false,
                        double_dotted: false,
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
fn measure_column_weights_scales_with_note_count() {
    let config = test_config();
    let sparse = make_block_with_notes("S", 1, 7);
    let dense = make_block_with_notes("S", 16, 16);
    let sparse_weight: f32 = measure_column_weights(&sparse, 8, &config).iter().sum();
    let dense_weight: f32 = measure_column_weights(&dense, 17, &config).iter().sum();
    assert!(dense_weight > sparse_weight);
}

#[test]
fn measure_column_weights_gives_dash_the_same_weight_as_a_fresh_note() {
    // A half note (`NoteHead` + `NoteDash`) spans the same 2 columns as two
    // quarter notes. The dash's glyph renders at `config`'s notes font size
    // (matching a fresh notehead), so the two columns weigh the same and a
    // half note weighs the same as two quarter notes.
    let config = test_config();
    let half_note = make_block_with_dash("S", 2);
    let weights = measure_column_weights(&half_note, 3, &config);
    assert_eq!(weights[0], notehead_weight(&config));
    assert_eq!(weights[1], dash_weight(&config));
    assert_eq!(
        dash_weight(&config),
        notehead_weight(&config),
        "dash weight {} should equal notehead weight {} since both render \
         at the notes font size",
        dash_weight(&config),
        notehead_weight(&config)
    );

    let two_quarters = make_block_with_notes("S", 2, 2);
    let quarters_weight: f32 = measure_column_weights(&two_quarters, 3, &config)
        .iter()
        .sum();
    let half_note_weight: f32 = weights.iter().sum();
    assert_eq!(
        half_note_weight, quarters_weight,
        "one half note ({half_note_weight}) should weigh the same as two \
         quarter notes ({quarters_weight}), since its dash now renders at \
         the same size as a fresh notehead"
    );
}

#[test]
fn measure_column_weights_takes_max_across_parts_not_sum() {
    // Part A and part B both have a fresh note at column 0 — the column
    // should weigh as much as one note (the widest need), not the sum of
    // both parts' weights.
    let config = test_config();
    let mut block = make_block_with_dash("A", 2);
    block
        .rows
        .push(make_block_with_notes("B", 1, 2).rows.remove(0));
    let weights = measure_column_weights(&block, 3, &config);
    assert_eq!(weights[0], notehead_weight(&config));
}

#[test]
fn build_measure_column_layout_never_collapses_to_zero() {
    let config = test_config();
    let empty = make_block_with_notes("S", 0, 0);
    let layout = build_measure_column_layout(&[empty], &config);
    assert_eq!(layout[0].weight, notehead_weight(&config));
}

#[test]
fn build_measure_column_layout_tracks_start_col_and_weight_per_measure() {
    let config = test_config();
    let system = vec![
        make_block_with_notes("S", 1, 3), // 4 cols, weight = 1 note
        make_block_with_notes("S", 8, 8), // 9 cols, weight = 8 notes
    ];
    let layout = build_measure_column_layout(&system, &config);
    assert_eq!(layout.len(), 2);
    // First measure's col_count is widened by 1 to absorb the system's
    // leading bar-line column (see `build_measure_column_layout`), but its
    // aggregate `weight` is unaffected — only real notes count there.
    assert_eq!(layout[0].col_count, 5);
    assert_eq!(layout[0].weight, notehead_weight(&config));
    assert_eq!(
        layout[1].start_col,
        layout[0].start_col + layout[0].col_count
    );
    assert_eq!(layout[1].col_count, 9);
    assert!(
        (layout[1].weight - 8.0 * notehead_weight(&config)).abs() < 0.001,
        "weight={} should be ~8x notehead_weight={}",
        layout[1].weight,
        notehead_weight(&config)
    );
}

#[test]
fn build_measure_column_layout_gives_equal_density_measures_equal_weight_regardless_of_position() {
    // The leading bar-line column absorbed into the first measure (see
    // `build_measure_column_layout`) must not inflate its aggregate weight
    // relative to an identical measure later in the system.
    let config = test_config();
    let system = vec![
        make_block_with_notes("S", 4, 7),
        make_block_with_notes("S", 4, 7),
    ];
    let layout = build_measure_column_layout(&system, &config);
    assert_eq!(layout[0].weight, layout[1].weight);
}

#[test]
fn dense_measure_renders_wider_than_sparse_measure_in_same_system() {
    let config = test_config();
    let system = vec![
        make_block_with_notes("S", 1, 3),   // sparse: 1 note
        make_block_with_notes("S", 16, 16), // dense: 16 notes
    ];
    let measure_layout = build_measure_column_layout(&system, &config);
    let rows = expand_system_to_rows(
        &system,
        30.0,
        &HashMap::new(),
        &HashMap::new(),
        &measure_layout,
        LyricSizing {
            font_sizes: LyricFontSizes {
                base: 18.0,
                cjk: 21.6,
            },
            click_target_padding_pt: 12.0,
            notes_vertical_padding_pt: 0.0,
        },
    );
    let row = &rows[0];
    let geometry = row.column_geometry(1000.0, 40.0);

    let sparse = &measure_layout[0];
    let dense = &measure_layout[1];
    let sparse_width = geometry.x_start((sparse.start_col + sparse.col_count) as f32)
        - geometry.x_start(sparse.start_col as f32);
    let dense_width = geometry.x_start((dense.start_col + dense.col_count) as f32)
        - geometry.x_start(dense.start_col as f32);

    assert!(
        dense_width > sparse_width,
        "dense_width={dense_width} should be > sparse_width={sparse_width}"
    );
}

#[test]
fn equal_density_measures_render_at_equal_width() {
    let config = test_config();
    let system = vec![
        make_block_with_notes("S", 4, 7),
        make_block_with_notes("S", 4, 7),
    ];
    let measure_layout = build_measure_column_layout(&system, &config);
    let rows = expand_system_to_rows(
        &system,
        30.0,
        &HashMap::new(),
        &HashMap::new(),
        &measure_layout,
        LyricSizing {
            font_sizes: LyricFontSizes {
                base: 18.0,
                cjk: 21.6,
            },
            click_target_padding_pt: 12.0,
            notes_vertical_padding_pt: 0.0,
        },
    );
    let row = &rows[0];
    let geometry = row.column_geometry(1000.0, 40.0);

    let widths: Vec<f32> = measure_layout
        .iter()
        .map(|m| {
            geometry.x_start((m.start_col + m.col_count) as f32)
                - geometry.x_start(m.start_col as f32)
        })
        .collect();
    assert!(
        (widths[0] - widths[1]).abs() < 0.001,
        "widths={widths:?} should be equal for equal-density measures"
    );
}

#[test]
fn proportional_widths_sum_to_full_usable_music_width() {
    let config = test_config();
    let system = vec![
        make_block_with_notes("S", 1, 3),
        make_block_with_notes("S", 5, 5),
        make_block_with_notes("S", 16, 16),
    ];
    let measure_layout = build_measure_column_layout(&system, &config);
    let rows = expand_system_to_rows(
        &system,
        30.0,
        &HashMap::new(),
        &HashMap::new(),
        &measure_layout,
        LyricSizing {
            font_sizes: LyricFontSizes {
                base: 18.0,
                cjk: 21.6,
            },
            click_target_padding_pt: 12.0,
            notes_vertical_padding_pt: 0.0,
        },
    );
    let row = &rows[0];
    let usable_width = 1000.0_f32;
    let label_width = 40.0_f32;
    let geometry = row.column_geometry(usable_width, label_width);

    let last = measure_layout.last().unwrap();
    let end_col = (last.start_col + last.col_count) as f32;
    let total_music_width = geometry.x_start(end_col) - label_width;
    assert!(
        (total_music_width - (usable_width - label_width)).abs() < 0.01,
        "total_music_width={total_music_width}"
    );
}
