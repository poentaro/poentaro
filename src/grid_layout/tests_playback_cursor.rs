use crate::ast::parsed::{Accidental, NumberedPitch, Offset};
use crate::compiler::types::{
    ColumnElement, CompileResult, ElementContent, MeasureBlock, MeasureRow, RowId,
};
use crate::grid_layout::types::Header;
use crate::render_config::RenderConfig;

fn no_header() -> Header {
    Header {
        title: None,
        subtitle: None,
        author: None,
        part_list: vec![],
        parts_list_columns: 3,
        sequence: None,
        title_font_size: 36.0,
        subtitle_font_size: 19.0,
        author_font_size: 14.0,
        sequence_font_size: 12.0,
        part_legend_font_size: 12.0,
        ..Default::default()
    }
}

/// A `notes+lyrics` part's notes row: two plain notes followed by a bar
/// line, sharing `source_part_index` with `lyric_verse_row`.
fn notes_row_with_two_notes() -> MeasureRow {
    MeasureRow {
        kind: crate::ast::parsed::PartKind::Notes,
        absorbed_rows: Vec::new(),
        id: RowId("V".to_string()),
        label: String::new(),
        elements: vec![
            ColumnElement {
                column: 0,
                content: ElementContent::NoteHead {
                    pitch: NumberedPitch::One,
                    accidental: Accidental::Natural,
                    octave: 0,
                    dotted: false,
                    double_dotted: false,
                    head_label: None,
                },
                note_id: Some(0),
            },
            ColumnElement {
                column: 1,
                content: ElementContent::NoteHead {
                    pitch: NumberedPitch::Two,
                    accidental: Accidental::Natural,
                    octave: 0,
                    dotted: false,
                    double_dotted: false,
                    head_label: None,
                },
                note_id: Some(1),
            },
            ColumnElement {
                column: 2,
                content: ElementContent::BarLine,
                note_id: None,
            },
        ],
        source_part_index: 0,
    }
}

/// The sibling lyric-verse row that follows `notes_row_with_two_notes` —
/// see `ElementContent::Lyric`'s doc comment for why lyrics live in their
/// own row rather than the notes row's own elements.
fn lyric_verse_row() -> MeasureRow {
    MeasureRow {
        kind: crate::ast::parsed::PartKind::Lyrics {
            target_part_index: 0,
        },
        absorbed_rows: Vec::new(),
        id: RowId("V-lyrics-0".to_string()),
        label: String::new(),
        elements: vec![
            ColumnElement {
                column: 0,
                content: ElementContent::Lyric {
                    text: "Ho".to_string(),
                    note_id: 0,
                },
                note_id: None,
            },
            ColumnElement {
                column: 1,
                content: ElementContent::Lyric {
                    text: "Ho".to_string(),
                    note_id: 0,
                },
                note_id: None,
            },
            ColumnElement {
                column: 2,
                content: ElementContent::BarLine,
                note_id: None,
            },
        ],
        source_part_index: 1,
    }
}

fn test_render_config() -> RenderConfig {
    RenderConfig {
        row_height: 24,
        max_measures_per_system: 28,
        note_number_width: 8,
        part_label_width_pt: 40,
        lyrics_font_size: 14,
        notes_font_size: 14,
        note_dash_font_size: 14,
        chords_font_size: 14,
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
fn playback_cursor_target_extends_over_its_lyric_verse_row() {
    // A `notes+lyrics` part compiles to two sibling `MeasureRow`s sharing
    // `source_part_index` — a notes row and a separate lyric-verse row (see
    // `ElementContent::Lyric`'s doc comment) — rather than lyrics being
    // mixed into the notes row's own elements. Each note's highlight target
    // must still reach down through its part's verse row(s) so the
    // highlight rect covers the lyric text too, not just the note glyph.
    let block = MeasureBlock {
        rows: vec![notes_row_with_two_notes(), lyric_verse_row()],
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
        system_break: false,
        source_span: crate::error::Span::new(0, 0),
    };

    let pages = crate::grid_layout::layout(
        &CompileResult {
            blocks: vec![block],
            slur_spans: vec![],
            tuplet_spans: vec![],
            lyric_links: vec![verse_link()],
        },
        &test_render_config(),
        &no_header(),
        crate::grid_layout::PageLayout::A4,
        None,
    )
    .pages;

    let targets = &pages[0].playback_cursor_targets;
    assert_eq!(
        targets.len(),
        2,
        "one highlight target per note, none for the lyric-only row itself"
    );
    for target in targets {
        assert_eq!(
            target.row_end - target.row_start,
            6,
            "note's highlight span should cover its 6 note sub-rows (no \
             tuplet in this block, so no tuplet_bracket sub-row) plus the 1 \
             lyric-verse row that follows them, not just the note sub-rows"
        );
    }
}

#[test]
fn note_click_target_does_not_extend_over_its_lyric_verse_row() {
    // Same fixture as `playback_cursor_target_extends_over_its_lyric_verse_row`,
    // but asserting the click/selection-target field (`click_row_end`)
    // instead of the playback-cursor field (`row_end`): a note's own
    // click/selection target must stop at its own note row, never reaching
    // into a following lyric-verse row — a lyric syllable is independently
    // selectable via its own `LyricClickTarget`. Regression test for the
    // bug where hovering/selecting a note with a lyric underneath drew a
    // selection box that also covered the lyric text.
    let block = MeasureBlock {
        rows: vec![notes_row_with_two_notes(), lyric_verse_row()],
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
        system_break: false,
        source_span: crate::error::Span::new(0, 0),
    };

    let pages = crate::grid_layout::layout(
        &CompileResult {
            blocks: vec![block],
            slur_spans: vec![],
            tuplet_spans: vec![],
            lyric_links: vec![verse_link()],
        },
        &test_render_config(),
        &no_header(),
        crate::grid_layout::PageLayout::A4,
        None,
    )
    .pages;

    let targets = &pages[0].playback_cursor_targets;
    assert_eq!(targets.len(), 2);
    for target in targets {
        assert_eq!(
            target.click_row_end - target.row_start,
            5,
            "note's click/selection span should cover only its 6 note \
             sub-rows (rows row_start..=row_start+5), not the lyric-verse \
             row that follows"
        );
        assert!(
            target.row_end > target.click_row_end,
            "playback-cursor row_end should still extend past click_row_end \
             to cover the lyric-verse row, so the two fields genuinely \
             differ for this fixture"
        );
    }
}

#[cfg(test)]
#[path = "tests_playback_cursor_bar_line_snapping.rs"]
mod tests_bar_line_snapping;

/// The compile-time link from `lyric_verse_row` to its notes row.
fn verse_link() -> crate::compiler::types::LyricLink {
    crate::compiler::types::LyricLink {
        lyric_row_id: RowId("V-lyrics-0".to_string()),
        target_row_id: RowId("V".to_string()),
    }
}
