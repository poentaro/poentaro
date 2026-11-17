use super::*;

pub(super) fn clickable_element_id_from_wit(
    id: ClickableElementId,
) -> crate::selection_range::ClickableElementId {
    match id {
        ClickableElementId::Note(fields) => crate::selection_range::ClickableElementId::Note {
            source_part_index: fields.source_part_index as usize,
            note_id: fields.note_id as usize,
        },
        ClickableElementId::Lyric(fields) => crate::selection_range::ClickableElementId::Lyric {
            source_part_index: fields.source_part_index as usize,
            note_id: fields.note_id as usize,
        },
        ClickableElementId::Measure(fields) => {
            crate::selection_range::ClickableElementId::Measure {
                measure_index_start: fields.measure_index_start as usize,
                measure_index_end: fields.measure_index_end as usize,
            }
        }
        ClickableElementId::PartLabel(fields) => {
            crate::selection_range::ClickableElementId::PartLabel {
                source_part_index: fields.source_part_index as usize,
                measure_index_start: fields.measure_index_start as usize,
                measure_index_end: fields.measure_index_end as usize,
            }
        }
        ClickableElementId::LyricLabel(fields) => {
            crate::selection_range::ClickableElementId::LyricLabel {
                source_part_index: fields.source_part_index as usize,
                measure_index_start: fields.measure_index_start as usize,
                measure_index_end: fields.measure_index_end as usize,
            }
        }
    }
}

pub(super) fn note_cell_out_to_wit(cell: &crate::selection_range::NoteCellOut) -> NoteCellOut {
    NoteCellOut {
        source_part_index: cell.source_part_index as u32,
        note_id: cell.note_id as u32,
    }
}

pub(super) fn lyric_cell_out_to_wit(cell: &crate::selection_range::LyricCellOut) -> LyricCellOut {
    LyricCellOut {
        source_part_index: cell.source_part_index as u32,
        note_id: cell.note_id as u32,
    }
}

pub(super) fn resolve_selection_range_response_to_wit(
    response: crate::selection_range::ResolveSelectionRangeResponse,
) -> ResolveSelectionRangeResponse {
    match response {
        crate::selection_range::ResolveSelectionRangeResponse::Ok {
            note_cells,
            lyric_cells,
        } => ResolveSelectionRangeResponse::Ok(ResolveSelectionRangeSuccess {
            note_cells: note_cells.iter().map(note_cell_out_to_wit).collect(),
            lyric_cells: lyric_cells.iter().map(lyric_cell_out_to_wit).collect(),
        }),
        crate::selection_range::ResolveSelectionRangeResponse::Err => {
            ResolveSelectionRangeResponse::Err
        }
    }
}
