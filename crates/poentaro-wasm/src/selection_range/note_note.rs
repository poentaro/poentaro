use crate::types::{LyricSpanOut, NoteSpanOut};

use super::helpers::{cross_row_range, NoteEndpoint, RowEndpoint};
use super::types::{ClickableElementId, NoteCellOut, ResolveSelectionRangeResponse};

/// `Note ↔ Note`, both scopes — same-part (ranged by `note_id`) and
/// cross-part (ranged by time). See
/// [`same_part`] and [`cross_part`] for each rule's own doc comment.
pub(crate) fn resolve(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> Option<ResolveSelectionRangeResponse> {
    match (anchor, current) {
        (
            ClickableElementId::Note {
                source_part_index: anchor_part,
                note_id: anchor_id,
            },
            ClickableElementId::Note {
                source_part_index: current_part,
                note_id: current_id,
            },
        ) if anchor_part == current_part => {
            Some(same_part(note_spans, *anchor_part, *anchor_id, *current_id))
        }
        // Cross-part `Note ↔ Note` — Phase 2's first row (see
        // `PLAN-clickable-element-id-selection.md`). The guard on the arm
        // above claims same-part pairs; a different-part pair falls through
        // to here, the same guard-then-fallthrough pattern `LyricLabel`'s
        // arm uses relative to `PartLabel`'s.
        (
            ClickableElementId::Note {
                source_part_index: anchor_part,
                note_id: anchor_id,
            },
            ClickableElementId::Note {
                source_part_index: current_part,
                note_id: current_id,
            },
        ) => Some(cross_part(
            note_spans,
            lyric_spans,
            NoteEndpoint {
                part: *anchor_part,
                note_id: *anchor_id,
            },
            NoteEndpoint {
                part: *current_part,
                note_id: *current_id,
            },
        )),
        _ => None,
    }
}

/// Same-part `Note ↔ Note` — ranges by `note_id` alone. No lyric cells —
/// an index range has no notion of a lyric row.
fn same_part(
    note_spans: &[NoteSpanOut],
    part: usize,
    anchor_id: usize,
    current_id: usize,
) -> ResolveSelectionRangeResponse {
    let range_start = anchor_id.min(current_id);
    let range_end = anchor_id.max(current_id);

    let note_cells = note_spans
        .iter()
        .filter(|span| {
            span.source_part_index == part
                && span.note_id >= range_start
                && span.note_id <= range_end
        })
        .map(|span| NoteCellOut {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        })
        .collect();

    ResolveSelectionRangeResponse::Ok {
        note_cells,
        lyric_cells: Vec::new(),
    }
}

/// Cross-part `Note ↔ Note` — see [`cross_row_range`].
fn cross_part(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: NoteEndpoint,
    current: NoteEndpoint,
) -> ResolveSelectionRangeResponse {
    cross_row_range(
        note_spans,
        lyric_spans,
        RowEndpoint::Note(anchor),
        RowEndpoint::Note(current),
    )
}
