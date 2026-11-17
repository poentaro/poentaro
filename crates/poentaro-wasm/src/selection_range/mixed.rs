use crate::types::{LyricSpanOut, NoteSpanOut};

use super::helpers::{lyric_measure_index, note_measure_index, MeasureSpan};
use super::types::{ClickableElementId, LyricCellOut, NoteCellOut, ResolveSelectionRangeResponse};

/// The three "mixed" label-mixed pairs — `Note ↔ LyricLabel`, `Lyric ↔
/// PartLabel`, and `PartLabel ↔ LyricLabel` — each of which has one
/// note-contributing side (a `Note`, treated as its own
/// `[measure_index, measure_index]` span, or a `PartLabel`, already a
/// `[start, end]` span) and one lyric-contributing side (a `Lyric`,
/// similarly treated as its own single-measure span, or a `LyricLabel`,
/// already a span) that always carries its own `verse`. See
/// [`note_like_lyric_like_range`] for the shared rule all three reuse. Each
/// pair gets its own small matcher below — [`resolve_note_lyric_label`],
/// [`resolve_lyric_part_label`], [`resolve_part_label_lyric_label`] — tried
/// in turn, since a single `match` covering all three (with each arm's
/// full field-destructuring pattern inlined) runs well past clippy's
/// per-function line limit.
pub(crate) fn resolve(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> Option<ResolveSelectionRangeResponse> {
    resolve_note_lyric_label(note_spans, lyric_spans, anchor, current)
        .or_else(|| resolve_lyric_part_label(note_spans, lyric_spans, anchor, current))
        .or_else(|| resolve_part_label_lyric_label(note_spans, lyric_spans, anchor, current))
}

/// `Note ↔ LyricLabel` — the `Note` side always contributes to
/// `note_cells` (treated as its own single-measure span, looked up from
/// `note_spans`), the `LyricLabel` side always contributes to
/// `lyric_cells` restricted to its own `verse`. `None` if the `Note`
/// endpoint's own span can't be found — shouldn't happen for a valid
/// click-derived ID, but every other topic module also declines this
/// variant pair, so `resolve_selection_range_response`'s fallback chain
/// still lands on `Err` overall, exactly like this crate's other guarded
/// lookups.
fn resolve_note_lyric_label(
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
        ClickableElementId::LyricLabel {
            source_part_index: label_part,
            measure_index_start: label_start,
            measure_index_end: label_end,
        },
    )
    | (
        ClickableElementId::LyricLabel {
            source_part_index: label_part,
            measure_index_start: label_start,
            measure_index_end: label_end,
        },
        ClickableElementId::Note {
            source_part_index: note_part,
            note_id,
        },
    )) = (anchor, current)
    else {
        return None;
    };

    let note_measure = note_measure_index(note_spans, *note_part, *note_id)?;
    Some(note_like_lyric_like_range(
        note_spans,
        lyric_spans,
        MeasureSpan {
            part: *note_part,
            start: note_measure,
            end: note_measure,
        },
        MeasureSpan {
            part: *label_part,
            start: *label_start,
            end: *label_end,
        },
    ))
}

