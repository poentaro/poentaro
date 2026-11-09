//! Symbol collection and rename-edit computation for user-renamable identifiers:
//! part abbreviations and `sequence` section labels. Built on top of
//! [`crate::ast::parsed::ParsedDocument`]'s already-tracked declaration and
//! reference spans, for use by editor tooling (e.g. a Monaco rename provider).

use crate::ast::parsed::ParsedDocument;
use crate::error::Span;
use crate::parser::anyfix;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// A part abbreviation.
    Abbreviation,
    /// A `label="..."` section label, referenced from `sequence` entries.
    SectionLabel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OccurrenceRole {
    Declaration,
    Reference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolOccurrence {
    /// The exact renamable text; used for the rename edit and for the
    /// text shown/typed in the editor's rename widget.
    pub span: Span,
    /// The region a caret may rest in to trigger a rename of this occurrence.
    /// Usually equal to `span`, but wider for occurrences whose renamable
    /// text sits inside a larger token (e.g. a section label's `span` covers
    /// just the quoted text in `label="C"`, while `hit_span` covers the
    /// whole `label="C"` token) so placing the caret anywhere in that token
    /// still triggers a rename.
    pub hit_span: Span,
    pub role: OccurrenceRole,
}

impl SymbolOccurrence {
    /// An occurrence whose hit-test region is exactly its renamable text.
    fn exact(span: Span, role: OccurrenceRole) -> Self {
        SymbolOccurrence {
            span,
            hit_span: span,
            role,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub occurrences: Vec<SymbolOccurrence>,
}

/// One replacement to make in the original source in order to rename a symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    pub span: Span,
    pub replacement: String,
}

/// Collects every renamable symbol (part abbreviations, section labels) in
/// `document`, each paired with its declaration and reference occurrences.
pub fn collect_symbols(document: &ParsedDocument) -> Vec<Symbol> {
    let mut abbreviations: Vec<Symbol> = Vec::new();
    let mut labels: Vec<Symbol> = Vec::new();

    collect_abbreviation_symbols(document, &mut abbreviations);
    collect_section_label_symbols(document, &mut labels, &mut abbreviations);

    abbreviations.into_iter().chain(labels).collect()
}

fn collect_abbreviation_symbols(document: &ParsedDocument, abbreviations: &mut Vec<Symbol>) {
    for decl in &document.declarations {
        push_occurrence(
            abbreviations,
            SymbolKind::Abbreviation,
            &decl.abbreviation,
            SymbolOccurrence::exact(decl.abbreviation_span, OccurrenceRole::Declaration),
        );
    }

    for reference in &document.abbreviation_references {
        push_occurrence(
            abbreviations,
            SymbolKind::Abbreviation,
            &reference.abbreviation,
            SymbolOccurrence::exact(reference.span, OccurrenceRole::Reference),
        );
    }
}

fn collect_section_label_symbols(
    document: &ParsedDocument,
    labels: &mut Vec<Symbol>,
    abbreviations: &mut Vec<Symbol>,
) {
    for events in &document.directive_events_per_measure {
        for event in events {
            if let crate::ast::parsed::ScoreEvent::LabelChange(text) = &event.value {
                push_occurrence(
                    labels,
                    SymbolKind::SectionLabel,
                    text,
                    SymbolOccurrence {
                        span: event.span,
                        hit_span: label_token_span(event.span),
                        role: OccurrenceRole::Declaration,
                    },
                );
            }
        }
    }

    let Some(sequence) = &document.sequence else {
        return;
    };
    for entry in &sequence.entries {
        push_occurrence(
            labels,
            SymbolKind::SectionLabel,
            &entry.label,
            SymbolOccurrence::exact(entry.label_span, OccurrenceRole::Reference),
        );
        let Some(filter) = &entry.part_filter else {
            continue;
        };
        for (part, part_span) in filter.parts.iter().zip(&filter.part_spans) {
            push_occurrence(
                abbreviations,
                SymbolKind::Abbreviation,
                part,
                SymbolOccurrence::exact(*part_span, OccurrenceRole::Reference),
            );
        }
    }
}

/// Widens a `label="..."` declaration's quoted-text span (as narrowed by the
/// directive parser, see `label=` handling in `interleaved_directives.rs`)
/// back out to the whole `label="..."` token, so a caret resting anywhere in
/// `label="C"` — not just inside the quotes — still hits this occurrence.
fn label_token_span(text_span: Span) -> Span {
    let prefix_and_quote_len = "label=\"".len();
    Span::new(text_span.start - prefix_and_quote_len, text_span.end + 1)
}

fn push_occurrence(
    symbols: &mut Vec<Symbol>,
    kind: SymbolKind,
    name: &str,
    occurrence: SymbolOccurrence,
) {
    match symbols.iter_mut().find(|s| s.name == name) {
        Some(symbol) => symbol.occurrences.push(occurrence),
        None => symbols.push(Symbol {
            name: name.to_string(),
            kind,
            occurrences: vec![occurrence],
        }),
    }
}

/// Computes the text edits needed to rename every occurrence (declaration and
/// references) of the symbol named `old_name` of the given `kind` to `new_name`.
/// Returns an empty list if no such symbol exists.
pub fn rename_edits(
    document: &ParsedDocument,
    kind: SymbolKind,
    old_name: &str,
    new_name: &str,
) -> Vec<TextEdit> {
    collect_symbols(document)
        .into_iter()
        .find(|symbol| symbol.kind == kind && symbol.name == old_name)
        .map(|symbol| {
            symbol
                .occurrences
                .into_iter()
                .map(|occurrence| TextEdit {
                    span: occurrence.span,
                    replacement: new_name.to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// [`rename_edits`] for `text`, the document's own text: outside its score an
/// anyfix document spells a name by where it sits, quoted or bare.
pub fn rename_edits_in_text(
    text: &str,
    document: &ParsedDocument,
    kind: SymbolKind,
    old_name: &str,
    new_name: &str,
) -> Vec<TextEdit> {
    let edits = rename_edits(document, kind, old_name, new_name);
    let score = crate::parser::read_document(text, &[]).score;
    let score_span = score.offset..score.offset + score.text.len();
    edits
        .into_iter()
        .map(|edit| {
            if score_span.contains(&edit.span.start) {
                return edit;
            }
            let is_in_quotes = text
                .get(..edit.span.start)
                .is_some_and(|before| before.ends_with('"'));
            let replacement = match (is_in_quotes, kind) {
                (true, _) => {
                    let quoted = anyfix::quoted(new_name);
                    quoted.trim_matches('"').to_string()
                }
                (false, SymbolKind::Abbreviation) => anyfix::abbreviation(new_name),
                (false, SymbolKind::SectionLabel) => anyfix::label(new_name),
            };
            TextEdit {
                span: edit.span,
                replacement,
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "symbols_tests.rs"]
mod tests;
