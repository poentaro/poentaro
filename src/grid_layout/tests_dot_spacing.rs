// ── Augmentation-dot column-width reservation ─────────────────────────────

use crate::ast::parsed::{Accidental, NumberedPitch, Offset};
use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::font_metrics;
use crate::grid_layout::layout::measure_column_weights;
use crate::render_config::RenderConfig;

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

/// Real rendered width of a single notehead/rest digit glyph at `config`'s
/// notes font size, mirroring `layout_spacing::note_glyph_weight`.
fn notehead_weight(config: &RenderConfig) -> f32 {
    font_metrics::monospace_char_advance_width('0', config.notes_font_size())
}

/// Real rendered width of the note-dash glyph at `config`'s note-dash font
/// size, mirroring `layout_spacing::dash_weight`.
fn dash_weight(config: &RenderConfig) -> f32 {
    font_metrics::monospace_char_advance_width('\u{2014}', config.note_dash_font_size())
}

/// Independently recomputed expected extra weight of a dotted note/rest/
/// dash's augmentation dot(s) — mirroring what `render_note_head`/
/// `render_rest`/`render_note_dash` now actually draw: the dot(s) appended
/// directly onto the glyph's own text run (see
/// `font_metrics::augmentation_dot_suffix`), so their real rendered width at
/// `dot_font_size` is exactly the extra room needed. Deliberately not just
/// calling `font_metrics::augmentation_dot_suffix`/`monospace_text_width`
/// back, so this test can't pass merely because production and test share a
/// bug.
fn expected_dot_extra_weight(dot_count: u32, dot_font_size: f32) -> f32 {
    let dot_advance = font_metrics::monospace_char_advance_width('\u{b7}', dot_font_size);
    dot_count as f32 * dot_advance
}

fn make_block(row_id: &str, content: ElementContent, bar_col: u32) -> MeasureBlock {
    MeasureBlock {
        rows: vec![MeasureRow {
            kind: crate::ast::parsed::PartKind::Notes,
            absorbed_rows: Vec::new(),
            id: RowId(row_id.to_string()),
            label: row_id.to_string(),
            elements: vec![
                ColumnElement {
                    column: 0,
                    content,
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

fn note_head(dotted: bool, double_dotted: bool) -> ElementContent {
    ElementContent::NoteHead {
        pitch: NumberedPitch::One,
        accidental: Accidental::Natural,
        octave: 0,
        dotted,
        double_dotted,
        head_label: None,
    }
}

fn rest(dotted: bool, double_dotted: bool) -> ElementContent {
    ElementContent::Rest {
        dotted,
        double_dotted,
        implicit_fill: false,
    }
}

fn note_dash(dotted: bool, double_dotted: bool) -> ElementContent {
    ElementContent::NoteDash {
        dotted,
        double_dotted,
    }
}

#[test]
fn dotted_note_head_column_reserves_exactly_its_dot_s_own_rendered_width() {
    let config = test_config();
    let plain = make_block("S", note_head(false, false), 1);
    let dotted = make_block("S", note_head(true, false), 1);
    let double_dotted = make_block("S", note_head(true, true), 1);

    let plain_weight = measure_column_weights(&plain, 2, &config)[0];
    let dotted_weight = measure_column_weights(&dotted, 2, &config)[0];
    let double_dotted_weight = measure_column_weights(&double_dotted, 2, &config)[0];

    assert_eq!(plain_weight, notehead_weight(&config));
    assert!(
        (dotted_weight
            - (notehead_weight(&config) + expected_dot_extra_weight(1, config.notes_font_size())))
        .abs()
            < 0.001,
        "dotted_weight={dotted_weight}"
    );
    assert!(
        (double_dotted_weight
            - (notehead_weight(&config) + expected_dot_extra_weight(2, config.notes_font_size())))
        .abs()
            < 0.001,
        "double_dotted_weight={double_dotted_weight}"
    );
    assert!(
        double_dotted_weight > dotted_weight,
        "double_dotted_weight={double_dotted_weight} should be > dotted_weight={dotted_weight}"
    );
}

#[test]
fn dotted_rest_column_reserves_exactly_its_dot_s_own_rendered_width() {
    let config = test_config();
    let dotted = make_block("S", rest(true, false), 1);
    let dotted_weight = measure_column_weights(&dotted, 2, &config)[0];

    let expected =
        notehead_weight(&config) + expected_dot_extra_weight(1, config.notes_font_size());
    assert!(
        (dotted_weight - expected).abs() < 0.001,
        "dotted_weight={dotted_weight}"
    );
}

#[test]
fn dotted_note_dash_column_reserves_its_dot_s_own_rendered_width_at_note_dash_font_size() {
    // `render_note_dash` draws its dot(s) at `config`'s note-dash font size
    // (not `notes_font_size` — see `Metadata::note_dash.font_size`), so the
    // dash's own extra weight must be measured at that size to match what's
    // actually drawn. notes/note_dash are given distinct sizes here so this
    // can't pass by coincidence.
    let config = RenderConfig {
        notes_font_size: 18,
        note_dash_font_size: 30,
        ..test_config()
    };
    let dotted = make_block("S", note_dash(true, false), 1);
    let dotted_weight = measure_column_weights(&dotted, 2, &config)[0];

    let expected =
        dash_weight(&config) + expected_dot_extra_weight(1, config.note_dash_font_size());
    assert!(
        (dotted_weight - expected).abs() < 0.001,
        "dotted_weight={dotted_weight}"
    );
}