/// `Lyric ↔ PartLabel` — the mirror of `resolve_note_lyric_label`: the
/// `PartLabel` side always contributes to `note_cells` (it already spans a
/// measure range directly, no lookup needed), the `Lyric` side always
/// contributes to `lyric_cells` restricted to its own `verse` (treated as
/// its own single-measure span, looked up from `lyric_spans`). Unlike
/// `Note ↔ PartLabel`, this is never gated by part-match: the
/// lyric-contributing side here is a real syllable, never a duplicate of
/// the note-contributing side the way two `PartLabel`s can be, so there's
/// no degenerate-click case to guard against. `None` for the same reason
/// [`resolve_note_lyric_label`] returns `None` rather than `Some(Err)`.
fn resolve_lyric_part_label(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> Option<ResolveSelectionRangeResponse> {
    let ((
        ClickableElementId::Lyric {
            source_part_index: lyric_part,
            note_id: lyric_note_id,
        },
        ClickableElementId::PartLabel {
            source_part_index: label_part,
            measure_index_start: label_start,
            measure_index_end: label_end,
        },
    )
    | (
        ClickableElementId::PartLabel {
            source_part_index: label_part,
            measure_index_start: label_start,
            measure_index_end: label_end,
        },
        ClickableElementId::Lyric {
            source_part_index: lyric_part,
            note_id: lyric_note_id,
        },
    )) = (anchor, current)
    else {
        return None;
    };

    let lyric_measure = lyric_measure_index(lyric_spans, *lyric_part, *lyric_note_id)?;
    Some(note_like_lyric_like_range(
        note_spans,
        lyric_spans,
        MeasureSpan {
            part: *label_part,
            start: *label_start,
            end: *label_end,
        },
        MeasureSpan {
            part: *lyric_part,
            start: lyric_measure,
            end: lyric_measure,
        },
    ))
}

/// `PartLabel ↔ LyricLabel` — the third "mixed" pair, and not a sixth
/// special case: it's the exact same shape as `Note ↔ LyricLabel`/
/// `Lyric ↔ PartLabel` with both sides already being labels, so neither
/// side needs a `note_spans`/`lyric_spans` lookup at all.
fn resolve_part_label_lyric_label(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> Option<ResolveSelectionRangeResponse> {
    let ((
        ClickableElementId::PartLabel {
            source_part_index: part_label_part,
            measure_index_start: part_label_start,
            measure_index_end: part_label_end,
        },
        ClickableElementId::LyricLabel {
            source_part_index: lyric_label_part,
            measure_index_start: lyric_label_start,
            measure_index_end: lyric_label_end,
        },
    )
    | (
        ClickableElementId::LyricLabel {
            source_part_index: lyric_label_part,
            measure_index_start: lyric_label_start,
            measure_index_end: lyric_label_end,
        },
        ClickableElementId::PartLabel {
            source_part_index: part_label_part,
            measure_index_start: part_label_start,
            measure_index_end: part_label_end,
        },
    )) = (anchor, current)
    else {
        return None;
    };

    Some(note_like_lyric_like_range(
        note_spans,
        lyric_spans,
        MeasureSpan {
            part: *part_label_part,
            start: *part_label_start,
            end: *part_label_end,
        },
        MeasureSpan {
            part: *lyric_label_part,
            start: *lyric_label_start,
            end: *lyric_label_end,
        },
    ))
}

/// Backs all three "mixed" label-mixed pairs. `note_cells` is always
/// populated from the combined `part_range`/`measure_range`, with no
/// part-match gate (unlike `Note ↔ PartLabel`'s `note_part_label_range`):
/// the lyric-contributing side here is a real syllable or verse label,
/// never a duplicate of the note-contributing side, so there's no
/// degenerate-click case to guard against. `lyric_cells` is always
/// populated too, restricted to the lyric-contributing side's own `verse`.
/// Callers resolve either side's own single-measure lookup (for a
/// `Note`/`Lyric` endpoint) before calling this — a `PartLabel`/
/// `LyricLabel` endpoint's span is already known, no lookup needed.
fn note_like_lyric_like_range(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    note_like: MeasureSpan,
    lyric_like: MeasureSpan,
) -> ResolveSelectionRangeResponse {
    let part_start = note_like.part.min(lyric_like.part);
    let part_end = note_like.part.max(lyric_like.part);
    let measure_start = note_like.start.min(lyric_like.start);
    let measure_end = note_like.end.max(lyric_like.end);

    let note_cells = note_spans
        .iter()
        .filter(|span| {
            span.source_part_index >= part_start
                && span.source_part_index <= part_end
                && span.measure_index >= measure_start
                && span.measure_index <= measure_end
        })
        .map(|span| NoteCellOut {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        })
        .collect();
    let lyric_cells = lyric_spans
        .iter()
        .filter(|span| {
            span.source_part_index >= part_start
                && span.source_part_index <= part_end
                && span.measure_index >= measure_start
                && span.measure_index <= measure_end
        })
        .map(|span| LyricCellOut {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        })
        .collect();

    ResolveSelectionRangeResponse::Ok {
        note_cells,
        lyric_cells,
    }
}
