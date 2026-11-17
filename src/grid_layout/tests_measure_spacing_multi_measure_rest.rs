// ── Multi-measure-rest column weight/width ───────────────────────────────────

use crate::ast::parsed::Offset;
use crate::compiler::types::MULTI_MEASURE_REST_WIDTH;
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

fn make_block_with_notes(row_id: &str, note_count: u32, bar_col: u32) -> MeasureBlock {
    let mut elements: Vec<ColumnElement> = (0..note_count)
        .map(|col| ColumnElement {
            column: col,
            content: ElementContent::NoteHead {
                pitch: crate::ast::parsed::NumberedPitch::One,
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

fn make_multi_measure_rest_block(row_id: &str, bar_col: u32, count: usize) -> MeasureBlock {
    MeasureBlock {
        rows: vec![MeasureRow {
            kind: crate::ast::parsed::PartKind::Notes,
            absorbed_rows: Vec::new(),
            id: RowId(row_id.to_string()),
            label: row_id.to_string(),
            elements: vec![
                ColumnElement {
                    column: 0,
                    content: ElementContent::MultiMeasureRest { count },
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
        represents_measures: count,
        merge_duplicate_measures_across_parts: true,
        system_break: false,
        source_span: crate::error::Span::new(0, 0),
    }
}

#[test]
fn measure_column_weights_gives_multi_measure_rest_uniform_weight() {
    // Mirrors the real shape built by `merge_rest_run`: the rest's own span
    // (columns `0..MULTI_MEASURE_REST_WIDTH`) plus one trailing `BarLine`
    // column at `MULTI_MEASURE_REST_WIDTH`. A small count's label ("4") is
    // far narrower than the `MULTI_MEASURE_REST_WIDTH`-note-glyphs floor, so
    // the span's total weight stays at that floor (plus the fixed
    // configured notes-padding clearance reserved on both ends — see
    // `multi_measure_rest_weight`), spread evenly across its columns; the
    // trailing bar-line column keeps its normal thin weight.
    let config = test_config();
    let block = make_multi_measure_rest_block("S", MULTI_MEASURE_REST_WIDTH, 4);
    let col_count = MULTI_MEASURE_REST_WIDTH + 1;
    let weights = measure_column_weights(&block, col_count, &config);
    let per_column = (notehead_weight(&config) * MULTI_MEASURE_REST_WIDTH as f32
        + config.notes_horizontal_padding_pt() * 2.0)
        / MULTI_MEASURE_REST_WIDTH as f32;
    let mut expected = vec![per_column; MULTI_MEASURE_REST_WIDTH as usize];
    expected.push(0.25);
    assert_eq!(
        weights, expected,
        "rest span columns should stay uniform, but the trailing bar-line \
         column should keep its normal thin weight instead of ballooning to \
         match a full rest column"
    );
}

#[test]
fn measure_column_weights_grows_multi_measure_rest_weight_with_a_wide_count_label() {
    // A run merging enough measures to need a many-digit count no longer
    // stays pinned at the small-count floor (`MULTI_MEASURE_REST_WIDTH`
    // note glyphs' worth of bar) once the label itself needs more than
    // that — its total weight (and so its per-column share) grows to match
    // the label's own real rendered width plus the same fixed end-padding
    // every count keeps, mirroring `multi_measure_rest_weight`.
    let config = test_config();
    let count = 123_456_789;
    let block = make_multi_measure_rest_block("S", MULTI_MEASURE_REST_WIDTH, count);
    let col_count = MULTI_MEASURE_REST_WIDTH + 1;
    let weights = measure_column_weights(&block, col_count, &config);
    let bar_span_weight: f32 = weights[..MULTI_MEASURE_REST_WIDTH as usize].iter().sum();
    let label_width =
        font_metrics::monospace_text_width(&count.to_string(), config.notes_font_size())
            + config.notes_horizontal_padding_pt() * 2.0;
    assert!(
        (bar_span_weight - label_width).abs() < 0.01,
        "a wide count label's total column weight ({bar_span_weight}) should \
         equal its own real rendered width plus end-padding ({label_width}) \
         once that exceeds the small-count floor"
    );
}

#[test]
fn multi_measure_rest_block_renders_wide_enough_for_its_count_label_when_squeezed_by_dense_neighbors(
) {
    // Regression test: a merged rest's own weight/rod used to be flat
    // constants independent of its count label's rendered width, so
    // squeezing it between dense measures (as here) could shrink its
    // rendered width far below what a multi-digit count needed, letting the
    // label overflow past the bar it sits on (see `multi_measure_rest_weight`).
    let config = test_config();
    let count = 120;
    let system = vec![
        make_block_with_notes("S", 16, 16),
        make_multi_measure_rest_block("S", MULTI_MEASURE_REST_WIDTH, count),
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
    let geometry = row.column_geometry(300.0, 40.0);

    let rest = &measure_layout[1];
    let rest_width = geometry.x_start((rest.start_col + rest.col_count) as f32)
        - geometry.x_start(rest.start_col as f32);
    let label_width =
        font_metrics::monospace_text_width(&count.to_string(), config.notes_font_size());

    assert!(
        rest_width >= label_width,
        "merged rest block width ({rest_width}) should be at least as wide \
         as its own count label ({label_width}) even when squeezed by dense \
         neighboring measures"
    );
}

#[test]
fn multi_measure_rest_block_keeps_horizontal_padding_even_when_squeezed_by_dense_neighbors() {
    // Regression test: even once `multi_measure_rest_block_renders_wide_enough_for_its_count_label_...`
    // holds, the bar's own drawn ink (`resolve_multi_measure_rest` insets it
    // by the configured notes padding on both ends) must stay clear of its bar's
    // *column region* by that same margin, or a tightly squeezed run renders
    // with its end ticks flush against the enclosing measure dividers.
    let config = test_config();
    let count = 120;
    let system = vec![
        make_block_with_notes("S", 16, 16),
        make_multi_measure_rest_block("S", MULTI_MEASURE_REST_WIDTH, count),
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
    let geometry = row.column_geometry(300.0, 40.0);

    let rest = &measure_layout[1];
    let rest_width = geometry.x_start((rest.start_col + rest.col_count) as f32)
        - geometry.x_start(rest.start_col as f32);
    let label_width =
        font_metrics::monospace_text_width(&count.to_string(), config.notes_font_size());
    let padding = config.notes_horizontal_padding_pt();

    assert!(
        rest_width >= label_width + padding * 2.0,
        "merged rest block width ({rest_width}) should reserve \
         {padding} points of clearance on both ends of its label \
         ({label_width}) even when squeezed by dense neighboring measures"
    );
}
