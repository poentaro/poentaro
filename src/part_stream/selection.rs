//! Selections between the part focus editor's stream and the **draft
//! source** (the source with the stream applied), so the editor and the
//! preview can select in each other.
//!
//! A measure's chunk and the focused part's line in the draft hold the same
//! non-whitespace characters, except that the chunk may end with more (the
//! padding `-`s of an untouched measure, a chords lyric part's `|`). So an
//! offset maps by counting non-whitespace characters from the start of its
//! measure.

use itertools::Itertools;

use crate::error::Span;

use super::document::ScoreDocument;
use super::{apply_part_stream, find_part};

/// One measure, located in both the stream and the draft source.
struct MeasureAlignment {
    /// The measure's chunk, up to the next measure's start. The first
    /// measure's starts at 0, so leading whitespace belongs to it.
    stream: Span,
    /// Where the measure's chunk itself starts in the stream.
    chunk_start: usize,
    /// The focused part's content in the draft; `None` when the draft
    /// doesn't write the part in this measure.
    content: Option<Span>,
    /// The measure group in the draft, from its first line to its last.
    group: Span,
}

/// Which side of a run of whitespace an offset lands on.
#[derive(Clone, Copy)]
enum Bias {
    /// Right before the next non-whitespace character: a range's start.
    Before,
    /// Right after the previous non-whitespace character: a range's end.
    After,
}

/// Maps editor selections in `stream` to selections in the draft source
/// (`apply_part_stream(source, abbreviation, stream).source`). A range
/// becomes one range per measure it covers, on the focused part's line;
/// a caret stays a caret, at the start of its measure group when the draft
/// doesn't write the part there. Empty when the stream has errors.
pub fn part_stream_ranges_to_source(
    source: &str,
    abbreviation: &str,
    stream: &str,
    ranges: &[Span],
) -> Vec<Span> {
    let Some((draft, alignments)) = align(source, abbreviation, stream) else {
        return Vec::new();
    };
    ranges
        .iter()
        .map(|range| clamp(stream, *range))
        .flat_map(|range| {
            if range.start == range.end {
                caret_to_source(&draft, stream, &alignments, range.start)
                    .into_iter()
                    .collect_vec()
            } else {
                range_to_source(&draft, stream, &alignments, range)
            }
        })
        .sorted_by_key(|span| (span.start, span.end))
        .dedup()
        .collect()
}

/// Maps draft source selections (such as the preview's note and lyric runs)
/// to selections in `stream`. A range on the focused part's line maps onto
/// its measure's chunk; a range anywhere else in a measure group becomes a
/// caret at that measure's chunk, unless another range already selects in
/// that measure. Empty when the stream has errors.
pub fn source_ranges_to_part_stream(
    source: &str,
    abbreviation: &str,
    stream: &str,
    ranges: &[Span],
) -> Vec<Span> {
    let Some((draft, alignments)) = align(source, abbreviation, stream) else {
        return Vec::new();
    };
    let (on_part, elsewhere): (Vec<Span>, Vec<Span>) = ranges
        .iter()
        .flat_map(|range| range_to_stream(&draft, stream, &alignments, clamp(&draft, *range)))
        .partition_map(|mapped| match mapped {
            Mapped::OnPart(span) => itertools::Either::Left(span),
            Mapped::Elsewhere(span) => itertools::Either::Right(span),
        });
    let selected_measures = on_part
        .iter()
        .filter_map(|span| measure_at(&alignments, span.start))
        .collect_vec();
    let jumps = elsewhere
        .into_iter()
        .filter(|caret| {
            measure_at(&alignments, caret.start)
                .is_none_or(|measure| !selected_measures.contains(&measure))
        })
        .collect_vec();
    on_part
        .into_iter()
        .chain(jumps)
        .sorted_by_key(|span| (span.start, span.end))
        .dedup()
        .collect()
}

/// A source range mapped into the stream.
enum Mapped {
    /// From the focused part's line.
    OnPart(Span),
    /// From another line of the measure group: a caret at its chunk.
    Elsewhere(Span),
}

fn align(
    source: &str,
    abbreviation: &str,
    stream: &str,
) -> Option<(String, Vec<MeasureAlignment>)> {
    let applied = apply_part_stream(source, abbreviation, stream);
    let draft = applied.source?;
    let document = ScoreDocument::load(&draft).ok()?;
    let part_index = find_part(&document, abbreviation).ok()?;
    let ends = applied
        .measure_starts
        .iter()
        .skip(1)
        .copied()
        .chain([stream.len()]);
    let alignments = applied
        .measure_starts
        .iter()
        .zip(ends)
        .zip(&document.measures)
        .enumerate()
        .map(|(index, ((&chunk_start, end), measure))| MeasureAlignment {
            stream: Span::new(if index == 0 { 0 } else { chunk_start }, end),
            chunk_start,
            content: measure
                .written_line(part_index)
                .map(|line| Span::new(line.content_start, line.content_end())),
            group: Span::new(measure.start, measure.end),
        })
        .collect();
    Some((draft, alignments))
}

