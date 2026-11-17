use crate::compiler::types::MeasureBlock;
use crate::grid_layout::types::Header;

use super::simple_block;

#[test]
fn erroneous_measure_produces_error_highlight() {
    use crate::error::{Diagnostic, Span, Warning};

    let erroneous_block = MeasureBlock {
        rows: simple_block(4).rows,
        decorations: vec![],
        diagnostics: vec![Diagnostic::Warning(Warning::new(
            Span::new(0, 1),
            "lyrics underflow",
        ))],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
        system_break: false,
        source_span: Span::new(0, 0),
    };
    let header = Header {
        title: Some("T".into()),
        subtitle: None,
        author: Some("A".into()),
        part_list: vec![],
        parts_list_columns: 3,
        sequence: None,
        title_font_size: 36.0,
        subtitle_font_size: 19.0,
        author_font_size: 14.0,
        sequence_font_size: 12.0,
        part_legend_font_size: 12.0,
        ..Default::default()
    };
    let config = crate::render_config::RenderConfig {
        row_height: 24,
        max_measures_per_system: 28,
        note_number_width: 8,
        part_label_width_pt: 40,
        lyrics_font_size: 14,
        notes_font_size: 14,
        note_dash_font_size: 14,
        chords_font_size: 14,
        hide_system_dividers: false,
        directive_row_offset: crate::ast::parsed::Offset::default(),
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
    };
    let pages = crate::grid_layout::layout(
        &crate::compiler::types::CompileResult {
            blocks: vec![erroneous_block],
            slur_spans: vec![],
            tuplet_spans: vec![],
            lyric_links: vec![],
        },
        &config,
        &header,
        crate::grid_layout::PageLayout::A4,
        None,
    )
    .pages;
    assert!(!pages.is_empty());
    assert_eq!(
        pages[0].error_highlights.len(),
        1,
        "erroneous measure should produce one error highlight"
    );
}

#[test]
fn click_target_row_start_skips_hidden_system_divider() {
    // Two systems (forced apart via max_measures_per_system = 1), each a
    // single-row 4-note block, so each system contributes 6 musical rows
    // (system_musical_row_count: 4 notes -> 6 sub-rows, no lyrics).
    //
    // With hide_system_dividers: false, build_page_rows inserts a separator
    // row between the two systems, so the second system's click target
    // should start 1 row later than when the divider is hidden and no such
    // row exists.
    let header = super::no_header();
    let blocks = vec![simple_block(4), simple_block(4)];

    let shown_config = crate::render_config::RenderConfig {
        row_height: 24,
        max_measures_per_system: 1,
        note_number_width: 8,
        part_label_width_pt: 40,
        lyrics_font_size: 14,
        notes_font_size: 14,
        note_dash_font_size: 14,
        chords_font_size: 14,
        hide_system_dividers: false,
        directive_row_offset: crate::ast::parsed::Offset::default(),
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
    };
    let shown_pages = crate::grid_layout::layout(
        &crate::compiler::types::CompileResult {
            blocks: blocks.clone(),
            slur_spans: vec![],
            tuplet_spans: vec![],
            lyric_links: vec![],
        },
        &shown_config,
        &header,
        crate::grid_layout::PageLayout::A4,
        None,
    )
    .pages;
    let hidden_config = crate::render_config::RenderConfig {
        hide_system_dividers: true,
        ..shown_config
    };
    let hidden_pages = crate::grid_layout::layout(
        &crate::compiler::types::CompileResult {
            blocks,
            slur_spans: vec![],
            tuplet_spans: vec![],
            lyric_links: vec![],
        },
        &hidden_config,
        &header,
        crate::grid_layout::PageLayout::A4,
        None,
    )
    .pages;

    let shown_second_row_start = shown_pages[0].measure_click_targets[1].row_start;
    let hidden_second_row_start = hidden_pages[0].measure_click_targets[1].row_start;

    assert_eq!(
        hidden_second_row_start,
        shown_second_row_start - 1,
        "when system dividers are hidden, the second system's rows shift up \
         by one (no separator row is rendered), so its click target's \
         row_start must shift up by one too instead of staying put"
    );
}

#[test]
fn non_erroneous_measure_produces_no_error_highlight() {
    let block = simple_block(4);
    let header = Header {
        title: Some("T".into()),
        subtitle: None,
        author: Some("A".into()),
        part_list: vec![],
        parts_list_columns: 3,
        sequence: None,
        title_font_size: 36.0,
        subtitle_font_size: 19.0,
        author_font_size: 14.0,
        sequence_font_size: 12.0,
        part_legend_font_size: 12.0,
        ..Default::default()
    };
    let config = crate::render_config::RenderConfig {
        row_height: 24,
        max_measures_per_system: 28,
        note_number_width: 8,
        part_label_width_pt: 40,
        lyrics_font_size: 14,
        notes_font_size: 14,
        note_dash_font_size: 14,
        chords_font_size: 14,
        hide_system_dividers: false,
        directive_row_offset: crate::ast::parsed::Offset::default(),
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
    };
    let pages = crate::grid_layout::layout(
        &crate::compiler::types::CompileResult {
            blocks: vec![block],
            slur_spans: vec![],
            tuplet_spans: vec![],
            lyric_links: vec![],
        },
        &config,
        &header,
        crate::grid_layout::PageLayout::A4,
        None,
    )
    .pages;
    assert!(!pages.is_empty());
    assert!(pages[0].error_highlights.is_empty());
}
