// ── `break` directive: forces a new system at the marked measure ────────────

use crate::ast::parsed::{NumberedPitch, Offset};
use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::grid_layout::layout::pack_into_systems;
use crate::render_config::RenderConfig;

fn make_block(row_id: &str, bar_col: u32, system_break: bool) -> MeasureBlock {
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
        system_break,
        source_span: crate::error::Span::new(0, 0),
    }
}

fn cfg(max_measures_per_system: u32) -> RenderConfig {
    RenderConfig {
        row_height: 30,
        note_number_width: 12,
        part_label_width_pt: 40,
        max_measures_per_system,
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
fn break_directive_forces_early_system_split() {
    // 4 measures, max_measures_per_system=4 (would otherwise fit in one
    // system), but the 3rd measure carries `break`.
    let blocks = vec![
        make_block("S", 3, false),
        make_block("S", 3, false),
        make_block("S", 3, true),
        make_block("S", 3, false),
    ];
    let systems = pack_into_systems(&blocks, &cfg(4), f32::INFINITY);
    assert_eq!(systems.len(), 2);
    assert_eq!(systems[0].len(), 2);
    assert_eq!(systems[1].len(), 2);
}

#[test]
fn no_break_directive_packs_purely_by_count() {
    let blocks = vec![
        make_block("S", 3, false),
        make_block("S", 3, false),
        make_block("S", 3, false),
        make_block("S", 3, false),
    ];
    let systems = pack_into_systems(&blocks, &cfg(4), f32::INFINITY);
    assert_eq!(systems.len(), 1);
    assert_eq!(systems[0].len(), 4);
}

#[test]
fn break_directive_on_first_measure_of_system_is_noop() {
    let blocks = vec![
        make_block("S", 3, true),
        make_block("S", 3, false),
        make_block("S", 3, false),
        make_block("S", 3, false),
    ];
    let systems = pack_into_systems(&blocks, &cfg(4), f32::INFINITY);
    assert_eq!(systems.len(), 1);
    assert_eq!(systems[0].len(), 4);
}

#[test]
fn multiple_break_directives_produce_multiple_early_splits() {
    let blocks = vec![
        make_block("S", 3, false),
        make_block("S", 3, false),
        make_block("S", 3, true),
        make_block("S", 3, false),
        make_block("S", 3, true),
        make_block("S", 3, false),
    ];
    let systems = pack_into_systems(&blocks, &cfg(6), f32::INFINITY);
    assert_eq!(systems.len(), 3);
    assert_eq!(systems[0].len(), 2);
    assert_eq!(systems[1].len(), 2);
    assert_eq!(systems[2].len(), 2);
}

#[test]
fn break_directive_interacts_with_max_measures_per_system() {
    // max_measures_per_system=2 already splits after every 2 measures; the
    // break on the 3rd measure lands exactly on an existing boundary, so it
    // doesn't change the split points.
    let blocks = vec![
        make_block("S", 3, false),
        make_block("S", 3, false),
        make_block("S", 3, true),
        make_block("S", 3, false),
        make_block("S", 3, false),
    ];
    let systems = pack_into_systems(&blocks, &cfg(2), f32::INFINITY);
    assert_eq!(systems.len(), 3);
    assert_eq!(systems[0].len(), 2);
    assert_eq!(systems[1].len(), 2);
    assert_eq!(systems[2].len(), 1);
}

#[test]
fn break_directive_does_not_persist_to_later_measures() {
    // Only the 3rd measure's block has system_break set; system packing after
    // it resumes purely by count, so measures 4-6 aren't forced apart.
    let blocks = vec![
        make_block("S", 3, false),
        make_block("S", 3, false),
        make_block("S", 3, true),
        make_block("S", 3, false),
        make_block("S", 3, false),
        make_block("S", 3, false),
    ];
    let systems = pack_into_systems(&blocks, &cfg(4), f32::INFINITY);
    assert_eq!(systems.len(), 2);
    assert_eq!(systems[0].len(), 2);
    assert_eq!(systems[1].len(), 4);
}

// ── Width: a system breaks early when the next measure wouldn't fit ─────────

/// Summed column rods of `block` laid out as a system of its own.
fn single_block_rod_pt(block: &MeasureBlock) -> f32 {
    crate::grid_layout::layout::build_measure_column_layout(std::slice::from_ref(block), &cfg(4))
        .iter()
        .map(|m| m.rod_pt)
        .sum()
}

#[test]
fn system_breaks_early_when_the_next_measure_would_overflow_the_width() {
    let blocks = vec![make_block("S", 3, false); 4];
    let width = 2.5 * single_block_rod_pt(&blocks[0]);
    let systems = pack_into_systems(&blocks, &cfg(4), width);
    assert_eq!(systems.iter().map(Vec::len).collect::<Vec<_>>(), [2, 2]);
}

#[test]
fn a_measure_wider_than_the_page_still_gets_a_system_of_its_own() {
    let blocks = vec![make_block("S", 3, false); 3];
    let width = 0.5 * single_block_rod_pt(&blocks[0]);
    let systems = pack_into_systems(&blocks, &cfg(4), width);
    assert_eq!(systems.iter().map(Vec::len).collect::<Vec<_>>(), [1, 1, 1]);
}
