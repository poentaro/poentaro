//! Reads `sequence [ … ]`.

use itertools::Itertools;

use crate::error::RecoverableError;
use crate::parser::anyx::decoded::invalid;
use crate::parser::anyx::decoded::{content_span, decode_error, span, AbbreviationText, WordsText};
use crate::parser::sequence_parser::{
    LineChoice, PartFilter, PartFilterKind, SequenceEntry, SequenceSection,
};
use poentaro_grammar::{
    self as shape, Abbreviation, DecodeError, EntryList, Label, LineChoices, Spanned,
};

pub(super) fn read_sequence(
    entries: &EntryList,
) -> (Option<SequenceSection>, Vec<RecoverableError>) {
    if entries.sequence_entry.is_empty() {
        return (None, Vec::new());
    }
    let (entries, errors): (Vec<_>, Vec<_>) = entries
        .sequence_entry
        .iter()
        .map(|entry| entry.as_ref().map_err(decode_error).and_then(read_entry))
        .partition_result();
    (Some(SequenceSection { entries }), errors)
}

/// A label, optionally followed by `WITH (lines)`, and then by
/// `WITHOUT [parts]` or `ONLY [parts]`.
fn read_entry(entry: &shape::SequenceEntry) -> Result<SequenceEntry, RecoverableError> {
    let (label, lines, part_filter) = match entry {
        shape::SequenceEntry::WithWithout {
            label,
            lines,
            parts,
            ..
        } => (
            label,
            Some(lines),
            part_filter(parts, PartFilterKind::Omit)?,
        ),
        shape::SequenceEntry::WithOnly {
            label,
            lines,
            parts,
            ..
        } => (
            label,
            Some(lines),
            part_filter(parts, PartFilterKind::Only)?,
        ),
        shape::SequenceEntry::With { label, lines, .. } => (label, Some(lines), None),
        shape::SequenceEntry::Without { label, parts, .. } => {
            (label, None, part_filter(parts, PartFilterKind::Omit)?)
        }
        shape::SequenceEntry::Only { label, parts, .. } => {
            (label, None, part_filter(parts, PartFilterKind::Only)?)
        }
        shape::SequenceEntry::Label(label) => (label, None, None),
    };
    let (label, label_span) = match label {
        Label::Text(text) => (text.value.value.clone(), content_span(text)),
        Label::Words(words) => (words.text(" "), span(words.span)),
    };
    Ok(SequenceEntry {
        label,
        label_span,
        line_choices: lines.map(line_choices).transpose()?.unwrap_or_default(),
        part_filter,
        span: span(entry.span()),
    })
}

fn part_filter(
    parts: &Spanned<Vec<Result<Abbreviation, DecodeError>>>,
    kind: PartFilterKind,
) -> Result<Option<PartFilter>, RecoverableError> {
    if parts.value.is_empty() {
        return Ok(None);
    }
    let parts: Vec<&Abbreviation> = parts
        .value
        .iter()
        .map(|part| part.as_ref().map_err(decode_error))
        .try_collect()?;
    Ok(Some(PartFilter {
        kind,
        parts: parts.iter().map(|part| part.text()).try_collect()?,
        part_spans: parts.iter().map(|part| part.text_span()).collect(),
    }))
}

/// `([w] 2, [a] 1)`, or a lone `[w] 2`.
fn line_choices(lines: &LineChoices) -> Result<Vec<LineChoice>, RecoverableError> {
    match lines {
        LineChoices::Many(choices) => choices
            .value
            .iter()
            .map(|choice| choice.as_ref().map_err(decode_error).and_then(line_choice))
            .try_collect(),
        LineChoices::One(choice) => Ok(vec![line_choice(choice)?]),
    }
}

/// `[w] 2`: a part and the number of the line it plays.
fn line_choice(choice: &shape::LineChoice) -> Result<LineChoice, RecoverableError> {
    let line = u32::try_from(choice.line.value)
        .ok()
        .filter(|line| *line >= 1)
        .ok_or_else(|| {
            invalid(
                choice.line.span,
                "a line number counts from 1, like `[w] 2`",
            )
        })?;
    Ok(LineChoice {
        part: choice.part.text()?,
        part_span: choice.part.text_span(),
        line,
        span: span(choice.span),
    })
}
