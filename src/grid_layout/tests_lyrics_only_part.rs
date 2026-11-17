use crate::ast::parsed::{NumberedPitch, Offset};
use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::font_metrics;
use crate::grid_layout::layout::{block_column_width, build_measure_column_layout};
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

fn notes_row(note_count: u32, bar_col: u32) -> MeasureRow {
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
    MeasureRow {
        kind: crate::ast::parsed::PartKind::Notes,
        absorbed_rows: Vec::new(),
        id: RowId("notes".to_string()),
        label: "M".to_string(),
        elements,
        source_part_index: 0,
    }
}

fn lyric_line_row(text: &str) -> MeasureRow {
    MeasureRow {
        kind: crate::ast::parsed::PartKind::Notes,
        absorbed_rows: Vec::new(),
        id: RowId("lyrics".to_string()),
        label: "C".to_string(),
        elements: vec![
            ColumnElement {
                column: 0,
                content: ElementContent::LyricLine {
                    text: text.to_string(),
                },
                note_id: None,
            },
            ColumnElement {
                column: 0,
                content: ElementContent::BarLine,
                note_id: None,
            },
        ],
        source_part_index: 1,
    }
}

fn make_block(rows: Vec<MeasureRow>) -> MeasureBlock {
    MeasureBlock {
        rows,
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
        system_break: false,
        source_span: crate::error::Span::new(0, 0),
    }
}

#[test]
fn block_column_width_uses_max_barline_column_across_rows() {
    // A `lyrics`-only row's `BarLine` always sits at column 0, regardless of
    // how many columns a sibling `notes` row needs — `block_column_width`
    // must not shrink the block to match whichever row happens to come first.
    let lyrics_first = make_block(vec![lyric_line_row("hi"), notes_row(4, 4)]);
    assert_eq!(block_column_width(&lyrics_first), 5);

    let notes_first = make_block(vec![notes_row(4, 4), lyric_line_row("hi")]);
    assert_eq!(block_column_width(&notes_first), 5);
}

#[test]
fn lyric_line_widens_measure_past_short_text() {
    let config = test_config();
    let short = make_block(vec![lyric_line_row("hi")]);
    let long = make_block(vec![lyric_line_row("this is a much longer caption line")]);
    let short_layout = &build_measure_column_layout(&[short], &config)[0];
    let long_layout = &build_measure_column_layout(&[long], &config)[0];
    assert!(
        long_layout.weight > short_layout.weight,
        "long lyric line ({}) should out-weigh short one ({})",
        long_layout.weight,
        short_layout.weight
    );
}

#[test]
fn measure_weight_is_max_of_notes_and_lyrics() {
    let config = test_config();
    // Few notes, but a very long caption: the measure's width should be
    // driven by the lyric text, not collapse to the sparse note count.
    let mixed = make_block(vec![
        notes_row(1, 1),
        lyric_line_row("a very long caption that should widen this sparse measure"),
    ]);
    let notes_only = make_block(vec![notes_row(1, 1)]);
    let mixed_layout = &build_measure_column_layout(&[mixed], &config)[0];
    let notes_only_layout = &build_measure_column_layout(&[notes_only], &config)[0];
    assert!(mixed_layout.weight > notes_only_layout.weight);

    // Sanity: the lyric weight really is real text width, not a flat constant.
    let note_glyph_weight =
        font_metrics::monospace_char_advance_width('0', config.notes_font_size());
    assert!(mixed_layout.weight > note_glyph_weight * 2.0);
}
