use crate::types::{LyricSpanOut, NoteSpanOut};

use super::helpers::{note_measure_index, MeasureSpan, NoteEndpoint};
use super::types::{ClickableElementId, LyricCellOut, NoteCellOut, ResolveSelectionRangeResponse};

/// `PartLabel ↔ PartLabel` and `Note ↔ PartLabel`. See [`part_label_range`]
/// and [`note_part_label_range`] for each rule's own doc comment.
pub(crate) fn resolve(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> Option<ResolveSelectionRangeResponse> {
    match (anchor, current) {
        // `PartLabel ↔ PartLabel` — system-agnostic, mirroring the
        // cross-part `Note ↔ Note` arm and the `LyricLabel ↔ LyricLabel`
        // arm: range resolution has no notion of "system" at all, so a
        // pair whose labels sit in different systems (different
        // `(measure_index_start, measure_index_end)`) resolves exactly
        // like a same-system pair, just with the measure range spanning
        // `min(anchor_start, current_start)` .. `max(anchor_end,
        // current_end)` instead of collapsing to one shared span. This *is*
        // the same-system case's rule too — when both labels share one
        // system, `anchor_start == current_start` and `anchor_end ==
        // current_end`, so the min/max are that shared span — so there's
        // no separate same-system arm to keep in sync. No Cmd/Ctrl
        // modifier required; see
        // `part-label-range-select-crosses-system.feature`.
        (
            ClickableElementId::PartLabel {
                source_part_index: anchor_part,
                measure_index_start: anchor_start,
                measure_index_end: anchor_end,
            },
            ClickableElementId::PartLabel {
                source_part_index: current_part,
                measure_index_start: current_start,
                measure_index_end: current_end,
            },
        ) => Some(part_label_range(
            note_spans,
            lyric_spans,
            MeasureSpan {
                part: *anchor_part,
                start: *anchor_start,
                end: *anchor_end,
            },
            MeasureSpan {
                part: *current_part,
                start: *current_start,
                end: *current_end,
            },
        )),
        // `Note ↔ PartLabel` — the first of the label-mixed rows (see
        // `PLAN-clickable-element-id-selection.md`'s Status section for the
        // full per-pair writeup). See `note_part_label_range`'s own doc
        // comment.
        (
            ClickableElementId::Note {
                source_part_index: note_part,
                note_id,
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
            ClickableElementId::Note {
                source_part_index: note_part,
                note_id,
            },
        ) => Some(note_part_label_range(
            note_spans,
            lyric_spans,
            NoteEndpoint {
                part: *note_part,
                note_id: *note_id,
            },
            MeasureSpan {
                part: *label_part,
                start: *label_start,
                end: *label_end,
            },
        )),
        _ => None,
    }
}

/// `PartLabel ↔ PartLabel`'s rule — a pure rectangle over rendered rows:
/// every note and lyric cell whose part lies between the two labels' parts
/// and whose measure lies in the merged measure range. Lyric parts are
/// ordinary parts, so a notes part's lyric rows are included only when they
/// fall inside the swept part range.
fn part_label_range(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: MeasureSpan,
    current: MeasureSpan,
) -> ResolveSelectionRangeResponse {
    let part_start = anchor.part.min(current.part);
    let part_end = anchor.part.max(current.part);
    let measure_start = anchor.start.min(current.start);
    let measure_end = anchor.end.max(current.end);

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

/// Backs the `Note ↔ PartLabel` arm. Treats the `Note` endpoint as a
/// degenerate single-measure "label" for its own part
/// (`measure_start == measure_end == its own measure_index`, looked up from
/// `note_spans` the same way the cross-part `Note ↔ Note` arm does) and
/// reuses `part_label_range`. `Err` if the `Note` endpoint's own span can't
/// be found (shouldn't happen for a valid click-derived ID; guarded rather
/// than panicking, mirroring this crate's other cross-scope arms).
fn note_part_label_range(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    note: NoteEndpoint,
    label: MeasureSpan,
) -> ResolveSelectionRangeResponse {
    let Some(note_measure) = note_measure_index(note_spans, note.part, note.note_id) else {
        return ResolveSelectionRangeResponse::Err;
    };

    part_label_range(
        note_spans,
        lyric_spans,
        MeasureSpan {
            part: note.part,
            start: note_measure,
            end: note_measure,
        },
        label,
    )
}
