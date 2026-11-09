//! Keywords of a document written in anyfix expressions. Sections, fields and
//! part kinds come from its tree. The score's directive keywords come from the
//! score lexer, since the score string holds the section syntax's text.

use anyfix_expr::{Call, NamePart, Node, NodeKind};
use itertools::Itertools;

use super::{HighlightKind, HighlightToken, SectionHeader};
use crate::error::{Diagnostic, RecoverableErrorKind, Span};
use crate::parser::score::interleaved_parser::directive_keyword_spans;
use crate::parser::section::SectionKind;
use crate::parser::{read_document, ReadDocument};

const PART_KINDS: [&str; 5] = ["notes", "chords", "percussion", "follow", "lyrics"];

struct Section<'n> {
    kind: SectionKind,
    keyword: Span,
    call: &'n Call,
}

fn word_spans(call: &Call) -> Vec<Span> {
    call.name
        .iter()
        .filter_map(|part| match part {
            NamePart::Word { span, .. } => Some(Span::new(span.start, span.end)),
            NamePart::Slot => None,
        })
        .collect()
}

fn section(node: &Node) -> Option<Section<'_>> {
    let NodeKind::Call(call) = &node.kind else {
        return None;
    };
    let kind = match call.name_text().as_str() {
        "metadata _" => SectionKind::Metadata,
        "parts _" => SectionKind::Parts,
        "sequence _" => SectionKind::Sequence,
        "preset _ _" => SectionKind::Preset,
        "score _" => SectionKind::Score,
        _ => return None,
    };
    let keyword = *word_spans(call).first()?;
    Some(Section {
        kind,
        keyword,
        call,
    })
}

fn children(node: &Node) -> &[Node] {
    match &node.kind {
        NodeKind::Braces(children) | NodeKind::Brackets(children) | NodeKind::Parens(children) => {
            children
        }
        _ => &[],
    }
}

/// Field names in a `{ … }` record, nested records included.
fn field_keys(node: &Node) -> Vec<HighlightToken> {
    let NodeKind::Braces(fields) = &node.kind else {
        return Vec::new();
    };
    fields
        .iter()
        .flat_map(|field| match &field.kind {
            NodeKind::Call(call) => word_spans(call)
                .into_iter()
                .map(|span| HighlightToken {
                    kind: HighlightKind::MetadataKey,
                    span,
                })
                .chain(call.arguments.iter().flat_map(field_keys))
                .collect_vec(),
            _ => Vec::new(),
        })
        .collect()
}

/// The part kind words of a `parts [ … ]` entry, and its option names.
fn part_tokens(node: &Node) -> Vec<HighlightToken> {
    let NodeKind::Call(call) = &node.kind else {
        return Vec::new();
    };
    let name = call.name_text();
    let is_part_kind = name
        .split(' ')
        .next()
        .is_some_and(|word| PART_KINDS.contains(&word));
    let own = word_spans(call)
        .into_iter()
        .take(usize::from(is_part_kind))
        .map(|span| HighlightToken {
            kind: HighlightKind::PartKind,
            span,
        });
    own.chain(call.arguments.iter().flat_map(|argument| {
        part_tokens(argument)
            .into_iter()
            .chain(field_keys(argument))
            .collect_vec()
    }))
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
    let parsed = anyfix_expr::parse(source);
    let read = read_document(source, &[]);
    let unknown_fields = unknown_field_spans(&read);
    parsed
        .syntax
        .nodes()
        .iter()
        .filter_map(section)
        .flat_map(|section| {
            let header = HighlightToken {
                kind: HighlightKind::SectionHeader,
                span: section.keyword,
            };
            let body = match section.kind {
                SectionKind::Metadata | SectionKind::Preset => section
                    .call
                    .arguments
                    .iter()
                    .flat_map(field_keys)
                    .collect_vec(),
                SectionKind::Parts => section
                    .call
                    .arguments
                    .iter()
                    .flat_map(children)
                    .flat_map(part_tokens)
                    .collect_vec(),
                SectionKind::Sequence | SectionKind::Score => Vec::new(),
            };
            std::iter::once(header).chain(body).collect_vec()
        })
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
    anyfix_expr::parse(source)
        .syntax
        .nodes()
        .iter()
        .filter_map(section)
        .map(|section| SectionHeader {
            kind: section.kind,
            span: section.keyword,
        })
        .collect()
}

#[cfg(test)]
#[path = "anyfix_highlight_tests.rs"]
mod anyfix_highlight_tests;
