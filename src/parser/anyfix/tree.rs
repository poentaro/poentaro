//! The shapes a `.poentaro` document's anyfix tree is made of. anyfix-expr does
//! the parsing, so reading a document is telling these shapes apart.

use std::ops::Neg;
use std::str::FromStr;

use anyfix_expr::{Call, NamePart, Node, NodeKind};
use itertools::Itertools;

use super::words::is_name_word;
use crate::error::{RecoverableError, Span};

pub(crate) fn span_of(node: &Node) -> Span {
    Span::new(node.span.start, node.span.end)
}

/// The text a name stands for: a string's content, without its quotes, or the
/// whole node.
pub(crate) fn name_span(node: &Node) -> Span {
    match &node.kind {
        NodeKind::String(literal) => Span::new(literal.content.start, literal.content.end),
        _ => span_of(node),
    }
}

pub(crate) fn invalid(node: &Node, message: impl Into<String>) -> RecoverableError {
    RecoverableError::anyfix_invalid(span_of(node), message)
}

/// `name words : value`, one field of a record.
pub(crate) struct Field<'n> {
    pub words: Vec<&'n str>,
    pub value: &'n Node,
    pub key_span: Span,
}

impl Field<'_> {
    /// The field's name as a keyword: its words joined with `_`.
    pub(crate) fn key(&self) -> String {
        self.words.join("_")
    }

    pub(crate) fn written_key(&self) -> String {
        self.words.join(" ")
    }
}

fn word_spans(call: &Call) -> impl Iterator<Item = Span> + '_ {
    call.name.iter().filter_map(|part| match part {
        NamePart::Word { span, .. } => Some(Span::new(span.start, span.end)),
        NamePart::Slot => None,
    })
}

/// `node` as a field: a call of name words that ends in one slot.
pub(crate) fn as_field(node: &Node) -> Option<Field<'_>> {
    let NodeKind::Call(call) = &node.kind else {
        return None;
    };
    let (last, name) = call.name.split_last()?;
    let words: Vec<&str> = name
        .iter()
        .map(|part| match part {
            NamePart::Word { text, .. } => Some(text.as_str()),
            NamePart::Slot => None,
        })
        .collect::<Option<_>>()?;
    match (last, call.arguments.as_slice()) {
        (NamePart::Slot, [value]) if !words.is_empty() => {
            let first = word_spans(call).next()?;
            let last = word_spans(call).last()?;
            Some(Field {
                words,
                value,
                key_span: Span::new(first.start, last.end),
            })
        }
        _ => None,
    }
}

/// `node` as a call with no slots, in words (`sans serif`).
pub(crate) fn as_bare_words(node: &Node) -> Option<Vec<&str>> {
    let NodeKind::Call(call) = &node.kind else {
        return None;
    };
    call.name
        .iter()
        .map(|part| match part {
            NamePart::Word { text, .. } => Some(text.as_str()),
            NamePart::Slot => None,
        })
        .collect()
}

/// `node`'s words, numbers and strings as one piece of text, with `separator`
/// between two name words. Read from the tree, not the source text, so
/// formatting the document (which spaces out `A1&T` as `A1 & T`) reads the same.
fn joined_text(node: &Node, separator: &str) -> Result<String, RecoverableError> {
    match &node.kind {
        NodeKind::Number(text) => Ok(text.clone()),
        NodeKind::String(literal) => Ok(literal.value.clone()),
        NodeKind::Call(call) => {
            let mut arguments = call.arguments.iter();
            let mut text = String::new();
            let mut previous_was_name_word = false;
            for part in &call.name {
                match part {
                    NamePart::Word { text: word, .. } => {
                        let is_name = is_name_word(word);
                        if is_name && previous_was_name_word {
                            text.push_str(separator);
                        }
                        text.push_str(word);
                        previous_was_name_word = is_name;
                    }
                    NamePart::Slot => {
                        if let Some(argument) = arguments.next() {
                            text.push_str(&joined_text(argument, separator)?);
                        }
                        previous_was_name_word = false;
                    }
                }
            }
            Ok(text)
        }
        _ => Err(invalid(node, "expected a word, number or string here")),
    }
}

/// A whole number: `16`.
pub(crate) fn whole_number<T: FromStr>(node: &Node) -> Option<T> {
    match &node.kind {
        NodeKind::Number(text) => text.parse().ok(),
        _ => None,
    }
}

/// A whole number that may carry a sign: `4`, `+4` or `-4`.
pub(crate) fn signed_number<T: FromStr + Neg<Output = T>>(node: &Node) -> Option<T> {
    let NodeKind::Call(call) = &node.kind else {
        return whole_number(node);
    };
    let ([NamePart::Word { text: sign, .. }, NamePart::Slot], [number]) =
        (call.name.as_slice(), call.arguments.as_slice())
    else {
        return None;
    };
    match sign.as_str() {
        "+" => whole_number(number),
        "-" => whole_number(number).map(T::neg),
        _ => None,
    }
}

/// A percentage: `80%`.
pub(crate) fn percentage<T: FromStr>(node: &Node) -> Option<T> {
    let NodeKind::Call(call) = &node.kind else {
        return None;
    };
    match (call.name.as_slice(), call.arguments.as_slice()) {
        ([NamePart::Slot, NamePart::Word { text, .. }], [number]) if text == "%" => {
            whole_number(number)
        }
        _ => None,
    }
}

/// A value as a keyword (`80%`, `+1`, `sans_serif`, `Bb4`).
pub(crate) fn value_text(node: &Node) -> Result<String, RecoverableError> {
    joined_text(node, "_")
}

/// An abbreviation: a string's value, or the element's words and symbols joined
/// without spaces (`A1 & T` is `A1&T`), except that two name words keep one.
pub(crate) fn abbreviation_text(node: &Node) -> Result<String, RecoverableError> {
    joined_text(node, " ")
}

