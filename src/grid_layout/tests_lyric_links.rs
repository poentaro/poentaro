//! Grouping a note row with the lyric rows under it reads the score's
//! `lyric_links`, never anything stored on the rows themselves.

use crate::ast::parsed::{Accidental, NumberedPitch, PartKind};
use crate::compiler::types::{
    ColumnElement, ElementContent, LyricLink, MeasureBlock, MeasureRow, RowId,
};
use crate::grid_layout::layout::SystemBands;
use crate::grid_layout::playback_cursor::{note_row_spans, NoteRowSpan};

fn note_row(id: &str, source_part_index: usize) -> MeasureRow {
    MeasureRow {
        id: RowId(id.to_string()),
        label: id.to_string(),
        kind: PartKind::Notes,
        elements: vec![ColumnElement {
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
        }],
        source_part_index,
        absorbed_rows: Vec::new(),
    }
}

fn lyric_row(id: &str, source_part_index: usize) -> MeasureRow {
    MeasureRow {
        kind: PartKind::Lyrics {
            target_part_index: 0,
        },
        elements: Vec::new(),
        ..note_row(id, source_part_index)
    }
}

fn link(lyric: &str, target: &str) -> LyricLink {
    LyricLink {
        lyric_row_id: RowId(lyric.to_string()),
        target_row_id: RowId(target.to_string()),
    }
}

fn spans_of(rows: Vec<MeasureRow>, links: &[LyricLink]) -> Vec<NoteRowSpan> {
    let block = MeasureBlock {
        rows,
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
        system_break: false,
        source_span: crate::error::Span::new(0, 0),
    };
    note_row_spans(links, &[block], 0, &SystemBands::default())
}

fn bounds(spans: &[NoteRowSpan]) -> Vec<(usize, usize, usize)> {
    spans
        .iter()
        .map(|s| (s.row_start, s.playback_row_end, s.click_row_end))
        .collect()
}

#[test]
fn two_verses_under_one_target_are_absorbed_into_its_playback_span() {
    let spans = spans_of(
        vec![note_row("M", 0), lyric_row("V1", 1), lyric_row("V2", 2)],
        &[link("V1", "M"), link("V2", "M")],
    );

    // A note row is 6 rows tall: the playback span also covers both verses
    // (rows 6 and 7) while the click span stops at the note row's own end.
    assert_eq!(bounds(&spans), vec![(0, 7, 5), (0, 7, 5), (0, 7, 5)]);
}

#[test]
fn a_lyric_row_whose_target_is_absent_is_not_absorbed_by_the_row_before_it() {
    let spans = spans_of(
        vec![note_row("C", 0), lyric_row("V1", 1)],
        &[link("V1", "M")],
    );

    assert_eq!(bounds(&spans), vec![(0, 5, 5), (6, 6, 6)]);
}

#[test]
fn only_the_lyric_rows_linked_to_a_target_are_absorbed_into_it() {
    let spans = spans_of(
        vec![note_row("M", 0), lyric_row("V1", 1), lyric_row("W1", 2)],
        &[link("V1", "M"), link("W1", "B")],
    );

    assert_eq!(bounds(&spans), vec![(0, 6, 5), (0, 6, 5), (7, 7, 7)]);
}
