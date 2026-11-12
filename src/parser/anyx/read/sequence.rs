//! Reads `sequence [ … ]`.

use itertools::Itertools;

use crate::error::RecoverableError;
use crate::parser::anyx::decoded::{content_span, decode_error, span, AbbreviationText, WordsText};
use crate::parser::sequence_parser::{PartFilter, PartFilterKind, SequenceEntry, SequenceSection};
use poentaro_grammar::{self as shape, Abbreviation, DecodeError, EntryList, Label, Spanned};

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

/// A label, optionally followed by `WITHOUT [parts]` or `ONLY [parts]`.
fn read_entry(entry: &shape::SequenceEntry) -> Result<SequenceEntry, RecoverableError> {
    let (label, part_filter) = match entry {
        shape::SequenceEntry::Without { label, parts, .. } => {
            (label, part_filter(parts, PartFilterKind::Omit)?)
        }
        shape::SequenceEntry::Only { label, parts, .. } => {
            (label, part_filter(parts, PartFilterKind::Only)?)
        }
        shape::SequenceEntry::Label(label) => (label, None),
    };
    let (label, label_span) = match label {
        Label::Text(text) => (text.value.value.clone(), content_span(text)),
        Label::Words(words) => (words.text(" "), span(words.span)),
    };
    Ok(SequenceEntry {
        label,
        label_span,
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
