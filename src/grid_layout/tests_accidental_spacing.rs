// ── Sharp/flat accidental column-width reservation ────────────────────────────

use crate::ast::parsed::{Accidental, NumberedPitch, Offset};
use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::font_metrics;
use crate::grid_layout::layout::{build_measure_column_layout, measure_column_weights};
use crate::grid_layout::types::GridRow;
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

/// Real rendered width of a single notehead digit glyph at `config`'s notes
/// font size, mirroring `layout_spacing::note_glyph_weight`.
fn notehead_weight(config: &RenderConfig) -> f32 {
    font_metrics::monospace_char_advance_width('0', config.notes_font_size())
}

/// Expected total column weight of a `NoteHead` carrying `symbol` (`"♯"` or
/// `"♭"`) as its accidental, mirroring `layout_spacing::accidental_extra_weight`:
/// the accidental now renders as part of the note digit's own text run, at
/// the same font size as the digit (see `render_note_head`), so its own real
/// rendered width at that size is exactly the extra room needed.
fn accidental_note_weight(symbol: &str, config: &RenderConfig) -> f32 {
    notehead_weight(config) + font_metrics::monospace_text_width(symbol, config.notes_font_size())
}

fn make_block_with_accidental_note(
    row_id: &str,
    accidental: Accidental,
    bar_col: u32,
) -> MeasureBlock {
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
                        accidental,
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
fn measure_column_weights_gives_sharp_or_flat_note_wider_weight_than_plain_note() {
    // A sharp/flat glyph renders as part of the note head's own text run
    // (see `render_note_head` in `glyph_renderers.rs`) and needs its own
    // reserved room, unlike a natural note which appends no accidental
    // character at all.
    let config = test_config();
    let plain = make_block_with_accidental_note("S", Accidental::Natural, 1);
    let sharp = make_block_with_accidental_note("S", Accidental::Sharp, 1);
    let flat = make_block_with_accidental_note("S", Accidental::Flat, 1);

    let plain_weight = measure_column_weights(&plain, 2, &config)[0];
    let sharp_weight = measure_column_weights(&sharp, 2, &config)[0];
    let flat_weight = measure_column_weights(&flat, 2, &config)[0];

    assert_eq!(plain_weight, notehead_weight(&config));
    assert!(
        sharp_weight > plain_weight,
        "sharp_weight={sharp_weight} should be > plain_weight={plain_weight}"
    );
    assert!(
        flat_weight > plain_weight,
        "flat_weight={flat_weight} should be > plain_weight={plain_weight}"
    );
    assert_eq!(sharp_weight, accidental_note_weight("\u{266F}", &config));
    assert_eq!(flat_weight, accidental_note_weight("\u{266D}", &config));
}

/// Real rendered width of `accidental`'s glyph at `config`'s notes font
/// size/family — the accidental lead `render_note_head` draws into.
fn accidental_width(accidental: &Accidental, config: &RenderConfig) -> f32 {
    font_metrics::accidental_width_for_family(
        config.glyph_font_families.notes,
        accidental,
        config.notes_font_size(),
    )
}

#[test]
fn a_sharp_note_s_column_gets_an_accidental_lead_included_in_its_rod() {
    // The sharp draws to the left of its digit, so its column reserves the
    // sharp's width as a lead ahead of every glyph, on top of the same rod a
    // plain note would need. A large notes font keeps even the plain
    // one-note measure's content rod above `MIN_MEASURE_WIDTH_PT`, so no
    // proportional rescale distorts the per-column rods compared here.
    let config = RenderConfig {
        notes_font_size: 40,
        ..test_config()
    };
    let plain = build_measure_column_layout(
        &[make_block_with_accidental_note("S", Accidental::Natural, 1)],
        &config,
    );
    let sharp = build_measure_column_layout(
        &[make_block_with_accidental_note("S", Accidental::Sharp, 1)],
        &config,
    );
    // Skip the system's leading placeholder column (see
    // `build_measure_column_layout`), which never carries a lead.
    let leading_extra = sharp[0].column_rods.len() - 2;
    let note_col = leading_extra;
    let sharp_width = accidental_width(&Accidental::Sharp, &config);

    assert_eq!(plain[0].column_accidental_leads[note_col], 0.0);
    assert_eq!(sharp[0].column_accidental_leads[note_col], sharp_width);
    assert!(
        (sharp[0].column_rods[note_col] - (plain[0].column_rods[note_col] + sharp_width)).abs()
            < 0.001
    );
    assert!(sharp[0].column_accidental_leads[..note_col]
        .iter()
        .chain(&sharp[0].column_accidental_leads[note_col + 1..])
        .all(|&lead| lead == 0.0));
}

#[test]
fn glyph_left_anchor_x_shifts_right_by_the_column_s_accidental_lead() {
    // Every glyph anchor in a sharp note's column (the digit itself, plus
    // anything else sharing the column, and the ties/underlines keyed off
    // it) moves right by the lead, so the sharp has room to its left and
    // the digit stays aligned with other parts' digits on the same beat.
    let config = test_config();
    let system = [make_block_with_accidental_note("S", Accidental::Sharp, 1)];
    let measure_layout = build_measure_column_layout(&system, &config);
    let start_col = measure_layout[0].start_col;
    let leading_extra = measure_layout[0].column_rods.len() - 2;
    let note_col = (start_col as usize + leading_extra) as f32;
    let row = GridRow {
        height_pt: 30.0,
        column_count: start_col + measure_layout[0].col_count,
        has_label_region: true,
        measure_layout,
        elements: Vec::new(),
    };
    let geometry = row.column_geometry(500.0, 40.0);
    let padding = 4.0;

    assert!(
        (geometry.glyph_left_anchor_x(note_col, padding)
            - (geometry.x_start(note_col)
                + accidental_width(&Accidental::Sharp, &config)
                + padding))
            .abs()
            < 0.001
    );
    let bar_col = note_col + 1.0;
    assert_eq!(
        geometry.glyph_left_anchor_x(bar_col, padding),
        geometry.x_start(bar_col) + padding
    );
}

fn make_block_with_chord(text: &str) -> MeasureBlock {
    MeasureBlock {
        rows: vec![MeasureRow {
            kind: crate::ast::parsed::PartKind::Notes,
            absorbed_rows: Vec::new(),
            id: RowId("C".to_string()),
            label: "C".to_string(),
            elements: vec![
                ColumnElement {
                    column: 0,
                    content: ElementContent::ChordSymbol {
                        text: text.to_string(),
                        degrees: crate::compiler::test_chord_degrees::tonic_major(),
                        guitar_voicing: None,
                        dotted: false,
                        double_dotted: false,
                    },
                    note_id: None,
                },
                ColumnElement {
                    column: 1,
                    content: ElementContent::BarLine,
                    note_id: None,
                },
            ],
            source_part_index: 0,
        }],
        ..make_block_with_accidental_note("S", Accidental::Natural, 1)
    }
}

#[test]
fn a_chord_symbol_s_leading_accidental_becomes_its_column_s_accidental_lead() {
    // A chord's root sharp/flat draws left of its degree (see
    // `render_chord_symbol`), so its column reserves that glyph's width as
    // a lead, exactly like a sharp note head. A bass-note accidental
    // (`1/♯4`) sits mid-text and needs no lead.
    let config = test_config();
    let leading = build_measure_column_layout(&[make_block_with_chord("\u{266F}1m")], &config);
    let bass_only = build_measure_column_layout(&[make_block_with_chord("1/\u{266F}4")], &config);
    let chord_col = leading[0].column_rods.len() - 2;
    let sharp_width = font_metrics::text_width_for_family(
        config.glyph_font_families.chords,
        "\u{266F}",
        config.chords_font_size(),
    );

    assert!(sharp_width > 0.0);
    assert_eq!(leading[0].column_accidental_leads[chord_col], sharp_width);
    assert_eq!(bass_only[0].column_accidental_leads[chord_col], 0.0);
}
