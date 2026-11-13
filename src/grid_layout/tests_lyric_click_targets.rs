use crate::ast::parsed::{Accidental, NumberedPitch, Offset};
use crate::compiler::types::{
    ColumnElement, CompileResult, ElementContent, MeasureBlock, MeasureRow, RowId,
};
use crate::grid_layout::types::Header;
use crate::render_config::RenderConfig;

pub(super) fn no_header() -> Header {
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

pub(super) fn test_render_config() -> RenderConfig {
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

/// A `notes+lyrics` part's notes row: one two-beat note (a note head at
/// column 0 followed by its own dash-continuation column at column 1) then
/// a plain one-beat note at column 2, followed by a bar line — sharing
/// `source_part_index` with `lyric_verse_row`.
fn notes_row_with_a_two_beat_note() -> MeasureRow {
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
                content: ElementContent::NoteDash {
                    dotted: false,
                    double_dotted: false,
                },
                note_id: Some(0),
            },
            ColumnElement {
                column: 2,
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
                column: 3,
                content: ElementContent::BarLine,
                note_id: None,
            },
        ],
        source_part_index: 0,
    }
}

/// The sibling lyric-verse row that follows `notes_row_with_a_two_beat_note`
/// — one syllable per note, each placed at its note's own start column (see
/// `ElementContent::Lyric`'s doc comment for why lyrics live in their own row
/// rather than the notes row's own elements).
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
                    text: "Hoo".to_string(),
                    note_id: 0,
                },
                note_id: None,
            },
            ColumnElement {
                column: 2,
                content: ElementContent::Lyric {
                    text: "ray".to_string(),
                    note_id: 1,
                },
                note_id: None,
            },
            ColumnElement {
                column: 3,
                content: ElementContent::BarLine,
                note_id: None,
            },
        ],
        source_part_index: 1,
    }
}

