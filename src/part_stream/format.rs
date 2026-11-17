//! Live re-barring of an edited stream: one measure per line, by changing
//! whitespace only.

use itertools::Itertools;

use crate::error::Span;

use super::document::ScoreDocument;
use super::editable::editable_part;
use super::find_part;
use super::split::SplitStream;

/// A byte offset in the stream, such as the editor's caret.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamCursor {
    pub offset: usize,
}

/// Replace `span` of the stream with `replacement`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamTextEdit {
    pub span: Span,
    pub replacement: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattedPartStream {
    /// Sorted, non-overlapping. Empty when the stream is already formatted,
    /// or when it has errors other than a note or tuplet crossing a barline
    /// (it's then left as typed).
    pub edits: Vec<StreamTextEdit>,
    /// Each input cursor, mapped into the formatted stream.
    pub cursors: Vec<StreamCursor>,
}

/// The whitespace edits that put each of `stream`'s measures on its own
/// line. Whitespace right before a measure becomes one `\n`, any other
/// whitespace holding a line break becomes one space, and leading
/// whitespace is dropped; everything else (spaces inside a measure, the
/// stream's trailing whitespace) stays as typed. The only non-whitespace
/// edit deletes a lyric stream's `_` placeholders pushed past its last slot.
pub fn format_part_stream(
    source: &str,
    abbreviation: &str,
    stream: &str,
    cursors: &[StreamCursor],
) -> FormattedPartStream {
    let edits = formatting_edits(source, abbreviation, stream);
    FormattedPartStream {
        cursors: cursors
            .iter()
            .map(|cursor| StreamCursor {
                offset: mapped_offset(cursor.offset, &edits),
            })
            .collect(),
        edits,
    }
}

/// `stream` split into measures, or `None` when its part can't be loaded.
fn split_part_stream(source: &str, abbreviation: &str, stream: &str) -> Option<SplitStream> {
    let document = ScoreDocument::load(source).ok()?;
    let part = find_part(&document, abbreviation)
        .and_then(|index| editable_part(&document, index))
        .ok()?;
    Some(part.grammar.split(stream, &document))
}

fn formatting_edits(source: &str, abbreviation: &str, stream: &str) -> Vec<StreamTextEdit> {
    let Some(split) = split_part_stream(source, abbreviation, stream) else {
        return Vec::new();
    };
    if split.blocks_rebarring {
        return Vec::new();
    }
    // An empty lyric chunk (a measure without slots) has nothing to put on a line.
    let measure_starts = split
        .measure_starts
        .iter()
        .zip(&split.chunks)
        .filter(|(_, chunk)| !chunk.is_empty())
        .map(|(&start, _)| start)
        .dedup()
        .collect_vec();
    if measure_starts.is_empty() {
        return Vec::new();
    }
    let dropped_placeholders = split.dropped_placeholders;
    let outside_dropped_placeholders =
        |run: &Span| dropped_placeholders.is_none_or(|dropped| run.end <= dropped.start);
    let whitespace_edits = whitespace_runs(stream)
        .filter(|run| run.end < stream.len())
        .filter(outside_dropped_placeholders)
        .filter_map(|run| {
            let text = stream.get(run.start..run.end).unwrap_or_default();
            let replacement = if run.start == 0 {
                ""
            } else if measure_starts.contains(&run.end) {
                "\n"
            } else if text.contains('\n') {
                " "
            } else {
                return None;
            };
            (text != replacement).then(|| StreamTextEdit {
                span: run,
                replacement: replacement.to_string(),
            })
        });
    let inserted_breaks = measure_starts
        .iter()
        .skip(1)
        .filter(|&&start| {
            stream
                .get(..start)
                .and_then(|before| before.chars().next_back())
                .is_some_and(|character| !character.is_whitespace())
        })
        .map(|&start| StreamTextEdit {
            span: Span::new(start, start),
            replacement: "\n".to_string(),
        });
    let deleted_placeholders = dropped_placeholders.map(|span| StreamTextEdit {
        span,
        replacement: String::new(),
    });
    whitespace_edits
        .chain(inserted_breaks)
        .chain(deleted_placeholders)
        .sorted_by_key(|edit| edit.span.start)
        .collect()
}

/// Every maximal run of whitespace in `stream`.
fn whitespace_runs(stream: &str) -> impl Iterator<Item = Span> + '_ {
    stream
        .char_indices()
        .chunk_by(|(_, character)| character.is_whitespace())
        .into_iter()
        .filter(|(is_whitespace, _)| *is_whitespace)
        .filter_map(|(_, mut run)| {
            let (start, first) = run.next()?;
            let end = run
                .last()
                .map_or(start + first.len_utf8(), |(offset, last)| {
                    offset + last.len_utf8()
                });
            Some(Span::new(start, end))
        })
        .collect_vec()
        .into_iter()
}

/// Where `offset` ends up once `edits` are applied. An offset inside an
/// edited span lands at the end of its replacement.
fn mapped_offset(offset: usize, edits: &[StreamTextEdit]) -> usize {
    edits.iter().fold(offset as isize, |mapped, edit| {
        let growth = edit.replacement.len() as isize - (edit.span.end - edit.span.start) as isize;
        if offset > edit.span.end || (offset == edit.span.end && edit.span.start == edit.span.end) {
            mapped + growth
        } else if offset > edit.span.start {
            mapped + (edit.span.start + edit.replacement.len()) as isize - offset as isize
        } else {
            mapped
        }
    }) as usize
}

#[cfg(test)]
#[path = "format_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "format_properties_tests.rs"]
mod properties;
