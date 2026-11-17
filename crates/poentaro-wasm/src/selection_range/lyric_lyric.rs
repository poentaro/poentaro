use crate::types::{LyricSpanOut, NoteSpanOut};

use super::helpers::{cross_row_range, LyricEndpoint, RowEndpoint};
use super::types::{ClickableElementId, LyricCellOut, ResolveSelectionRangeResponse};

/// `Lyric ↔ Lyric`, every scope: same lyric part ([`same_part`]) and
/// different lyric parts ([`cross_part`]). Each lyric part is its own part,
/// so a pair across verses of one target is just a cross-part pair.
pub(crate) fn resolve(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> Option<ResolveSelectionRangeResponse> {
    match (anchor, current) {
        (
            ClickableElementId::Lyric {
                source_part_index: anchor_part,
                note_id: anchor_id,
            },
            ClickableElementId::Lyric {
                source_part_index: current_part,
                note_id: current_id,
            },
        ) if anchor_part == current_part => Some(same_part(
            lyric_spans,
            *anchor_part,
            *anchor_id,
            *current_id,
        )),
        (
            ClickableElementId::Lyric {
                source_part_index: anchor_part,
                note_id: anchor_id,
            },
            ClickableElementId::Lyric {
                source_part_index: current_part,
                note_id: current_id,
            },
        ) => Some(cross_part(
            note_spans,
            lyric_spans,
            LyricEndpoint {
                part: *anchor_part,
                note_id: *anchor_id,
            },
            LyricEndpoint {
                part: *current_part,
                note_id: *current_id,
            },
        )),
        _ => None,
    }
}

/// Same lyric part — ranges by `note_id` alone. No note cells.
fn same_part(
    lyric_spans: &[LyricSpanOut],
    part: usize,
    anchor_id: usize,
    current_id: usize,
) -> ResolveSelectionRangeResponse {
    let range_start = anchor_id.min(current_id);
    let range_end = anchor_id.max(current_id);

    let lyric_cells = lyric_spans
        .iter()
        .filter(|span| {
            span.source_part_index == part
                && span.note_id >= range_start
                && span.note_id <= range_end
        })
        .map(|span| LyricCellOut {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        })
        .collect();

    ResolveSelectionRangeResponse::Ok {
        note_cells: Vec::new(),
        lyric_cells,
    }
}

/// Different lyric parts — the lyric cells of [`cross_row_range`]. A range
/// between two syllables selects only syllables, so the notes rows it passes
/// over stay unselected.
fn cross_part(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: LyricEndpoint,
    current: LyricEndpoint,
) -> ResolveSelectionRangeResponse {
    match cross_row_range(
        note_spans,
        lyric_spans,
        RowEndpoint::Lyric(anchor),
        RowEndpoint::Lyric(current),
    ) {
        ResolveSelectionRangeResponse::Ok { lyric_cells, .. } => {
            ResolveSelectionRangeResponse::Ok {
                note_cells: Vec::new(),
                lyric_cells,
            }
        }
        ResolveSelectionRangeResponse::Err => ResolveSelectionRangeResponse::Err,
    }
}
