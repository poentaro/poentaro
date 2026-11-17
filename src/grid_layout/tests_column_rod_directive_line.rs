// ── Directive-line rod clearance ──────────────────────────────────────────
//
// Split out of `tests_column_rod_spring.rs` (see its own header) — these
// regressions exercise the directive-line-specific rod floor added for
// PLAN-directive-width-column-spacing.md, distinct from that file's plain
// rod/padding invariants.

use crate::ast::parsed::{NumberedPitch, Offset};
use crate::compiler::types::{
    ColumnElement, Decoration, ElementContent, MeasureBlock, MeasureRow, RowId,
};
use crate::coordinate_resolver::LyricFontSizes;
use crate::grid_layout::expand::expand_system_to_rows;
use crate::grid_layout::layout::{
    build_measure_column_layout, directive_line_rod_width, LyricSizing,
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

#[test]
fn measure_rod_widens_to_fit_a_long_directive_line() {
    // Regression: a measure whose directive line (label/key/bpm/time) is
    // wider than its own musical content must still reserve enough rod for
    // that directive line, so the next measure's directive line doesn't
    // overlap it (see PLAN-directive-width-column-spacing.md).
    let config = test_config();
    let mut wide_directive_block = make_block_with_notes("S", 1, 1);
    let decoration = Decoration::DirectiveLine {
        label: Some("A Very Long Section Label".to_string()),
        bar_number: Some(1),
        key: Some("C Major".to_string()),
        bpm: Some(120),
        time_signature: Some((4, 4)),
    };
    wide_directive_block.decorations = vec![decoration.clone()];
    let ordinary_block = make_block_with_notes("S", 4, 4);
    let system = vec![wide_directive_block, ordinary_block];

    let expected_directive_width = directive_line_rod_width(
        &decoration,
        config.measure_number_font_size as f32,
        config.section_label_font_size as f32,
        config.section_label_bold,
    );
    let measure_layout = build_measure_column_layout(&system, &config);

    assert!(
        measure_layout[0].rod_pt >= expected_directive_width - 0.01,
        "first measure's rod_pt ({}) should be at least its directive \
         line's own rendered width ({expected_directive_width})",
        measure_layout[0].rod_pt
    );

    // The next measure's leading bar line (this block's own trailing bar
    // line) must land at or past the directive line's right edge.
    let label_width = 40.0_f32;
    let total_rod: f32 = measure_layout.iter().map(|m| m.rod_pt).sum();
    let usable_width = label_width + total_rod + 1.0;
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
    let geometry = rows[0].column_geometry(usable_width, label_width);
    let m0 = &measure_layout[0];
    let bar_line_col = (m0.start_col + m0.col_count - 1) as f32;
    // The next measure's directive line is anchored at (and starts drawing
    // from) this measure's own trailing bar-line column — its *left* edge,
    // not its right edge — so that's the bound that actually matters: the
    // first measure's directive line must finish before the bar-line
    // column's own left edge, not merely before its right edge.
    let measure_end_x = geometry.x_start(bar_line_col);
    let directive_line_start_x = geometry.x_start(m0.start_col as f32);
    assert!(
        measure_end_x - directive_line_start_x >= expected_directive_width - 0.01,
        "first measure's span before its trailing bar line \
         (x={directive_line_start_x}..{measure_end_x}) should be at least \
         as wide as its directive line ({expected_directive_width}), since \
         that's where the next measure's own directive line starts drawing"
    );
}

#[test]
fn two_adjacent_directive_lines_do_not_overlap() {
    // Regression: when a system's second measure ALSO carries a directive
    // line (not just the first), the first measure's directive text must
    // still fit before its own trailing bar-line column — which is exactly
    // where the second measure's directive line starts drawing (see
    // `layout_decoration::make_decoration_row`'s doc comment). The single-
    // directive case above can't catch this: a lone directive block's own
    // `rod_pt` floor is enough by itself, but the closed-form clearance for
    // the *trailing bar line's own rescaled share* only bites when solved
    // per-block, so a second, independently-directive-bearing block is
    // needed to exercise it.
    let config = test_config();
    let make_directive_block = |label: &str, bar_col: u32| {
        let mut block = make_block_with_notes("S", 1, bar_col);
        block.decorations = vec![Decoration::DirectiveLine {
            label: Some(label.to_string()),
            bar_number: None,
            key: None,
            bpm: None,
            time_signature: None,
        }];
        block
    };
    let block0 = make_directive_block("Verse 1 begins here with a long label", 1);
    let block1 = make_directive_block("Pre-Chorus transition also quite long", 1);
    let decoration0 = block0.decorations[0].clone();
    let system = vec![block0, block1];

    let expected_directive_width = directive_line_rod_width(
        &decoration0,
        config.measure_number_font_size as f32,
        config.section_label_font_size as f32,
        config.section_label_bold,
    );
    let measure_layout = build_measure_column_layout(&system, &config);
    let total_rod: f32 = measure_layout.iter().map(|m| m.rod_pt).sum();
    let label_width = 40.0_f32;
    let usable_width = label_width + total_rod + 1.0;
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
    let geometry = rows[0].column_geometry(usable_width, label_width);

    let m0 = &measure_layout[0];
    let bar_line_col = (m0.start_col + m0.col_count - 1) as f32;
    let block0_directive_end_x = geometry.x_start(m0.start_col as f32) + expected_directive_width;
    let block1_directive_start_x = geometry.x_start(bar_line_col);

    assert!(
        block0_directive_end_x <= block1_directive_start_x + 0.01,
        "block 0's directive line (ends at x={block0_directive_end_x}) must \
         not extend past where block 1's own directive line starts drawing \
         (x={block1_directive_start_x}), since block 1 also carries a \
         directive line anchored at block 0's trailing bar-line column"
    );
}