/// `[ … ]`'s elements.
pub(crate) fn list_of(node: &Node) -> Result<&[Node], RecoverableError> {
    match &node.kind {
        NodeKind::Brackets(elements) => Ok(elements),
        _ => Err(invalid(node, "expected `[ … ]` here")),
    }
}

/// `{ … }`'s elements.
pub(crate) fn record_of(node: &Node) -> Result<&[Node], RecoverableError> {
    match &node.kind {
        NodeKind::Braces(elements) => Ok(elements),
        _ => Err(invalid(node, "expected `{ … }` here")),
    }
}

/// `could not convert "…"`: text the converter kept because it couldn't read it.
pub(crate) fn unconverted_text(node: &Node) -> Option<&str> {
    let NodeKind::Call(call) = &node.kind else {
        return None;
    };
    match (call.name_text().as_str(), call.arguments.as_slice()) {
        ("could not convert _", [argument]) => match &argument.kind {
            NodeKind::String(literal) => Some(&literal.value),
            _ => None,
        },
        _ => None,
    }
}

/// The error for text the converter couldn't read, if `node` holds some.
pub(crate) fn unconverted_error(node: &Node) -> Option<RecoverableError> {
    let text = unconverted_text(node)?;
    Some(invalid(
        node,
        format!(
            "this could not be converted to the new syntax: {}",
            text.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .join(" / ")
        ),
    ))
}

/// `"Name" [Abbr] : kind` or `"Name" : kind`.
pub(crate) struct PartNode<'n> {
    pub display: &'n Node,
    pub abbreviation: Option<&'n Node>,
    pub kind: &'n Node,
}

impl PartNode<'_> {
    /// The one element inside the abbreviation's `[ ]`.
    fn abbreviation_element(&self) -> Option<&Node> {
        match &self.abbreviation?.kind {
            NodeKind::Brackets(elements) => match elements.as_slice() {
                [element] => Some(element),
                _ => None,
            },
            _ => None,
        }
    }

    /// The abbreviation the part goes by: the display name when it has none.
    pub(crate) fn abbreviation_text(&self) -> Option<String> {
        match (self.abbreviation, &self.display.kind) {
            (Some(_), _) => abbreviation_text(self.abbreviation_element()?).ok(),
            (None, NodeKind::String(display)) => Some(display.value.clone()),
            (None, _) => None,
        }
    }
}

pub(crate) fn as_part(node: &Node) -> Option<PartNode<'_>> {
    let NodeKind::Call(call) = &node.kind else {
        return None;
    };
    match (call.name_text().as_str(), call.arguments.as_slice()) {
        ("_ _ _", [display, abbreviation, kind]) => Some(PartNode {
            display,
            abbreviation: Some(abbreviation),
            kind,
        }),
        ("_ _", [display, kind]) => Some(PartNode {
            display,
            abbreviation: None,
            kind,
        }),
        _ => None,
    }
}

/// One top-level section: the whole expression, and its body.
#[derive(Clone, Copy)]
pub(crate) struct SectionNode<'n> {
    pub whole: &'n Node,
    pub body: &'n Node,
}

/// `preset "Name" { … }`.
#[derive(Clone, Copy)]
pub(crate) struct PresetNode<'n> {
    pub whole: &'n Node,
    pub name: &'n Node,
    pub fields: &'n Node,
}

/// A document's top-level sections.
#[derive(Default)]
pub(crate) struct Sections<'n> {
    pub metadata: Option<SectionNode<'n>>,
    pub parts: Option<SectionNode<'n>>,
    pub sequence: Option<SectionNode<'n>>,
    pub score: Option<SectionNode<'n>>,
    pub presets: Vec<PresetNode<'n>>,
    pub errors: Vec<RecoverableError>,
}

impl<'n> Sections<'n> {
    /// Sorts top-level `nodes` into sections. `doc_span` locates a repeated one.
    pub(crate) fn of(nodes: &'n [Node], doc_span: Span) -> Self {
        use crate::error::DocumentSection;
        let mut sections = Sections::default();
        for whole in nodes {
            let Some(call) = (match &whole.kind {
                NodeKind::Call(call) => Some(call),
                _ => None,
            }) else {
                sections
                    .errors
                    .push(invalid(whole, unknown_section_message("this")));
                continue;
            };
            let name = call.name_text();
            let (slot, section) = match (name.as_str(), call.arguments.as_slice()) {
                ("metadata _", [_]) => (&mut sections.metadata, DocumentSection::Metadata),
                ("parts _", [_]) => (&mut sections.parts, DocumentSection::Parts),
                ("sequence _", [_]) => (&mut sections.sequence, DocumentSection::Sequence),
                ("score _", [_]) => (&mut sections.score, DocumentSection::Score),
                ("preset _ _", [name, fields]) => {
                    sections.presets.push(PresetNode {
                        whole,
                        name,
                        fields,
                    });
                    continue;
                }
                _ => {
                    let error = unconverted_error(whole).unwrap_or_else(|| {
                        invalid(whole, unknown_section_message(&format!("`{name}`")))
                    });
                    sections.errors.push(error);
                    continue;
                }
            };
            if slot.is_some() {
                sections
                    .errors
                    .push(RecoverableError::section_duplicate(doc_span, section));
            } else if let Some(body) = call.arguments.first() {
                *slot = Some(SectionNode { whole, body });
            }
        }
        sections
    }
}

fn unknown_section_message(found: &str) -> String {
    format!(
        "found {found} at the top level; expected `metadata {{…}}`, `parts […]`, \
         `sequence […]`, `preset \"Name\" {{…}}` or `score \"\"\"…\"\"\"`"
    )
}
