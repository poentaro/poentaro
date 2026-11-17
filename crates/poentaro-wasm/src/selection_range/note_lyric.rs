use crate::types::{LyricSpanOut, NoteSpanOut};

use super::helpers::{cross_row_range, LyricEndpoint, NoteEndpoint, RowEndpoint};
use super::types::{ClickableElementId, ResolveSelectionRangeResponse};

/// `Note ↔ Lyric` cross-row — see [`cross_part`]. Lyric parts are ordinary
/// parts, so this is the same time range as cross-part `Note ↔ Note`, with
/// no special case for a lyric part and the part it sings along to.
pub(crate) fn resolve(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> Option<ResolveSelectionRangeResponse> {
    let ((
        ClickableElementId::Note {
            source_part_index: note_part,
            note_id,
        },
        ClickableElementId::Lyric {
            source_part_index: lyric_part,
            note_id: lyric_note_id,
        },
    )
    | (
        ClickableElementId::Lyric {
            source_part_index: lyric_part,
            note_id: lyric_note_id,
        },
        ClickableElementId::Note {
            source_part_index: note_part,
            note_id,
        },
    )) = (anchor, current)
    else {
        return None;
    };
    let lyric = LyricEndpoint {
        part: *lyric_part,
        note_id: *lyric_note_id,
    };
    Some(cross_part(
        note_spans,
        lyric_spans,
        NoteEndpoint {
            part: *note_part,
            note_id: *note_id,
        },
        lyric,
    ))
}

/// The `Note ↔ Lyric` cross-row rule — see [`cross_row_range`].
fn cross_part(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    note: NoteEndpoint,
    lyric: LyricEndpoint,
) -> ResolveSelectionRangeResponse {
    cross_row_range(
        note_spans,
        lyric_spans,
        RowEndpoint::Note(note),
        RowEndpoint::Lyric(lyric),
    )
}
