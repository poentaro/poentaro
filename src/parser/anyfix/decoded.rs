//! What a decoded `.poentaro` document means beyond its shape: the text a value
//! or abbreviation stands for, and which section is which. The shapes come from
//! the `poentaro_grammar` crate.

use anyfix_expr::{Node, NodeKind, StringLiteral};
use itertools::Itertools;

use super::words::is_name_word;
use crate::error::{DocumentSection, RecoverableError, Span};
use poentaro_grammar::{
    decode_document, Abbreviation, DecodeError, Document, Field, Part, Record, Section,
    SequenceEntry, SignedInteger, Spanned, Unconverted, Value, Words,
};

pub(crate) fn span(span: anyfix_expr::Span) -> Span {
    Span::new(span.start, span.end)
}

pub(crate) fn invalid(at: anyfix_expr::Span, message: impl Into<String>) -> RecoverableError {
    RecoverableError::anyfix_invalid(span(at), message)
}

pub(crate) fn decode_error(error: &DecodeError) -> RecoverableError {
    invalid(error.span, error.message())
}

/// The source's anyfix syntax errors, and its sections.
pub(crate) struct Decoded {
    pub document: Document,
    pub syntax_errors: Vec<RecoverableError>,
}

pub(crate) fn decode(source: &str) -> Decoded {
    let parsed = anyfix_expr::parse(source);
    let nodes = parsed.syntax.nodes();
    let mut syntax_errors = parsed
        .errors
        .iter()
        .map(|error| invalid(error.span, error.message.clone()))
        .collect_vec();
    // Every section is decoded on its own, so only a document of no nodes at
    // all could fail as a whole.
    let document = decode_document(&nodes).unwrap_or_else(|error| {
        syntax_errors.push(decode_error(&error));
        Document {
            section: Vec::new(),
            span: anyfix_expr::Span::empty(0),
        }
    });
    Decoded {
        document,
        syntax_errors,
    }
}

/// A string's text without its quotes.
pub(crate) fn content_span(literal: &Spanned<StringLiteral>) -> Span {
    span(literal.content)
}

fn field_words(field: &Field) -> impl Iterator<Item = &str> {
    let name = field.name.value.as_str();
    name.strip_suffix(" _").unwrap_or(name).split(' ')
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
        field_words(self).join("_")
    }

    fn written_key(&self) -> String {
        field_words(self).join(" ")
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

fn unconverted_error_at(at: anyfix_expr::Span, text: &str) -> RecoverableError {
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

/// A whole number that may carry a sign.
pub(crate) trait Signed {
    fn value(&self) -> i64;
    /// As written, with its sign: `4`, `+4` or `-4`.
    fn text(&self) -> String;
}

impl Signed for SignedInteger {
    fn value(&self) -> i64 {
        match self {
            SignedInteger::Number(number) | SignedInteger::Plus { number, .. } => number.value,
            SignedInteger::Minus { number, .. } => -number.value,
        }
    }

    fn text(&self) -> String {
        match self {
            SignedInteger::Plus { number, .. } => format!("+{}", number.value),
            _ => self.value().to_string(),
        }
    }
}

/// A field's value as text.
pub(crate) trait ValueText {
    /// The value as a keyword (`80%`, `+1`, `sans_serif`, `Bb4`), or a pair
    /// as its two values with a space between (`0 -4`).
    fn text(&self) -> Result<String, RecoverableError>;
}

impl ValueText for Value {
    fn text(&self) -> Result<String, RecoverableError> {
        match self {
            Value::Integer(integer) => Ok(integer.text()),
            Value::Text(text) => Ok(text.value.value.clone()),
            Value::Percentage(percentage) => Ok(format!("{}%", percentage.number.value)),
            Value::Words(words) => Ok(words.text("_")),
            Value::Written {
                name, arguments, ..
            } => joined_name(&name.value, arguments, "_"),
            Value::Pair { x, y, .. } => Ok(format!("{} {}", x.text()?, y.text()?)),
            Value::Record(Record { span, .. }) => {
                Err(invalid(*span, "expected a word, number or string here"))
            }
            Value::List(list) => Err(invalid(list.span, "expected a word, number or string here")),
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
        joined_name(&self.name.value, &[], separator).unwrap_or_default()
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
            } => joined_name(&name.value, arguments, " "),
        }
    }

    fn text_span(&self) -> Span {
        match self {
            Abbreviation::Text(text) => content_span(text),
            _ => span(self.span()),
        }
    }
}

/// A call's name words with its arguments in their slots, as one piece of text.
fn joined_name(
    name: &str,
    arguments: &[Node],
    separator: &str,
) -> Result<String, RecoverableError> {
    let mut arguments = arguments.iter();
    let mut text = String::new();
    let mut previous_was_name_word = false;
    for word in name.split(' ') {
        if word == "_" {
            if let Some(argument) = arguments.next() {
                text.push_str(&joined_node(argument, separator)?);
            }
            previous_was_name_word = false;
        } else {
            let is_name = is_name_word(word);
            if is_name && previous_was_name_word {
                text.push_str(separator);
            }
            text.push_str(word);
            previous_was_name_word = is_name;
        }
    }
    Ok(text)
}

fn joined_node(node: &Node, separator: &str) -> Result<String, RecoverableError> {
    match &node.kind {
        NodeKind::Number(text) => Ok(text.clone()),
        NodeKind::String(literal) => Ok(literal.value.clone()),
        NodeKind::Call(call) => joined_name(&call.name_text(), &call.arguments, separator),
        _ => Err(invalid(node.span, "expected a word, number or string here")),
    }
}

/// A top-level section: where all of it is, and its body.
pub(crate) struct SectionOf<'d, T> {
    pub whole: anyfix_expr::Span,
    pub body: &'d T,
}

