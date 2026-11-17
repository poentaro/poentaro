use crate::selection_range::resolve_selection_range_response;
use crate::selection_range::types::{
    ClickableElementId, LyricCellOut, NoteCellOut, ResolveSelectionRangeResponse,
};
use crate::types::{LyricSpanOut, MeasureTimingOut, NoteSpanOut};

/// A 4/4 measure's length in quarter-beats.
const MEASURE_LENGTH: u32 = 16;

/// A note filling its whole measure.
pub(super) fn note_span(
    source_part_index: usize,
    note_id: usize,
    measure_index: usize,
) -> NoteSpanOut {
    timed_note_span(source_part_index, note_id, measure_index, 0, MEASURE_LENGTH)
}

/// A note sounding `duration` quarter-beats from `offset` into its measure.
pub(super) fn timed_note_span(
    source_part_index: usize,
    note_id: usize,
    measure_index: usize,
    offset: u32,
    duration: u32,
) -> NoteSpanOut {
    NoteSpanOut {
        source_part_index,
        part_abbreviation: None,
        note_id,
        measure_index,
        start: Some(note_id * 10),
        end: Some(note_id * 10 + 1),
        timing: MeasureTimingOut { offset, duration },
    }
}

/// A syllable sung for its whole measure.
pub(super) fn lyric_span(
    source_part_index: usize,
    note_id: usize,
    measure_index: usize,
) -> LyricSpanOut {
    LyricSpanOut {
        source_part_index,
        part_abbreviation: None,
        note_id,
        measure_index,
        start: note_id * 10,
        end: note_id * 10 + 1,
        timing: MeasureTimingOut {
            offset: 0,
            duration: MEASURE_LENGTH,
        },
    }
}

pub(super) fn measure(start: usize, end: usize) -> ClickableElementId {
    ClickableElementId::Measure {
        measure_index_start: start,
        measure_index_end: end,
    }
}

pub(super) fn note(source_part_index: usize, note_id: usize) -> ClickableElementId {
    ClickableElementId::Note {
        source_part_index,
        note_id,
    }
}

pub(super) fn lyric(source_part_index: usize, note_id: usize) -> ClickableElementId {
    ClickableElementId::Lyric {
        source_part_index,
        note_id,
    }
}

pub(super) fn part_label(
    source_part_index: usize,
    measure_index_start: usize,
    measure_index_end: usize,
) -> ClickableElementId {
    ClickableElementId::PartLabel {
        source_part_index,
        measure_index_start,
        measure_index_end,
    }
}

pub(super) fn lyric_label(
    source_part_index: usize,
    measure_index_start: usize,
    measure_index_end: usize,
) -> ClickableElementId {
    ClickableElementId::LyricLabel {
        source_part_index,
        measure_index_start,
        measure_index_end,
    }
}

pub(super) fn note_cell(source_part_index: usize, note_id: usize) -> NoteCellOut {
    NoteCellOut {
        source_part_index,
        note_id,
    }
}

pub(super) fn lyric_cell(source_part_index: usize, note_id: usize) -> LyricCellOut {
    LyricCellOut {
        source_part_index,
        note_id,
    }
}

/// Fixture shared by every case: parts in declaration order are
/// 0 = notes A (note_ids 0-2 in measures 0-2), 1 = lyric part of A (a
/// syllable per note), 2 = a second lyric part of A (note_ids 0-1),
/// 3 = notes B (note_id 3, measure 1 only), 4 = lyric part of B.
pub(super) fn fixture() -> (Vec<NoteSpanOut>, Vec<LyricSpanOut>) {
    let note_spans = vec![
        note_span(0, 0, 0),
        note_span(0, 1, 1),
        note_span(0, 2, 2),
        note_span(3, 3, 1),
    ];
    let lyric_spans = vec![
        lyric_span(1, 0, 0),
        lyric_span(1, 1, 1),
        lyric_span(1, 2, 2),
        lyric_span(2, 0, 0),
        lyric_span(2, 1, 1),
        lyric_span(4, 3, 1),
    ];
    (note_spans, lyric_spans)
}

/// Asserts that resolving `(anchor, current)` — in both orders — over
/// [`fixture`] yields exactly the expected cells.
pub(super) fn assert_range(
    anchor: &ClickableElementId,
    current: &ClickableElementId,
    expected_note_cells: &[NoteCellOut],
    expected_lyric_cells: &[LyricCellOut],
) {
    let (note_spans, lyric_spans) = fixture();
    for (first, second) in [(anchor, current), (current, anchor)] {
        match resolve_selection_range_response(&note_spans, &lyric_spans, first, second) {
            ResolveSelectionRangeResponse::Ok {
                note_cells,
                lyric_cells,
            } => {
                assert_eq!(note_cells, expected_note_cells);
                assert_eq!(lyric_cells, expected_lyric_cells);
            }
            ResolveSelectionRangeResponse::Err => panic!("expected Ok, got Err"),
        }
    }
}

/// Asserts that resolving `(anchor, current)` over [`fixture`] is `Err`.
pub(super) fn assert_unresolved(anchor: &ClickableElementId, current: &ClickableElementId) {
    let (note_spans, lyric_spans) = fixture();
    assert!(matches!(
        resolve_selection_range_response(&note_spans, &lyric_spans, anchor, current),
        ResolveSelectionRangeResponse::Err
    ));
}
