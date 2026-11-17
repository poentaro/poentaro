//! Highlighting inside the score string, which anyx sees as one raw string:
//! directive keywords from the score lexer, and lyric readings. Everything
//! outside it is colored by the editor's anyx tokenizer. Section headers come
//! from the document's tree, for the section CodeLens.

use itertools::Itertools;

use super::{HighlightKind, HighlightToken, SectionHeader};
use crate::error::Span;
use crate::parser::anyx::decoded::{decode, span};
use crate::parser::score::interleaved_parser::directive_keyword_spans;
use crate::parser::section::SectionKind;
use crate::parser::{read_document, ReadDocument};
use crate::utils::{is_han_char, pinyin_override_at};
use poentaro_grammar::Document;

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

fn directive_tokens(read: &ReadDocument) -> Vec<HighlightToken> {
    directive_keyword_spans(&read.score.text, read.score.offset)
        .into_iter()
        .map(|span| HighlightToken {
            kind: HighlightKind::DirectiveKey,
            span,
        })
        .collect()
}

/// Every valid `(pinyin)` attached to a Chinese character in the score. Only
/// lyric lines hold Chinese characters, so the whole score is searched.
fn lyric_reading_tokens(read: &ReadDocument) -> Vec<HighlightToken> {
    let text = read.score.text.as_str();
    text.char_indices()
        .filter(|(_, c)| is_han_char(*c))
        .filter_map(|(at, c)| {
            let open = at + c.len_utf8();
            let reading = pinyin_override_at(text, open, read.score.offset)?;
            reading.pinyin.is_ok().then(|| HighlightToken {
                kind: HighlightKind::LyricReading,
                span: Span::new(read.score.offset + open, read.score.offset + reading.end),
            })
        })
        .collect()
}

pub(super) fn highlight_tokens(source: &str) -> Vec<HighlightToken> {
    let read = read_document(source, &[]);
    directive_tokens(&read)
        .into_iter()
        .chain(lyric_reading_tokens(&read))
        .sorted_by_key(|token| token.span.start)
        .collect()
}

pub(super) fn section_headers(source: &str) -> Vec<SectionHeader> {
    headers(&decode(source).document)
}

#[cfg(test)]
#[path = "anyx_highlight_tests.rs"]
mod anyx_highlight_tests;