impl<T> Clone for SectionOf<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for SectionOf<'_, T> {}

pub(crate) type PartList = Spanned<Vec<Result<Part, DecodeError>>>;
pub(crate) type EntryList = Spanned<Vec<Result<SequenceEntry, DecodeError>>>;

/// `preset "Name" { … }`.
#[derive(Clone, Copy)]
pub(crate) struct PresetSection<'d> {
    pub whole: anyfix_expr::Span,
    pub name: &'d Spanned<StringLiteral>,
    pub fields: &'d Record,
}

/// A document's top-level sections.
pub(crate) struct Sections<'d> {
    pub metadata: Option<SectionOf<'d, Record>>,
    pub parts: Option<SectionOf<'d, PartList>>,
    pub sequence: Option<SectionOf<'d, EntryList>>,
    pub score: Option<SectionOf<'d, Spanned<StringLiteral>>>,
    pub presets: Vec<PresetSection<'d>>,
    pub errors: Vec<RecoverableError>,
}

impl<'d> Sections<'d> {
    /// Sorts `document`'s sections. `doc_span` locates a repeated one.
    pub(crate) fn of(document: &'d Document, doc_span: Span) -> Self {
        let mut sections = Sections {
            metadata: None,
            parts: None,
            sequence: None,
            score: None,
            presets: Vec::new(),
            errors: Vec::new(),
        };
        for section in &document.section {
            let duplicate = match section {
                Err(error) => {
                    sections.errors.push(decode_error(error));
                    None
                }
                Ok(Section::Unconverted(unconverted)) => {
                    sections.errors.push(unconverted_error(unconverted));
                    None
                }
                Ok(Section::Metadata { fields, span }) => {
                    fill(&mut sections.metadata, *span, fields).then_some(DocumentSection::Metadata)
                }
                Ok(Section::Parts { parts, span }) => {
                    fill(&mut sections.parts, *span, parts).then_some(DocumentSection::Parts)
                }
                Ok(Section::Sequence { entries, span }) => {
                    fill(&mut sections.sequence, *span, entries)
                        .then_some(DocumentSection::Sequence)
                }
                Ok(Section::Score { text, span }) => {
                    fill(&mut sections.score, *span, text).then_some(DocumentSection::Score)
                }
                Ok(Section::Preset { name, fields, span }) => {
                    sections.presets.push(PresetSection {
                        whole: *span,
                        name,
                        fields,
                    });
                    None
                }
            };
            if let Some(section) = duplicate {
                sections
                    .errors
                    .push(RecoverableError::section_duplicate(doc_span, section));
            }
        }
        sections
    }
}

/// Puts the section into an empty `slot`, or says it was taken.
fn fill<'d, T>(slot: &mut Option<SectionOf<'d, T>>, whole: anyfix_expr::Span, body: &'d T) -> bool {
    if slot.is_some() {
        return true;
    }
    *slot = Some(SectionOf { whole, body });
    false
}
