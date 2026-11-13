//! Keywords of a document written in anyx expressions. Sections, fields and
//! part kinds come from its tree. The score's directive keywords come from the
//! score lexer, since the score string holds the section syntax's text.

use itertools::Itertools;

use super::{HighlightKind, HighlightToken, SectionHeader};
use crate::error::{Diagnostic, RecoverableErrorKind, Span};
use crate::parser::anyx::decoded::{decode, span};
use crate::parser::score::interleaved_parser::directive_keyword_spans;
use crate::parser::section::SectionKind;
use crate::parser::{read_document, ReadDocument};
use crate::utils::{is_han_char, pinyin_override_at};
use poentaro_grammar::{DecodeError, Document, Keyed, Part, PartKind, PartOption, Value};

fn token(kind: HighlightKind, at: anyx::Span) -> HighlightToken {
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

/// Field keys in a `{ … }` record, a text style's included.
fn field_keys<F: Keyed>(source: &str, fields: &[Result<F, DecodeError>]) -> Vec<HighlightToken> {
    fields
        .iter()
        .flatten()
        .flat_map(|field| {
            let words = key_words(source, field.key_words_span())
                .map(|word| token(HighlightKind::MetadataKey, word));
            let nested = match field.value() {
                Value::Record(record) => field_keys(source, &record.style_field),
                _ => Vec::new(),
            };
            words.chain(nested).collect_vec()
        })
        .collect()
}

/// Each word of a key, so that the spaces between them stay uncolored.
fn key_words(source: &str, key: anyx::Span) -> impl Iterator<Item = anyx::Span> + '_ {
    let text = source.get(key.start..key.end).unwrap_or_default();
    text.split(' ')
        .scan(key.start, |start, word| {
            let span = anyx::Span::new(*start, *start + word.len());
            *start += word.len() + 1;
            Some(span)
        })
        .filter(|span| span.start < span.end)
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
        .flat_map(|section| field_keys(source, &section.fields.metadata_field))
        .chain(
            document
                .preset
                .iter()
                .flat_map(|section| field_keys(source, &section.fields.preset_field)),
        );
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
