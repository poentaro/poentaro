use crate::types::{LyricSpanOut, NoteSpanOut};

use super::types::{ClickableElementId, LyricCellOut, NoteCellOut, ResolveSelectionRangeResponse};

/// `Measure ↔ Measure` — Phase 1's simplest, lowest-risk row, chosen to
/// validate the wasm plumbing end-to-end before 'note'/'lyric'/label modes
/// followed (see `PLAN-clickable-element-id-selection.md`).
pub(crate) fn resolve(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> Option<ResolveSelectionRangeResponse> {
    match (anchor, current) {
        (
            ClickableElementId::Measure {
                measure_index_start: anchor_start,
                measure_index_end: anchor_end,
            },
            ClickableElementId::Measure {
                measure_index_start: current_start,
                measure_index_end: current_end,
            },
        ) => {
            let range_start = (*anchor_start).min(*current_start);
            let range_end = (*anchor_end).max(*current_end);

            let note_cells = note_spans
                .iter()
                .filter(|span| span.measure_index >= range_start && span.measure_index <= range_end)
                .map(|span| NoteCellOut {
                    source_part_index: span.source_part_index,
                    note_id: span.note_id,
                })
                .collect();
            let lyric_cells = lyric_spans
                .iter()
                .filter(|span| span.measure_index >= range_start && span.measure_index <= range_end)
                .map(|span| LyricCellOut {
                    source_part_index: span.source_part_index,
                    note_id: span.note_id,
                })
                .collect();

            Some(ResolveSelectionRangeResponse::Ok {
                note_cells,
                lyric_cells,
            })
        }
        _ => None,
    }
}