fn caret_to_source(
    draft: &str,
    stream: &str,
    alignments: &[MeasureAlignment],
    offset: usize,
) -> Option<Span> {
    let alignment = alignments.get(measure_at(alignments, offset)?)?;
    // At the end of its chunk, a caret stays after the last thing written,
    // rather than moving onto the padding the draft adds.
    let rest = stream.get(offset..alignment.stream.end).unwrap_or_default();
    let bias = if rest.trim().is_empty() {
        Bias::After
    } else {
        Bias::Before
    };
    let mapped = match alignment.content {
        Some(content) => translate(stream, alignment.stream, offset, draft, content, bias),
        None => alignment.group.start,
    };
    Some(Span::new(mapped, mapped))
}

fn range_to_source(
    draft: &str,
    stream: &str,
    alignments: &[MeasureAlignment],
    range: Span,
) -> Vec<Span> {
    alignments
        .iter()
        .filter(|alignment| overlaps(alignment.stream, range))
        .filter_map(|alignment| {
            let content = alignment.content?;
            let start = translate(
                stream,
                alignment.stream,
                range.start.max(alignment.stream.start),
                draft,
                content,
                Bias::Before,
            );
            let end = translate(
                stream,
                alignment.stream,
                range.end.min(alignment.stream.end),
                draft,
                content,
                Bias::After,
            );
            (start < end).then(|| Span::new(start, end))
        })
        .collect()
}

fn range_to_stream(
    draft: &str,
    stream: &str,
    alignments: &[MeasureAlignment],
    range: Span,
) -> Vec<Mapped> {
    let on_part = alignments
        .iter()
        .filter_map(|alignment| {
            let content = alignment.content?;
            let touches = if range.start == range.end {
                content.start <= range.start && range.start <= content.end
            } else {
                overlaps(content, range)
            };
            touches.then(|| {
                let start = translate(
                    draft,
                    content,
                    range.start.max(content.start),
                    stream,
                    alignment.stream,
                    Bias::Before,
                );
                let end = translate(
                    draft,
                    content,
                    range.end.min(content.end),
                    stream,
                    alignment.stream,
                    Bias::After,
                );
                Mapped::OnPart(Span::new(start, end.max(start)))
            })
        })
        .collect_vec();
    if !on_part.is_empty() {
        return on_part;
    }
    alignments
        .iter()
        .find(|alignment| {
            alignment.group.start <= range.start && range.start <= alignment.group.end
        })
        .map(|alignment| Mapped::Elsewhere(Span::new(alignment.chunk_start, alignment.chunk_start)))
        .into_iter()
        .collect()
}

/// `offset` (in `from_text`, within `from`) moved to the same place among
/// the non-whitespace characters of `to` (in `to_text`). Past the last one
/// of `to`, it lands right after it.
fn translate(
    from_text: &str,
    from: Span,
    offset: usize,
    to_text: &str,
    to: Span,
    bias: Bias,
) -> usize {
    let before = from_text.get(from.start..offset).unwrap_or_default();
    let count = before
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    let target = to_text.get(to.start..to.end).unwrap_or_default();
    let mut non_whitespace = target
        .char_indices()
        .filter(|(_, character)| !character.is_whitespace());
    let last_end = || target.trim_end().len();
    let relative = match bias {
        Bias::Before => non_whitespace
            .nth(count)
            .map_or_else(last_end, |(index, _)| index),
        Bias::After => match count.checked_sub(1) {
            None => non_whitespace.next().map_or(0, |(index, _)| index),
            Some(last) => non_whitespace
                .nth(last)
                .map_or_else(last_end, |(index, character)| index + character.len_utf8()),
        },
    };
    to.start + relative
}

/// `range` kept inside `text` and on its character boundaries: the editor
/// may report offsets into a newer stream than the last good one it maps
/// through.
fn clamp(text: &str, range: Span) -> Span {
    let floor = |offset: usize| {
        (0..=offset.min(text.len()))
            .rev()
            .find(|&index| text.is_char_boundary(index))
            .unwrap_or_default()
    };
    let start = floor(range.start);
    Span::new(start, floor(range.end).max(start))
}

fn overlaps(left: Span, right: Span) -> bool {
    left.start < right.end && right.start < left.end
}

/// The index of the measure whose stream range holds `offset`.
fn measure_at(alignments: &[MeasureAlignment], offset: usize) -> Option<usize> {
    alignments
        .iter()
        .rposition(|alignment| alignment.stream.start <= offset)
}

#[cfg(test)]
#[path = "selection_tests.rs"]
mod tests;
