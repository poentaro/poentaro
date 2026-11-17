use crate::types::LyricSpanOut;

use super::helpers::{lyric_measure_index, LyricEndpoint, MeasureSpan};
use super::types::{ClickableElementId, LyricCellOut, ResolveSelectionRangeResponse};

/// `LyricLabel ↔ LyricLabel` and `Lyric ↔ LyricLabel`. Range resolution has
/// no notion of "system", so labels in different systems resolve exactly
/// like same-system ones: the measure range spans `min(starts)..max(ends)`
/// and the part range spans the two lyric parts.
pub(crate) fn resolve(
    _note_spans: &[crate::types::NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> Option<ResolveSelectionRangeResponse> {
    match (anchor, current) {
        (
            ClickableElementId::LyricLabel {
                source_part_index: anchor_part,
                measure_index_start: anchor_start,
                measure_index_end: anchor_end,
            },
            ClickableElementId::LyricLabel {
                source_part_index: current_part,
                measure_index_start: current_start,
                measure_index_end: current_end,
            },
        ) => Some(lyric_label_range(
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
        (
            ClickableElementId::Lyric {
                source_part_index: lyric_part,
                note_id: lyric_note_id,
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
            ClickableElementId::Lyric {
                source_part_index: lyric_part,
                note_id: lyric_note_id,
            },
        ) => Some(lyric_lyric_label_range(
            lyric_spans,
            LyricEndpoint {
                part: *lyric_part,
                note_id: *lyric_note_id,
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

/// Every syllable of the lyric parts from `anchor.part` to `current.part`
/// within the union of the two measure spans.
fn lyric_label_range(
    lyric_spans: &[LyricSpanOut],
    anchor: MeasureSpan,
    current: MeasureSpan,
) -> ResolveSelectionRangeResponse {
    lyric_cells_in(
        lyric_spans,
        anchor.part.min(current.part),
        anchor.part.max(current.part),
        anchor.start.min(current.start),
        anchor.end.max(current.end),
    )
}

/// `Lyric ↔ LyricLabel`: the lyric endpoint collapses to its own measure,
/// then ranges like two labels. `Err` if the lyric's span can't be found.
fn lyric_lyric_label_range(
    lyric_spans: &[LyricSpanOut],
    lyric: LyricEndpoint,
    label: MeasureSpan,
) -> ResolveSelectionRangeResponse {
    let Some(lyric_measure) = lyric_measure_index(lyric_spans, lyric.part, lyric.note_id) else {
        return ResolveSelectionRangeResponse::Err;
    };
    lyric_cells_in(
        lyric_spans,
        lyric.part.min(label.part),
        lyric.part.max(label.part),
        lyric_measure.min(label.start),
        lyric_measure.max(label.end),
    )
}

fn lyric_cells_in(
    lyric_spans: &[LyricSpanOut],
    part_start: usize,
    part_end: usize,
    measure_start: usize,
    measure_end: usize,
) -> ResolveSelectionRangeResponse {
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
        note_cells: Vec::new(),
        lyric_cells,
    }
}
