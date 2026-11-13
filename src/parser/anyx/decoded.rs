//! What a decoded `.poentaro` document means beyond its shape: the text a value
//! or abbreviation stands for. The shapes come from the `poentaro_grammar` crate.

use anyx::{Node, NodeKind, StringLiteral};
use itertools::Itertools;

use crate::error::{RecoverableError, Span};
use poentaro_grammar::{
    parse_document, Abbreviation, CallName, DecodeError, Document, Field, Spanned, Unconverted,
    Value, Words,
};

pub(crate) fn span(span: anyx::Span) -> Span {
    Span::new(span.start, span.end)
}

pub(crate) fn invalid(at: anyx::Span, message: impl Into<String>) -> RecoverableError {
    RecoverableError::anyx_invalid(span(at), message)
}

pub(crate) fn decode_error(error: &DecodeError) -> RecoverableError {
    invalid(error.span, error.message())
}

/// The source's sections, and what is wrong with the document as a whole: its
/// anyx syntax errors, a repeated section, a node that is no section, and
/// text the converter couldn't read.
pub(crate) struct Decoded {
    pub document: Document,
    pub errors: Vec<RecoverableError>,
}

pub(crate) fn decode(source: &str) -> Decoded {
    let line_strings = line_strings(source);
    let blanked = blank(source, &line_strings);
    let parsed = parse_document(&blanked);
    let mut errors = line_strings
        .iter()
        .map(|at| invalid(*at, "found a `#` line; a comment starts with `##`"))
        .chain(
            parsed
                .syntax_errors
                .iter()
                .map(|error| invalid(error.span, error.message.clone())),
        )
        .collect_vec();
    // Every section is optional, so only a document of no nodes at all could
    // fail as a whole.
    let document = parsed.document.unwrap_or_else(|error| {
        errors.push(decode_error(&error));
        Document {
            unconverted: Vec::new(),
            metadata: None,
            parts: None,
            sequence: None,
            preset: Vec::new(),
            score: None,
            errors: Vec::new(),
            span: anyx::Span::empty(0),
        }
    });
    errors.extend(document.errors.iter().map(decode_error));
    errors.extend(document.unconverted.iter().map(unconverted_error));
    Decoded { document, errors }
}

/// Where `source` has a `#` line string. anyx reads `#` as a line string and
/// `##` as a comment; a document gives line strings no meaning, so each is a
/// comment written with one `#` too few.
fn line_strings(source: &str) -> Vec<anyx::Span> {
    fn collect(source: &str, nodes: &[Node], found: &mut Vec<anyx::Span>) {
        for node in nodes {
            match &node.kind {
                NodeKind::String(_) if source[node.span.start..].starts_with('#') => {
                    found.push(node.span)
                }
                NodeKind::Call(call) => collect(source, &call.arguments, found),
                NodeKind::Parens(nodes) | NodeKind::Brackets(nodes) | NodeKind::Braces(nodes) => {
                    collect(source, nodes, found)
                }
                _ => {}
            }
        }
    }
    let mut found = Vec::new();
    collect(source, &anyx::parse(source).syntax.nodes(), &mut found);
    found
}

/// `source` with the text at `spans` turned into spaces, keeping every byte
/// offset and line break, so the rest of the document reads as if they were
/// comments.
fn blank(source: &str, spans: &[anyx::Span]) -> String {
    spans.iter().rev().fold(source.to_owned(), |mut text, at| {
        let spaces: String = text[at.start..at.end]
            .chars()
            .map(|char| match char {
                '\n' => "\n".to_owned(),
                other => " ".repeat(other.len_utf8()),
            })
            .collect();
        text.replace_range(at.start..at.end, &spaces);
        text
    })
}

/// A string's text without its quotes.
pub(crate) fn content_span(literal: &Spanned<StringLiteral>) -> Span {
    span(literal.content)
}

/// What a `name : value` field's name says.
pub(crate) trait FieldName {
    /// The field's name as a keyword: its words joined with `_`.
    fn key(&self) -> String;
    fn written_key(&self) -> String;
    fn key_span(&self) -> Span;
    /// `could not convert "…"`: text the converter kept because it couldn't read it.
    fn unconverted_text(&self) -> Option<&str>;
    fn unconverted_error(&self) -> Option<RecoverableError>;
}

