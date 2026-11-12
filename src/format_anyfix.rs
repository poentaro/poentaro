//! [`format_anyfix`]: formats a document (see `syntax.md`).
//!
//! 1. anyx's canonical formatter lays out the whole document.
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

use crate::format_source::format_measures;
use crate::parser;
use crate::parser::anyfix::decoded::{decode, span};

/// Formats an `.anyfix` document; see the module docs.
pub fn format_anyfix(source: &str) -> String {
    if !crate::document::anyfix_errors(source, &[]).is_empty() {
        return source.to_string();
    }
    let Ok(formatted) = anyx::format(source) else {
        return source.to_string();
    };
    let Some(with_measures) = format_score_string(&formatted) else {
        return formatted;
    };
    anyx::format(&with_measures).unwrap_or(formatted)
}

/// `source` with the score string's measures formatted, or `None` when
/// nothing changes.
fn format_score_string(source: &str) -> Option<String> {
    let declarations = parser::read_document(source, &[]).declarations;
    if declarations.is_empty() {
        return None;
    }
    let score = decode(source).document.score?;
    let text = &score.text.value.value;
    if text.contains("//") {
        return None;
    }
    let measures = format_measures(text, 0, &declarations)?;
    (measures.trim_end() != text.trim_end()).then(|| {
        let at = span(score.text.span);
        let mut formatted = source.to_string();
        formatted.replace_range(at.start..at.end, &raw_string(&measures));
        formatted
    })
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
