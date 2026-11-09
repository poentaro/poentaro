//! [`format_anyfix`]: formats a document (see `syntax.md`).
//!
//! 1. anyfix-expr's canonical formatter lays out the whole document.
//! 2. The score's raw string then goes through the same
//!    cleanups as the current syntax's `score` ([`format_measures`]):
//!    redundant-line removal, part-order sorting, whitespace normalization.
//! 3. The canonical formatter runs again, so the rewritten strings are
//!    indented like everything else.
//!
//! A document with anyfix errors comes back unchanged: laying out text that
//! isn't really anyfix, like a file still in an older syntax, scrambles it. A
//! score string holding a `//` comment is left as it is, since the measure
//! cleanups would drop the comment.

use anyfix_expr::{Node, NodeKind};
use itertools::Itertools;

use crate::ast::parsed::PartDecl;
use crate::format_source::format_measures;
use crate::parser;

/// Formats an `.anyfix` document; see the module docs.
pub fn format_anyfix(source: &str) -> String {
    if !crate::document::anyfix_errors(source, &[]).is_empty() {
        return source.to_string();
    }
    let Ok(formatted) = anyfix_expr::format(source) else {
        return source.to_string();
    };
    let Some(with_measures) = format_score_strings(&formatted) else {
        return formatted;
    };
    anyfix_expr::format(&with_measures).unwrap_or(formatted)
}

/// `source` with the score string's measures formatted, or `None` when
/// nothing changes.
fn format_score_strings(source: &str) -> Option<String> {
    let declarations = declarations(source)?;
    let nodes = anyfix_expr::parse(source).syntax.nodes();
    let replacements = nodes
        .iter()
        .filter_map(score_body)
        .filter_map(|element| match &element.kind {
            NodeKind::String(literal) if !literal.value.contains("//") => {
                let measures = format_measures(&literal.value, 0, &declarations)?;
                (measures.trim_end() != literal.value.trim_end())
                    .then(|| (element.span, raw_string(&measures)))
            }
            _ => None,
        })
        .collect_vec();
    if replacements.is_empty() {
        return None;
    }
    // Back to front, so earlier spans stay valid.
    Some(
        replacements
            .iter()
            .rev()
            .fold(source.to_string(), |mut text, (span, replacement)| {
                text.replace_range(span.start..span.end, replacement);
                text
            }),
    )
}

/// The document's part declarations, when it has any.
fn declarations(source: &str) -> Option<Vec<PartDecl>> {
    let declarations = parser::read_document(source, &[]).declarations;
    (!declarations.is_empty()).then_some(declarations)
}

/// The string of a top-level `score """ … """`.
fn score_body(node: &Node) -> Option<&Node> {
    let NodeKind::Call(call) = &node.kind else {
        return None;
    };
    match (call.name_text().as_str(), call.arguments.as_slice()) {
        ("score _", [body]) => Some(body),
        _ => None,
    }
}

/// Measures as an unindented raw string. The quotes outnumber any run of
/// quotes that starts a line, so no line closes it early; the canonical
/// formatter indents it afterwards.
fn raw_string(measures: &str) -> String {
    let longest_leading_quotes = measures
        .lines()
        .map(|line| line.chars().take_while(|&c| c == '"').count())
        .max()
        .unwrap_or(0);
    let quotes = "\"".repeat((longest_leading_quotes + 1).max(3));
    format!("{quotes}\n{}\n{quotes}", measures.trim_end())
}

#[cfg(test)]
#[path = "format_anyfix_tests.rs"]
mod tests;
