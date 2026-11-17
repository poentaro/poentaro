// ── Letter-name view column widths ────────────────────────────────────────────

use crate::ast::parsed::{Accidental, NumberedPitch, Offset};
use crate::compiler::types::{
    ColumnElement, ElementContent, HeadLabelPlacement, MeasureBlock, MeasureRow, NoteHeadLabel,
    RowId,
};
use crate::font_metrics;
use crate::grid_layout::layout::{build_measure_column_layout, measure_column_weights};
use crate::render_config::RenderConfig;

fn test_config() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        note_number_width: 12,
        part_label_width_pt: 40,
        max_measures_per_system: 48,
        notes_font_size: 18,
        directive_row_offset: Offset::default(),
        notes_horizontal_padding_pt: 4,
        ..Default::default()
    }
}

fn in_place_of_degree(text: &str) -> Option<NoteHeadLabel> {
    Some(NoteHeadLabel::playable(text.to_string()))
}

fn after_degree(text: &str) -> Option<NoteHeadLabel> {
    Some(NoteHeadLabel {
        placement: HeadLabelPlacement::AfterDegree,
        ..NoteHeadLabel::playable(text.to_string())
    })
}

fn block_with_note(head_label: Option<NoteHeadLabel>, accidental: Accidental) -> MeasureBlock {
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
                        pitch: NumberedPitch::Three,
                        accidental,
                        octave: 0,
                        dotted: false,
                        double_dotted: false,
                        head_label,
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
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
        system_break: false,
        source_span: crate::error::Span::new(0, 0),
    }
}

fn notes_text_width(text: &str, config: &RenderConfig) -> f32 {
    font_metrics::text_width_for_family(
        config.glyph_font_families.notes,
        text,
        config.notes_font_size(),
    )
}

#[test]
fn a_letter_name_s_column_is_measured_from_the_letter_name_itself() {
    let config = test_config();
    let digit = measure_column_weights(&block_with_note(None, Accidental::Natural), 2, &config)[0];
    let letter = measure_column_weights(
        &block_with_note(in_place_of_degree("F\u{266F}"), Accidental::Natural),
        2,
        &config,
    )[0];

    assert_eq!(letter, notes_text_width("F\u{266F}", &config));
    assert!(
        letter > digit,
        "F♯ ({letter}) should be wider than a lone digit ({digit})"
    );
}

#[test]
fn a_letter_name_reserves_no_accidental_lead_since_its_accidental_trails() {
    // A written `4#` in C reads `F♯`: the sharp follows the letter, so the
    // column needs no room ahead of the glyph, unlike the numbered `♯4`.
    let config = test_config();
    let layout = build_measure_column_layout(
        &[block_with_note(
            in_place_of_degree("F\u{266F}"),
            Accidental::Sharp,
        )],
        &config,
    );

    assert!(layout[0]
        .column_accidental_leads
        .iter()
        .all(|&lead| lead == 0.0));
}

#[test]
fn a_guitar_view_note_head_s_column_is_measured_from_its_digit_and_letter_name() {
    let config = test_config();
    let weight = measure_column_weights(
        &block_with_note(after_degree("E"), Accidental::Natural),
        2,
        &config,
    )[0];

    assert_eq!(weight, notes_text_width("3 (E)", &config));
}

#[test]
fn a_guitar_view_note_head_still_leads_with_its_digit_s_accidental() {
    // A written `3b` in C reads `♭3 (E♭)`: the flat still leads the digit,
    // so the column reserves room ahead of the glyph as in the numbered view.
    let config = test_config();
    let layout = build_measure_column_layout(
        &[block_with_note(after_degree("E\u{266D}"), Accidental::Flat)],
        &config,
    );
    // Skip the system's leading placeholder column (see
    // `build_measure_column_layout`), which never carries a lead.
    let note_col = layout[0].column_rods.len() - 2;

    assert!(layout[0].column_accidental_leads[note_col] > 0.0);
}