impl FieldName for Field {
    fn key(&self) -> String {
        self.name.words().join("_")
    }

    fn written_key(&self) -> String {
        self.name.words().join(" ")
    }

    fn key_span(&self) -> Span {
        span(self.name.span)
    }

    fn unconverted_text(&self) -> Option<&str> {
        match (&self.arguments, self.key().as_str()) {
            (Value::Text(text), "could_not_convert") => Some(&text.value.value),
            _ => None,
        }
    }

    fn unconverted_error(&self) -> Option<RecoverableError> {
        let text = self.unconverted_text()?;
        Some(unconverted_error_at(self.span, text))
    }
}

pub(crate) fn unconverted_error(unconverted: &Unconverted) -> RecoverableError {
    unconverted_error_at(unconverted.span, &unconverted.text.value.value)
}

fn unconverted_error_at(at: anyx::Span, text: &str) -> RecoverableError {
    invalid(
        at,
        format!(
            "this could not be converted to the new syntax: {}",
            text.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .join(" / ")
        ),
    )
}

/// A field's value as text.
pub(crate) trait ValueText {
    /// The value as a keyword (`80%`, `-1`, `sans_serif`, `Bb4`), or a pair
    /// as its two values with a space between (`0 -4`).
    fn text(&self) -> Result<String, RecoverableError>;
}

impl ValueText for Value {
    fn text(&self) -> Result<String, RecoverableError> {
        match self {
            Value::Integer(integer) => Ok(integer.value.to_string()),
            Value::Text(text) => Ok(text.value.value.clone()),
            Value::Percentage(percentage) => Ok(format!("{}%", percentage.number.value)),
            Value::Words(words) => Ok(words.text("_")),
            Value::Written {
                name, arguments, ..
            } => joined(name, arguments, "_"),
            Value::Pair { x, y, .. } => Ok(format!("{} {}", x.text()?, y.text()?)),
            Value::Record(record) => Err(not_flat(record.span)),
            Value::List(list) => Err(not_flat(list.span)),
        }
    }
}

/// Words as one piece of text.
pub(crate) trait WordsText {
    /// The words joined with `separator`, except a symbol, which joins its
    /// neighbours without one.
    fn text(&self, separator: &str) -> String;
}

impl WordsText for Words {
    fn text(&self, separator: &str) -> String {
        joined(&self.name, &[], separator).unwrap_or_default()
    }
}

/// The part an abbreviation names.
pub(crate) trait AbbreviationText {
    /// A string's value, or the element's words and symbols joined without
    /// spaces (`A1 & T` is `A1&T`), except that two name words keep one. Read
    /// from the tree, not the source text, so formatting the document (which
    /// spaces out `A1&T` as `A1 & T`) reads the same.
    fn text(&self) -> Result<String, RecoverableError>;
    /// Where the abbreviation's text is: inside a string's quotes, or the
    /// whole element.
    fn text_span(&self) -> Span;
}

impl AbbreviationText for Abbreviation {
    fn text(&self) -> Result<String, RecoverableError> {
        match self {
            Abbreviation::Text(text) => Ok(text.value.value.clone()),
            Abbreviation::Number(number) => Ok(number.value.to_string()),
            Abbreviation::Written {
                name, arguments, ..
            } => joined(name, arguments, " "),
        }
    }

    fn text_span(&self) -> Span {
        match self {
            Abbreviation::Text(text) => content_span(text),
            _ => span(self.span()),
        }
    }
}

/// A call written out flat as one piece of text: `separator` goes between two
/// name words, and nothing between anything else.
fn joined(
    name: &CallName,
    arguments: &[Node],
    separator: &str,
) -> Result<String, RecoverableError> {
    let words = name
        .flat_words(arguments)
        .map_err(|node| not_flat(node.span))?;
    let gaps = std::iter::once("").chain(words.iter().tuple_windows().map(|(before, after)| {
        match before.is_name_or_keyword() && after.is_name_or_keyword() {
            true => separator,
            false => "",
        }
    }));
    Ok(gaps
        .zip(&words)
        .map(|(gap, word)| format!("{gap}{}", word.text))
        .collect())
}

fn not_flat(at: anyx::Span) -> RecoverableError {
    invalid(at, "expected a word, number or string here")
}
