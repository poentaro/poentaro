use super::*;

pub(super) fn note_span_from_wit(span: NoteSpan) -> crate::types::NoteSpanOut {
    crate::types::NoteSpanOut {
        source_part_index: span.source_part_index as usize,
        part_abbreviation: span.part_abbreviation,
        note_id: span.note_id as usize,
        measure_index: span.measure_index as usize,
        start: span.start.map(|v| v as usize),
        end: span.end.map(|v| v as usize),
        timing: crate::types::MeasureTimingOut {
            offset: span.timing.offset,
            duration: span.timing.duration,
        },
    }
}

pub(super) fn note_cell_in_from_wit(cell: NoteCellIn) -> crate::note_selection_types::NoteCellIn {
    crate::note_selection_types::NoteCellIn {
        source_part_index: cell.source_part_index as usize,
        note_id: cell.note_id as usize,
    }
}

pub(super) fn group_note_selection_response_to_wit(
    response: crate::note_selection_types::GroupNoteSelectionResponse,
) -> GroupNoteSelectionResponse {
    match response {
        crate::note_selection_types::GroupNoteSelectionResponse::Ok { runs } => {
            GroupNoteSelectionResponse::Ok(GroupNoteSelectionSuccess {
                runs: runs
                    .into_iter()
                    .map(|run| NoteSelectionRun {
                        source_part_index: run.source_part_index as u32,
                        part_abbreviation: run.part_abbreviation,
                        measure_index: run.measure_index as u32,
                        start_byte: run.start_byte as u32,
                        end_byte: run.end_byte as u32,
                    })
                    .collect(),
            })
        }
        crate::note_selection_types::GroupNoteSelectionResponse::Err => {
            GroupNoteSelectionResponse::Err
        }
    }
}

pub(super) fn lyric_span_from_wit(span: LyricSpan) -> crate::types::LyricSpanOut {
    crate::types::LyricSpanOut {
        source_part_index: span.source_part_index as usize,
        part_abbreviation: span.part_abbreviation,
        note_id: span.note_id as usize,
        measure_index: span.measure_index as usize,
        start: span.start as usize,
        end: span.end as usize,
        timing: crate::types::MeasureTimingOut {
            offset: span.timing.offset,
            duration: span.timing.duration,
        },
    }
}

pub(super) fn lyric_cell_in_from_wit(
    cell: LyricCellIn,
) -> crate::lyric_selection_types::LyricCellIn {
    crate::lyric_selection_types::LyricCellIn {
        source_part_index: cell.source_part_index as usize,
        note_id: cell.note_id as usize,
    }
}

pub(super) fn group_lyric_selection_response_to_wit(
    response: crate::lyric_selection_types::GroupLyricSelectionResponse,
) -> GroupLyricSelectionResponse {
    match response {
        crate::lyric_selection_types::GroupLyricSelectionResponse::Ok { runs } => {
            GroupLyricSelectionResponse::Ok(GroupLyricSelectionSuccess {
                runs: runs
                    .into_iter()
                    .map(|run| LyricSelectionRun {
                        source_part_index: run.source_part_index as u32,
                        part_abbreviation: run.part_abbreviation,
                        measure_index: run.measure_index as u32,
                        start_byte: run.start_byte as u32,
                        end_byte: run.end_byte as u32,
                    })
                    .collect(),
            })
        }
        crate::lyric_selection_types::GroupLyricSelectionResponse::Err => {
            GroupLyricSelectionResponse::Err
        }
    }
}

pub(super) fn measure_span_to_wit(span: crate::types::MeasureSpanOut) -> MeasureSpan {
    MeasureSpan {
        start: span.start as u32,
        end: span.end as u32,
        view_zone_start: span.view_zone_start as u32,
        section_label: span.section_label,
        start_line: span.start_line as u32,
        end_line: span.end_line as u32,
    }
}

pub(super) fn section_range_to_wit(range: crate::types::SectionRangeOut) -> SectionRange {
    SectionRange {
        first_line: range.first_line as u32,
        last_line: range.last_line as u32,
        labels: range.labels,
    }
}

pub(super) fn sequence_entry_to_wit(entry: crate::types::SequenceEntryOut) -> SequenceEntry {
    SequenceEntry {
        label: entry.label,
        start_measure_index: entry.start_measure_index as u32,
        end_measure_index: entry.end_measure_index as u32,
    }
}

pub(super) fn list_measure_spans_response_to_wit(
    response: crate::types::ListMeasureSpansResponse,
) -> ListMeasureSpansResponse {
    match response {
        crate::types::ListMeasureSpansResponse::Ok {
            spans,
            section_ranges,
            sequence_entries,
        } => ListMeasureSpansResponse::Ok(ListMeasureSpansSuccess {
            spans: spans.into_iter().map(measure_span_to_wit).collect(),
            section_ranges: section_ranges
                .into_iter()
                .map(section_range_to_wit)
                .collect(),
            sequence_entries: sequence_entries
                .into_iter()
                .map(sequence_entry_to_wit)
                .collect(),
        }),
        crate::types::ListMeasureSpansResponse::Err => ListMeasureSpansResponse::Err,
    }
}

