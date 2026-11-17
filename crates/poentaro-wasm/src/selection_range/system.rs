use poentaro::grid_layout::MeasureRange;

use crate::types::{LyricSpanOut, NoteSpanOut};

use super::helpers::{lyric_measure_index, note_measure_index};
use super::types::{ClickableElementId, ResolveSelectionRangeResponse};

/// The measures an endpoint covers: a note's or syllable's own measure, or
/// a measure's, part label's or lyric label's own range.
fn endpoint_measures(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    endpoint: &ClickableElementId,
) -> Option<MeasureRange> {
    let single = |measure_index: usize| MeasureRange {
        start: measure_index,
        end: measure_index,
    };
    match endpoint {
        ClickableElementId::Note {
            source_part_index,
            note_id,
        } => note_measure_index(note_spans, *source_part_index, *note_id).map(single),
        ClickableElementId::Lyric {
            source_part_index,
            note_id,
        } => lyric_measure_index(lyric_spans, *source_part_index, *note_id).map(single),
        ClickableElementId::Measure {
            measure_index_start,
            measure_index_end,
        }
        | ClickableElementId::PartLabel {
            measure_index_start,
            measure_index_end,
            ..
        }
        | ClickableElementId::LyricLabel {
            measure_index_start,
            measure_index_end,
            ..
        } => Some(MeasureRange {
            start: *measure_index_start,
            end: *measure_index_end,
        }),
    }
}

/// The system holding `measure_index`, or just that measure when no system
/// does.
fn system_containing(systems: &[MeasureRange], measure_index: usize) -> MeasureRange {
    systems
        .iter()
        .find(|system| (system.start..=system.end).contains(&measure_index))
        .copied()
        .unwrap_or(MeasureRange {
            start: measure_index,
            end: measure_index,
        })
}

/// Every note and syllable, in every part, of every system from the
/// anchor's through the current endpoint's. Backs the whole-system gestures
/// (a click on a bar number, a Cmd/Ctrl-click on a part label). `systems`
/// are the rendered systems' inclusive measure ranges; an endpoint whose
/// note or syllable isn't in the spans makes the response `Err`.
pub(crate) fn resolve_system_selection_range_response(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    systems: &[MeasureRange],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> ResolveSelectionRangeResponse {
    let range = [anchor, current]
        .iter()
        .map(|endpoint| endpoint_measures(note_spans, lyric_spans, endpoint))
        .collect::<Option<Vec<_>>>()
        .and_then(|ranges| {
            Some(MeasureRange {
                start: system_containing(systems, ranges.iter().map(|range| range.start).min()?)
                    .start,
                end: system_containing(systems, ranges.iter().map(|range| range.end).max()?).end,
            })
        });
    let Some(range) = range else {
        return ResolveSelectionRangeResponse::Err;
    };
    let measures = ClickableElementId::Measure {
        measure_index_start: range.start,
        measure_index_end: range.end,
    };
    super::measure::resolve(note_spans, lyric_spans, &measures, &measures)
        .unwrap_or(ResolveSelectionRangeResponse::Err)
}
