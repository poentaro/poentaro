//! Keywords of a document written in anyfix expressions. Sections, fields and
//! part kinds come from its tree. The score's directive keywords come from the
//! score lexer, since the score string holds the section syntax's text.

use itertools::Itertools;

use super::{HighlightKind, HighlightToken, SectionHeader};
use crate::error::{Diagnostic, RecoverableErrorKind, Span};
use crate::parser::anyfix::decoded::{decode, span};
use crate::parser::score::interleaved_parser::directive_keyword_spans;
use crate::parser::section::SectionKind;
use crate::parser::{read_document, ReadDocument};
use anyfix_expr::NamePart;
use poentaro_grammar::{Document, Part, PartKind, PartOption, Record, Value};

fn token(kind: HighlightKind, at: anyfix_expr::Span) -> HighlightToken {
    HighlightToken {
        kind,
        span: span(at),
    }
}

/// Each section's keyword, in source order.
fn headers(document: &Document) -> Vec<SectionHeader> {
    let header = |kind, keyword| SectionHeader {
        kind,
        span: span(keyword),
    };
    [
        document
            .metadata
            .as_ref()
            .map(|section| header(SectionKind::Metadata, section.keyword)),
        document
            .parts
            .as_ref()
            .map(|section| header(SectionKind::Parts, section.keyword)),
        document
            .sequence
            .as_ref()
            .map(|section| header(SectionKind::Sequence, section.keyword)),
        document
            .score
            .as_ref()
            .map(|section| header(SectionKind::Score, section.keyword)),
    ]
    .into_iter()
    .flatten()
    .chain(
        document
            .preset
            .iter()
            .map(|section| header(SectionKind::Preset, section.keyword)),
    )
    .sorted_by_key(|header| header.span.start)
    .collect()
}

/// Field names in a `{ … }` record, nested records included.
fn field_keys(record: &Record) -> Vec<HighlightToken> {
    record
        .field
        .iter()
        .flatten()
        .flat_map(|field| {
            let words = field.name.parts.iter().filter_map(|part| match part {
                NamePart::Word { span, .. } => Some(token(HighlightKind::MetadataKey, *span)),
                NamePart::Slot => None,
            });
            let nested = match &field.arguments {
                Value::Record(record) => field_keys(record),
                _ => Vec::new(),
            };
            words.chain(nested).collect_vec()
        })
        .collect()
}

/// A part's kind word, and its option names.
fn part_tokens(part: &Part) -> Vec<HighlightToken> {
    let (Part::Abbreviated { kind, .. } | Part::Named { kind, .. }) = part else {
        return Vec::new();
    };
    let (keyword, options) = match kind {
        PartKind::Notes {
            keyword, options, ..
        }
        | PartKind::Chords {
            keyword, options, ..
        }
        | PartKind::Percussion {
            keyword, options, ..
        }
        | PartKind::Follow {
            keyword, options, ..
        } => (keyword, options.as_ref()),
        PartKind::Lyrics { keyword, .. } => (keyword, None),
    };
    let option_keys = options
        .into_iter()
        .flat_map(|options| options.part_option.iter().flatten())
        .map(|option| match option {
            PartOption::Instrument { keyword, .. }
            | PartOption::Range { keyword, .. }
            | PartOption::Sound { keyword, .. }
            | PartOption::Volume { keyword, .. }
            | PartOption::Octave { keyword, .. } => token(HighlightKind::MetadataKey, *keyword),
        });
    std::iter::once(token(HighlightKind::PartKind, *keyword))
        .chain(option_keys)
        .collect()
}

fn directive_tokens(read: &ReadDocument) -> Vec<HighlightToken> {
    directive_keyword_spans(&read.score.text, read.score.offset)
        .into_iter()
        .map(|span| HighlightToken {
            kind: HighlightKind::DirectiveKey,
            span,
        })
        .collect()
}

/// The keys the reader doesn't recognize, which stay uncolored.
fn unknown_field_spans(read: &ReadDocument) -> Vec<Span> {
    read.metadata_parse_errors
        .iter()
        .chain(
            read.presets
                .diagnostics
                .iter()
                .filter_map(|diagnostic| match diagnostic {
                    Diagnostic::Error(error) => Some(error),
                    Diagnostic::Warning(_) => None,
                }),
        )
        .filter(|error| {
            matches!(
                error.kind,
                RecoverableErrorKind::MetadataUnknownField { .. }
            )
        })
        .map(|error| error.span)
        .collect()
}

pub(super) fn highlight_tokens(source: &str) -> Vec<HighlightToken> {
    let document = decode(source).document;
    let read = read_document(source, &[]);
    let unknown_fields = unknown_field_spans(&read);
    let headers = headers(&document).into_iter().map(|header| HighlightToken {
        kind: HighlightKind::SectionHeader,
        span: header.span,
    });
    let records = document
        .metadata
        .iter()
        .map(|section| &section.fields)
        .chain(document.preset.iter().map(|section| &section.fields))
        .flat_map(field_keys);
    let parts = document
        .parts
        .iter()
        .flat_map(|section| section.parts.part.iter().flatten())
        .flat_map(part_tokens);
    headers
        .chain(records)
        .chain(parts)
        .filter(|token| {
            !unknown_fields
                .iter()
                .any(|unknown| unknown.start <= token.span.start && token.span.end <= unknown.end)
        })
        .chain(directive_tokens(&read))
        .sorted_by_key(|token| token.span.start)
        .collect()
}

pub(super) fn section_headers(source: &str) -> Vec<SectionHeader> {
    headers(&decode(source).document)
}

#[cfg(test)]
#[path = "anyfix_highlight_tests.rs"]
mod anyfix_highlight_tests;