pub(super) fn note_span_to_wit(span: &crate::types::NoteSpanOut) -> NoteSpan {
    NoteSpan {
        source_part_index: span.source_part_index as u32,
        part_abbreviation: span.part_abbreviation.clone(),
        note_id: span.note_id as u32,
        measure_index: span.measure_index as u32,
        start: span.start.map(|v| v as u32),
        end: span.end.map(|v| v as u32),
        timing: MeasureTiming {
            offset: span.timing.offset,
            duration: span.timing.duration,
        },
    }
}

pub(super) fn list_note_spans_response_to_wit(
    response: crate::types::ListNoteSpansResponse,
) -> ListNoteSpansResponse {
    match response {
        crate::types::ListNoteSpansResponse::Ok { spans } => {
            ListNoteSpansResponse::Ok(ListNoteSpansSuccess {
                spans: spans.iter().map(note_span_to_wit).collect(),
            })
        }
        crate::types::ListNoteSpansResponse::Err => ListNoteSpansResponse::Err,
    }
}

pub(super) fn lyric_span_to_wit(span: &crate::types::LyricSpanOut) -> LyricSpan {
    LyricSpan {
        source_part_index: span.source_part_index as u32,
        part_abbreviation: span.part_abbreviation.clone(),
        note_id: span.note_id as u32,
        measure_index: span.measure_index as u32,
        start: span.start as u32,
        end: span.end as u32,
        timing: MeasureTiming {
            offset: span.timing.offset,
            duration: span.timing.duration,
        },
    }
}

pub(super) fn list_lyric_spans_response_to_wit(
    response: crate::types::ListLyricSpansResponse,
) -> ListLyricSpansResponse {
    match response {
        crate::types::ListLyricSpansResponse::Ok { spans } => {
            ListLyricSpansResponse::Ok(ListLyricSpansSuccess {
                spans: spans.iter().map(lyric_span_to_wit).collect(),
            })
        }
        crate::types::ListLyricSpansResponse::Err => ListLyricSpansResponse::Err,
    }
}

pub(super) fn note_source_span_from_wit(span: NoteSpan) -> poentaro::NoteSourceSpan {
    poentaro::NoteSourceSpan {
        source_part_index: span.source_part_index as usize,
        part_abbreviation: span.part_abbreviation,
        note_id: span.note_id as usize,
        measure_index: span.measure_index as usize,
        start: span.start.map(|v| v as usize),
        end: span.end.map(|v| v as usize),
        timing: poentaro::note_spans::MeasureTiming {
            offset: span.timing.offset,
            duration: span.timing.duration,
        },
    }
}

pub(super) fn lyric_source_span_from_wit(span: LyricSpan) -> poentaro::LyricSourceSpan {
    poentaro::LyricSourceSpan {
        source_part_index: span.source_part_index as usize,
        part_abbreviation: span.part_abbreviation,
        note_id: span.note_id as usize,
        measure_index: span.measure_index as usize,
        start: span.start as usize,
        end: span.end as usize,
        timing: poentaro::note_spans::MeasureTiming {
            offset: span.timing.offset,
            duration: span.timing.duration,
        },
    }
}

pub(super) fn byte_ranges_from_wit(
    ranges: Vec<ByteRange>,
) -> Vec<poentaro::source_edit::ByteRange> {
    ranges
        .into_iter()
        .map(|range| poentaro::source_edit::ByteRange {
            start_byte: range.start_byte,
            end_byte: range.end_byte,
        })
        .collect()
}

pub(super) fn note_selection_to_wit(selection: poentaro::NoteSelection) -> NoteSelection {
    NoteSelection {
        cells: selection
            .cells
            .into_iter()
            .map(|cell| NoteCellOut {
                source_part_index: cell.source_part_index as u32,
                note_id: cell.note_id as u32,
            })
            .collect(),
        runs: selection
            .runs
            .into_iter()
            .map(|run| NoteSelectionRun {
                source_part_index: run.source_part_index as u32,
                part_abbreviation: run.part_abbreviation,
                measure_index: run.measure_index as u32,
                start_byte: run.start_byte as u32,
                end_byte: run.end_byte as u32,
            })
            .collect(),
    }
}

pub(super) fn lyric_selection_to_wit(selection: poentaro::LyricSelection) -> LyricSelection {
    LyricSelection {
        cells: selection
            .cells
            .into_iter()
            .map(|cell| LyricCellOut {
                source_part_index: cell.source_part_index as u32,
                note_id: cell.note_id as u32,
            })
            .collect(),
        runs: selection
            .runs
            .into_iter()
            .map(|run| LyricSelectionRun {
                source_part_index: run.source_part_index as u32,
                part_abbreviation: run.part_abbreviation,
                measure_index: run.measure_index as u32,
                start_byte: run.start_byte as u32,
                end_byte: run.end_byte as u32,
            })
            .collect(),
    }
}
