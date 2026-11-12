//! Reads `sequence [ … ]`.

use anyfix_expr::{Node, NodeKind};
use itertools::Itertools;

use crate::error::RecoverableError;
use crate::parser::anyfix::tree::{
    abbreviation_text, as_bare_words, invalid, list_of, name_span, span_of, whole_number,
};
use crate::parser::sequence_parser::{
    LineChoice, PartFilter, PartFilterKind, SequenceEntry, SequenceSection,
};

pub(super) fn read_sequence(body: &Node) -> (Option<SequenceSection>, Vec<RecoverableError>) {
    let mut errors = Vec::new();
    let elements = match list_of(body) {
        Ok(elements) => elements,
        Err(error) => return (None, vec![error]),
    };
    if elements.is_empty() {
        return (None, errors);
    }
    let entries = elements
        .iter()
        .filter_map(|element| entry(element).map_err(|error| errors.push(error)).ok())
        .collect();
    (Some(SequenceSection { entries }), errors)
}

/// A label, optionally followed by `WITH (lines)`, and then by
/// `WITHOUT [parts]` or `ONLY [parts]`.
fn entry(node: &Node) -> Result<SequenceEntry, RecoverableError> {
    let call = match &node.kind {
        NodeKind::Call(call) => Some((call.name_text(), call.arguments.as_slice())),
        _ => None,
    };
    let (label_node, choices, filter) = match call
        .as_ref()
        .map(|(name, arguments)| (name.as_str(), *arguments))
    {
        Some(("_ WITH _", [label, choices])) => (label, Some(choices), None),
        Some(("_ WITHOUT _", [label, parts])) => (label, None, Some((parts, PartFilterKind::Omit))),
        Some(("_ ONLY _", [label, parts])) => (label, None, Some((parts, PartFilterKind::Only))),
        Some(("_ WITH _ WITHOUT _", [label, choices, parts])) => {
            (label, Some(choices), Some((parts, PartFilterKind::Omit)))
        }
        Some(("_ WITH _ ONLY _", [label, choices, parts])) => {
            (label, Some(choices), Some((parts, PartFilterKind::Only)))
        }
        _ => (node, None, None),
    };
    Ok(SequenceEntry {
        label: label_text(label_node)?,
        label_span: name_span(label_node),
        line_choices: choices.map(line_choices).transpose()?.unwrap_or_default(),
        part_filter: filter
            .map(|(parts, kind)| part_filter(parts, kind))
            .transpose()?
            .flatten(),
        span: span_of(node),
    })
}

/// `([w] 2, [a] 1)`, or a lone `[w] 2`.
fn line_choices(node: &Node) -> Result<Vec<LineChoice>, RecoverableError> {
    match &node.kind {
        NodeKind::Parens(choices) => choices.iter().map(line_choice).try_collect(),
        _ => Ok(vec![line_choice(node)?]),
    }
}

/// `[w] 2`: a part and the number of the line it plays.
fn line_choice(node: &Node) -> Result<LineChoice, RecoverableError> {
    let expected = || invalid(node, "expected a part and a line number, like `[w] 2`");
    let NodeKind::Call(call) = &node.kind else {
        return Err(expected());
    };
    let [part, line] = call.arguments.as_slice() else {
        return Err(expected());
    };
    if call.name_text() != "_ _" {
        return Err(expected());
    }
    let line_number = whole_number::<u32>(line)
        .filter(|number| *number >= 1)
        .ok_or_else(|| invalid(line, "a line number counts from 1, like `[w] 2`"))?;
    let [part] = list_of(part)? else {
        return Err(expected());
    };
    Ok(LineChoice {
        part: abbreviation_text(part)?,
        part_span: name_span(part),
        line: line_number,
        span: span_of(node),
    })
}

fn label_text(node: &Node) -> Result<String, RecoverableError> {
    match &node.kind {
        NodeKind::String(literal) => Some(literal.value.clone()),
        _ => as_bare_words(node).map(|words| words.join(" ")),
    }
    .ok_or_else(|| {
        invalid(
            node,
            "expected a label, optionally followed by `WITH ([part] line, …)`, `WITHOUT [parts]` or `ONLY [parts]`",
        )
    })
}

fn part_filter(parts: &Node, kind: PartFilterKind) -> Result<Option<PartFilter>, RecoverableError> {
    let elements = list_of(parts)?;
    if elements.is_empty() {
        return Ok(None);
    }
    Ok(Some(PartFilter {
        kind,
        parts: elements.iter().map(abbreviation_text).try_collect()?,
        part_spans: elements.iter().map(name_span).collect(),
    }))
}
