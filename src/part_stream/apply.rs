//! Edited stream → source: rewrite only the measures whose chunk changed.

use itertools::Itertools;

use crate::error::Span;

use super::diagnostics::has_error;
use super::document::{ScoreDocument, ScoreLine, ScoreMeasure};
use super::editable::{editable_part, EditablePart};
use super::{
    find_part, PartStreamApplyResult, PartStreamError, StreamDiagnostic, StreamDiagnosticSeverity,
    FOLLOW_PLACEHOLDER,
};

/// Replace `span` of the source with `replacement`.
struct SourceEdit {
    span: Span,
    replacement: String,
}

impl SourceEdit {
    fn new(start: usize, end: usize, replacement: impl Into<String>) -> Self {
        Self {
            span: Span::new(start, end),
            replacement: replacement.into(),
        }
    }
}

pub(super) fn apply(source: &str, abbreviation: &str, stream: &str) -> PartStreamApplyResult {
    let loaded = ScoreDocument::load(source).and_then(|document| {
        let index = find_part(&document, abbreviation)?;
        let part = editable_part(&document, index)?;
        Ok(LoadedPart { document, part })
    });
    let LoadedPart { document, part } = match loaded {
        Ok(loaded) => loaded,
        Err(error) => return refused(&error),
    };
    let split = part.grammar.split(stream, &document);
    if has_error(&split.diagnostics) {
        return PartStreamApplyResult {
            source: None,
            diagnostics: split.diagnostics,
            measure_starts: split.measure_starts,
            changed_measure_count: 0,
        };
    }
    let key = abbreviation;
    let mut edits: Vec<SourceEdit> = document
        .measures
        .iter()
        .enumerate()
        .filter_map(|(index, measure)| {
            measure_edit(
                source,
                measure,
                &part,
                key,
                part.chunks.get(index),
                split.chunks.get(index),
            )
        })
        .collect();
    let appended: Vec<&String> = split.chunks.iter().skip(document.measures.len()).collect();
    let changed_measure_count = edits.len() + appended.len();
    if !appended.is_empty() {
        edits.push(appended_measures(source, &document, key, &appended));
    }
    PartStreamApplyResult {
        source: Some(splice(source, edits)),
        diagnostics: split.diagnostics,
        measure_starts: split.measure_starts,
        changed_measure_count,
    }
}

/// New measures past the last one, indented like the score's lines. A score
/// with no measures may be an empty raw string, whose closing quotes keep their
/// own line, or `""`, which becomes a raw string since measures span lines.
fn appended_measures(
    source: &str,
    document: &ScoreDocument,
    key: &str,
    chunks: &[&String],
) -> SourceEdit {
    let offset = document.append_offset;
    let lines = |indent: &str| {
        chunks
            .iter()
            .map(|chunk| format!("{indent}[{key}] {chunk}"))
            .join("\n\n")
    };
    if let Some(measure) = document.measures.first() {
        let text = lines(line_indent(source, measure.start));
        return SourceEdit::new(offset, offset, format!("\n\n{text}"));
    }
    let before = source.get(..offset).unwrap_or_default();
    let after = source.get(offset..).unwrap_or_default();
    if before.ends_with('"') && after.starts_with('"') {
        let indent = "    ";
        let text = format!("\"\"\"\n{}\n{indent}\"\"\"", lines(indent));
        return SourceEdit::new(offset - 1, offset + 1, text);
    }
    let start = line_start(source, offset);
    let line = source.get(start..).unwrap_or_default();
    if line.trim_start_matches([' ', '\t']).starts_with('"') {
        let text = lines(line_indent(source, start));
        return SourceEdit::new(start, start, format!("{text}\n"));
    }
    SourceEdit::new(offset, offset, lines(""))
}

fn line_start(source: &str, offset: usize) -> usize {
    source
        .get(..offset)
        .and_then(|before| before.rfind('\n'))
        .map_or(0, |newline| newline + 1)
}

/// The whitespace that starts the line `offset` sits on.
fn line_indent(source: &str, offset: usize) -> &str {
    let line = source.get(line_start(source, offset)..).unwrap_or_default();
    line.get(..line.len() - line.trim_start_matches([' ', '\t']).len())
        .unwrap_or_default()
}

