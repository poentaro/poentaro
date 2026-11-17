use itertools::Itertools;

use crate::types::{LyricSpanOut, MeasureTimingOut, NoteSpanOut};

use super::types::{LyricCellOut, NoteCellOut, ResolveSelectionRangeResponse};

/// A `Note` endpoint of a range-resolution pair, grouped into a struct
/// (rather than passed as loose arguments) per this repo's "never use
/// tuples in new data structures" rule — bundles the
/// `(source_part_index, note_id)` pair the cross-scope helper functions
/// below need together, and keeps each of those functions under
/// clippy's `too-many-arguments` limit.
#[derive(Clone, Copy)]
pub(crate) struct NoteEndpoint {
    pub part: usize,
    pub note_id: usize,
}

/// The `Lyric`-endpoint analog of [`NoteEndpoint`]; `part` is the lyric
/// part's own index.
pub(crate) type LyricEndpoint = NoteEndpoint;

/// A part-scoped measure span — the shared shape of a `PartLabel` or
/// `LyricLabel` endpoint, and of a `Note`/`Lyric` endpoint once collapsed to
/// its own degenerate single-measure span (see [`note_measure_index`]).
#[derive(Clone, Copy)]
pub(crate) struct MeasureSpan {
    pub part: usize,
    pub start: usize,
    pub end: usize,
}

/// Looks up a `Note` endpoint's own `measure_index` from `note_spans` by
/// `(source_part_index, note_id)` — shared by every arm that treats a
/// `Note` endpoint as a degenerate single-measure span for its own part
/// (the cross-part `Note ↔ Note` arm's own lookup, and every label-mixed
/// arm that reuses the same trick).
pub(crate) fn note_measure_index(
    note_spans: &[NoteSpanOut],
    part: usize,
    note_id: usize,
) -> Option<usize> {
    note_spans
        .iter()
        .find(|span| span.source_part_index == part && span.note_id == note_id)
        .map(|span| span.measure_index)
}

/// The `Lyric`-endpoint analog of `note_measure_index`.
pub(crate) fn lyric_measure_index(
    lyric_spans: &[LyricSpanOut],
    part: usize,
    note_id: usize,
) -> Option<usize> {
    lyric_spans
        .iter()
        .find(|span| span.source_part_index == part && span.note_id == note_id)
        .map(|span| span.measure_index)
}

/// A moment in the score. Offsets are in their own measure's quarter-beats,
/// which differ from measure to measure, so moments compare by measure first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ScoreTime {
    measure_index: usize,
    offset: u32,
}

/// The stretch of the score a cross-row range covers, end exclusive.
#[derive(Debug, Clone, Copy)]
struct TimeWindow {
    start: ScoreTime,
    end: ScoreTime,
}

impl TimeWindow {
    /// When a span sounds. A zero-length event (a grace note) still gets a
    /// sliver, so it can be selected.
    fn of_span(measure_index: usize, timing: MeasureTimingOut) -> Self {
        TimeWindow {
            start: ScoreTime {
                measure_index,
                offset: timing.offset,
            },
            end: ScoreTime {
                measure_index,
                offset: timing.offset + timing.duration.max(1),
            },
        }
    }

    fn union(self, other: Self) -> Self {
        TimeWindow {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }
}

/// A `Note` or `Lyric` endpoint of a cross-row range.
#[derive(Clone, Copy)]
pub(crate) enum RowEndpoint {
    Note(NoteEndpoint),
    Lyric(LyricEndpoint),
}

impl RowEndpoint {
    fn part(&self) -> usize {
        match self {
            RowEndpoint::Note(endpoint) | RowEndpoint::Lyric(endpoint) => endpoint.part,
        }
    }

    /// When the endpoint's cell sounds: every span of it, since a note tied
    /// across a bar line is one cell with a span in each measure.
    fn window(
        &self,
        note_spans: &[NoteSpanOut],
        lyric_spans: &[LyricSpanOut],
    ) -> Option<TimeWindow> {
        let windows = match self {
            RowEndpoint::Note(endpoint) => note_spans
                .iter()
                .filter(|span| {
                    span.source_part_index == endpoint.part && span.note_id == endpoint.note_id
                })
                .map(|span| TimeWindow::of_span(span.measure_index, span.timing))
                .collect_vec(),
            RowEndpoint::Lyric(endpoint) => lyric_spans
                .iter()
                .filter(|span| {
                    span.source_part_index == endpoint.part && span.note_id == endpoint.note_id
                })
                .map(|span| TimeWindow::of_span(span.measure_index, span.timing))
                .collect_vec(),
        };
        windows.into_iter().reduce(TimeWindow::union)
    }
}

/// A range between two endpoints in different rows: every note and syllable,
/// in the endpoints' rows and every row between them, that sounds between
/// the earlier endpoint's start and the later one's end. Bounding by time
/// rather than by each note's position in its measure keeps the selection
/// contiguous across parts with different rhythms, and the rows between
/// include lyric rows, so a range never skips one.
///
/// Rows are ordered by `source_part_index`, which is render order (the parser
/// moves every lyric part under its target, and hidden parts have no spans).
/// `Err` if either endpoint's span can't be found.
pub(crate) fn cross_row_range(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: RowEndpoint,
    current: RowEndpoint,
) -> ResolveSelectionRangeResponse {
    let (Some(anchor_window), Some(current_window)) = (
        anchor.window(note_spans, lyric_spans),
        current.window(note_spans, lyric_spans),
    ) else {
        return ResolveSelectionRangeResponse::Err;
    };
    let window = anchor_window.union(current_window);
    let parts = anchor.part().min(current.part())..=anchor.part().max(current.part());
    let in_range = |part: usize, measure_index: usize, timing: MeasureTimingOut| {
        parts.contains(&part) && TimeWindow::of_span(measure_index, timing).overlaps(&window)
    };

    let note_cells = note_spans
        .iter()
        .filter(|span| in_range(span.source_part_index, span.measure_index, span.timing))
        .map(|span| NoteCellOut {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        })
        .unique()
        .collect();
    let lyric_cells = lyric_spans
        .iter()
        .filter(|span| in_range(span.source_part_index, span.measure_index, span.timing))
        .map(|span| LyricCellOut {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        })
        .unique()
        .collect();

    ResolveSelectionRangeResponse::Ok {
        note_cells,
        lyric_cells,
    }
}