/// A lyric syllable's own hover/click box must span its whole note's written
/// duration, not just the single column its text is drawn in — otherwise a
/// two-beat note's syllable is only hoverable/selectable over its first
/// beat. Regression test for the bug where `column_end` was always
/// `column_start + 1.0` regardless of the note's dash-continuation columns.
#[test]
fn lyric_click_target_spans_its_note_full_column_width() {
    let block = MeasureBlock {
        rows: vec![notes_row_with_a_two_beat_note(), lyric_verse_row()],
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

    let targets = &pages[0].lyric_click_targets;
    assert_eq!(targets.len(), 2, "one click target per syllable");

    let two_beat = targets
        .iter()
        .find(|t| t.note_id == 0)
        .expect("syllable for the two-beat note");
    // This is also the measure's first syllable, so its left edge snaps to
    // the system's leading bar line (`LABEL_COLS as f32` = 1.0 — see
    // `lyric_click_target_of_first_syllable_snaps_to_leading_bar_line`),
    // one column earlier than its raw grid column (2.0). Its `column_end`
    // is unaffected by that snap, so the span is 1.0 wider than the note's
    // own two written columns (attack + dash-continuation) would otherwise
    // give.
    assert_eq!(
        two_beat.column_end - two_beat.column_start,
        3.0,
        "the two-beat note's syllable box should span both of its columns \
         (attack + dash-continuation) plus the leading bar-line snap, not \
         just the attack column"
    );

    let one_beat = targets
        .iter()
        .find(|t| t.note_id == 1)
        .expect("syllable for the one-beat note");
    // This is also the measure's last (and only measure's) last syllable,
    // so its right edge snaps flush to the system's trailing bar line (see
    // `lyric_click_target_of_last_syllable_snaps_to_trailing_bar_line`),
    // one column past its raw `max_col + 1`.
    assert_eq!(
        one_beat.column_end - one_beat.column_start,
        2.0,
        "a plain one-beat note's syllable box should span its own column \
         plus the trailing bar-line snap"
    );
}

/// The first syllable of a system's first measure should have its click
/// box's left edge snapped to the system's leading bar line (`column_start
/// == LABEL_COLS as f32 == 1.0`), exactly like `compute_all_playback_cursor_targets`
/// snaps the first note's own click target in
/// `playback_cursor_targets_snap_to_bar_lines_at_measure_edges`. Without that
/// snap, the syllable's box starts at the raw `MUSIC_START_COL` grid column
/// (2.0) — one whole column short of the bar line — leaving a visible gap
/// between the bar line and the hover box's left edge.
#[test]
fn lyric_click_target_of_first_syllable_snaps_to_leading_bar_line() {
    let block = MeasureBlock {
        rows: vec![notes_row_with_a_two_beat_note(), lyric_verse_row()],
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

    let targets = &pages[0].lyric_click_targets;
    let first_syllable = targets
        .iter()
        .find(|t| t.note_id == 0)
        .expect("syllable for the measure's first note");
    assert_eq!(
        first_syllable.column_start, 1.0,
        "the first syllable's left edge should snap to the system's leading \
         bar line, matching the first note's own click target"
    );
}

/// The `notes_row_with_a_two_beat_note`/`lyric_verse_row` pair, but with
/// `note_id`s shifted by 2 so a second copy can share a system with the
/// first without colliding note IDs.
fn notes_row_with_a_two_beat_note_shifted() -> MeasureRow {
    let mut row = notes_row_with_a_two_beat_note();
    for el in &mut row.elements {
        if let Some(id) = el.note_id.as_mut() {
            *id += 2;
        }
    }
    row
}

fn lyric_verse_row_shifted() -> MeasureRow {
    let mut row = lyric_verse_row();
    for el in &mut row.elements {
        if let ElementContent::Lyric { note_id, .. } = &mut el.content {
            *note_id += 2;
        }
    }
    row
}

/// The last syllable of a system's *last* measure should have its click
/// box's right edge snapped flush to the system's trailing bar line
/// (`column_end == bar_line_col + 1.0`), while the same-shaped last syllable
/// of an *inter-measure* block only snaps to the bar line's own centered x
/// (`bar_line_col + 0.5`) — mirroring exactly how
/// `compute_all_playback_cursor_targets` treats a measure's last note in
/// `playback_cursor_targets_snap_to_bar_lines_at_measure_edges`. Without this
/// snap, a last syllable's box stops a whole column short of (or past) where
/// the bar line is actually drawn.
#[test]
fn lyric_click_target_of_last_syllable_snaps_to_trailing_bar_line() {
    let block1 = MeasureBlock {
        rows: vec![notes_row_with_a_two_beat_note(), lyric_verse_row()],
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
        system_break: false,
        source_span: crate::error::Span::new(0, 0),
    };
    let block2 = MeasureBlock {
        rows: vec![
            notes_row_with_a_two_beat_note_shifted(),
            lyric_verse_row_shifted(),
        ],
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
        system_break: false,
        source_span: crate::error::Span::new(0, 0),
    };

    let pages = crate::grid_layout::layout(
        &CompileResult {
            blocks: vec![block1, block2],
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

    let targets = &pages[0].lyric_click_targets;

    let inter_measure_last = targets
        .iter()
        .find(|t| t.note_id == 1)
        .expect("syllable for the first measure's last note");
    let system_last = targets
        .iter()
        .find(|t| t.note_id == 3)
        .expect("syllable for the system's last note");

    assert_eq!(
        inter_measure_last.column_end.fract(),
        0.5,
        "an inter-measure bar line is drawn centered on its column, so the \
         preceding syllable's right edge should snap to its half-column x, \
         not the next whole column"
    );
    assert_eq!(
        system_last.column_end.fract(),
        0.0,
        "the system's closing bar line is drawn flush right at a whole \
         column, so the last syllable's right edge should snap there \
         exactly, not to a half-column x"
    );
}

/// The compile-time link from `lyric_verse_row` to its notes row.
fn verse_link() -> crate::compiler::types::LyricLink {
    crate::compiler::types::LyricLink {
        lyric_row_id: RowId("V-lyrics-0".to_string()),
        target_row_id: RowId("V".to_string()),
    }
}