struct LoadedPart {
    document: ScoreDocument,
    part: EditablePart,
}

fn refused(error: &PartStreamError) -> PartStreamApplyResult {
    PartStreamApplyResult {
        source: None,
        diagnostics: vec![StreamDiagnostic {
            span: Span::new(0, 0),
            message: error.describe(),
            severity: StreamDiagnosticSeverity::Error,
        }],
        measure_starts: Vec::new(),
        changed_measure_count: 0,
    }
}

/// What a measure's new chunk asks of its source line.
enum MeasureTarget<'a> {
    /// Write exactly this content.
    Write(&'a str),
    /// Have no line: past the stream's end, or a follow part's `@`.
    Unwritten,
}

/// The edit that turns `original` into `edited` in one measure, if any.
fn measure_edit(
    source: &str,
    measure: &ScoreMeasure,
    part: &EditablePart,
    key: &str,
    original: Option<&String>,
    edited: Option<&String>,
) -> Option<SourceEdit> {
    if edited.is_some() && edited == original {
        return None;
    }
    let empty_measure = part.grammar.empty_measure(measure.capacity);
    let target = match edited {
        Some(chunk) if chunk == FOLLOW_PLACEHOLDER => MeasureTarget::Unwritten,
        Some(chunk) => MeasureTarget::Write(chunk),
        // An unwritten follow measure copies its target, so a follow part
        // that stops early writes explicit rests instead.
        None if part.grammar.is_follow() => MeasureTarget::Write(&empty_measure),
        None => MeasureTarget::Unwritten,
    };
    let written = measure.written_line(part.index);
    match (target, written) {
        (MeasureTarget::Write(content), Some(line)) => (line.content != content)
            .then(|| SourceEdit::new(line.content_start, line.content_end(), content)),
        (MeasureTarget::Write(content), None) => Some(insert_line(
            source,
            measure,
            part.index,
            &format!("[{key}] {content}"),
        )),
        (MeasureTarget::Unwritten, None) => None,
        // Removing a group's only data line would leave it empty.
        (MeasureTarget::Unwritten, Some(line)) if measure.data_lines.len() == 1 => {
            (line.content != empty_measure).then(|| {
                SourceEdit::new(
                    line.content_start,
                    line.content_end(),
                    empty_measure.clone(),
                )
            })
        }
        (MeasureTarget::Unwritten, Some(line)) => Some(remove_line(measure, line)),
    }
}

/// Removing a line takes its own `\n`, or — for the group's last line — the
/// `\n` before it, so the blank line separating the next group stays intact.
fn remove_line(measure: &ScoreMeasure, line: &ScoreLine) -> SourceEdit {
    if line.line_end < measure.end {
        SourceEdit::new(line.line_start, line.line_end + 1, "")
    } else {
        SourceEdit::new(line.line_start.saturating_sub(1), line.line_end, "")
    }
}

/// A new `[Key] chunk` line, placed before the first line of a part declared
/// after this one, or after the group's last line.
fn insert_line(source: &str, measure: &ScoreMeasure, part_index: usize, line: &str) -> SourceEdit {
    let indent = line_indent(source, measure.start);
    match measure
        .data_lines
        .iter()
        .find(|existing| existing.part_index > part_index)
    {
        Some(later) => SourceEdit::new(
            later.line_start,
            later.line_start,
            format!("{indent}{line}\n"),
        ),
        None => SourceEdit::new(measure.end, measure.end, format!("\n{indent}{line}")),
    }
}

fn splice(source: &str, edits: Vec<SourceEdit>) -> String {
    let mut result = String::with_capacity(source.len());
    let mut cursor = 0;
    for edit in edits.into_iter().sorted_by_key(|edit| edit.span.start) {
        result.push_str(source.get(cursor..edit.span.start).unwrap_or_default());
        result.push_str(&edit.replacement);
        cursor = cursor.max(edit.span.end);
    }
    result.push_str(source.get(cursor..).unwrap_or_default());
    result
}

#[cfg(test)]
#[path = "apply_tests.rs"]
mod tests;
