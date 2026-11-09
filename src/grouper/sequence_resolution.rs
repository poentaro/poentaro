use crate::ast::grouped::{PartFilterDisplay, Score, SequenceSpan};
use crate::ast::parsed::PartDecl;
use crate::error::{Diagnostic, RecoverableError, Span};
use crate::parser::sequence_parser::{PartFilter, PartFilterKind, SequenceSection};

/// Resolves a parsed `sequence` section against `score.measures`' labels
/// and stores the result on `score.sequence`, or leaves it `None` if the
/// section is absent or invalid.
///
/// - Each label must be defined on at most one measure; a duplicate
///   definition is a recoverable error (attached to the second occurrence)
///   and the sequence is dropped, since it would be ambiguous which measure
///   a `sequence` entry refers to.
/// - A `sequence` entry referencing an undefined label is a recoverable
///   document-level error; that entry is skipped but the rest of the
///   sequence still resolves.
/// - A label defined but never referenced by `sequence` is not an error.
/// - An entry's `(-abbrev ...)` / `(abbrev ...)` suffix referencing an
///   abbreviation that matches no declared part is a recoverable
///   document-level error; that abbreviation is dropped from the entry's
///   filter but the rest of the entry still resolves.
pub(super) fn resolve_sequence(
    score: &mut Score,
    sequence: Option<SequenceSection>,
    parse_errors: Vec<RecoverableError>,
    declarations: &[PartDecl],
) {
    score
        .document_diagnostics
        .extend(parse_errors.into_iter().map(Diagnostic::Error));

    let Some(sequence) = sequence else {
        return;
    };

    let Some(label_starts) = collect_unique_label_starts(score) else {
        return;
    };

    let spans = build_spans(&label_starts, score.measures.len());
    score.sequence = Some(resolve_entries(score, sequence, &spans, declarations));
}

/// Whether `abbreviation` matches a declared part.
fn is_declared(abbreviation: &str, declarations: &[PartDecl]) -> bool {
    declarations
        .iter()
        .any(|decl| decl.abbreviation == abbreviation)
}

/// Collects each label's first-occurrence measure index, or attaches a
/// recoverable error and returns `None` if a label is defined more than once.
fn collect_unique_label_starts(score: &mut Score) -> Option<Vec<(String, usize)>> {
    let mut label_starts: Vec<(String, usize)> = Vec::new();
    for (index, measure) in score.measures.iter().enumerate() {
        let Some(label) = &measure.label else {
            continue;
        };
        if label_starts.iter().any(|(l, _)| l == label) {
            score
                .document_diagnostics
                .push(Diagnostic::Error(RecoverableError::general(
                    measure.source_span,
                    format!("label \"{label}\" is defined more than once"),
                )));
            return None;
        }
        label_starts.push((label.clone(), index));
    }
    Some(label_starts)
}

/// Turns each label's start index into an inclusive span running to the
/// measure before the next label (or to the last measure, for the final one).
fn build_spans(label_starts: &[(String, usize)], measure_count: usize) -> Vec<SequenceSpan> {
    let last_measure_index = measure_count.saturating_sub(1);
    label_starts
        .iter()
        .enumerate()
        .map(|(position, (label, start))| {
            let end = label_starts
                .get(position + 1)
                .map(|(_, next_start)| next_start - 1)
                .unwrap_or(last_measure_index);
            SequenceSpan {
                label: label.clone(),
                start: *start,
                end,
                omit_parts: Vec::new(),
                part_filter_display: None,
            }
        })
        .collect()
}

/// Resolves each `sequence` entry against the known spans, attaching a
/// recoverable error and skipping any entry that references an undefined
/// label.
fn resolve_entries(
    score: &mut Score,
    sequence: SequenceSection,
    spans: &[SequenceSpan],
    declarations: &[PartDecl],
) -> Vec<SequenceSpan> {
    sequence
        .entries
        .into_iter()
        .filter_map(
            |entry| match spans.iter().find(|span| span.label == entry.label) {
                Some(span) => {
                    let resolved = resolve_part_filter(
                        score,
                        &entry.label,
                        entry.part_filter.as_ref(),
                        entry.span,
                        declarations,
                    );
                    Some(SequenceSpan {
                        label: span.label.clone(),
                        start: span.start,
                        end: span.end,
                        omit_parts: resolved.omit_parts,
                        part_filter_display: resolved.display,
                    })
                }
                None => {
                    score
                        .document_diagnostics
                        .push(Diagnostic::Error(RecoverableError::general(
                            entry.span,
                            format!("sequence references undefined label \"{}\"", entry.label),
                        )));
                    None
                }
            },
        )
        .collect()
}

/// The result of resolving an entry's `(-abbrev ...)` / `(abbrev ...)`
/// suffix: the part abbreviations to filter out at MIDI-expansion time
/// (`omit_parts`, already expanded from an `Only` filter to its complement
/// against `parts`), and the suffix as written, for display (`display`).
struct ResolvedPartFilter {
    omit_parts: Vec<String>,
    display: Option<PartFilterDisplay>,
}

/// Validates each abbreviation named in `part_filter` against the declared
/// parts, attaching a recoverable error and dropping any abbreviation that
/// matches none. An `Only` filter is then expanded to its complement against
/// `declarations`, since that's the concrete set MIDI-expansion time omits.
fn resolve_part_filter(
    score: &mut Score,
    label: &str,
    part_filter: Option<&PartFilter>,
    span: Span,
    declarations: &[PartDecl],
) -> ResolvedPartFilter {
    let Some(filter) = part_filter else {
        return ResolvedPartFilter {
            omit_parts: Vec::new(),
            display: None,
        };
    };

    let mut validated = Vec::new();
    for abbreviation in &filter.parts {
        if is_declared(abbreviation, declarations) {
            validated.push(abbreviation.clone());
        } else {
            let verb = match filter.kind {
                PartFilterKind::Omit => "omits",
                PartFilterKind::Only => "keeps",
            };
            score
                .document_diagnostics
                .push(Diagnostic::Error(RecoverableError::general(
                    span,
                    format!("sequence entry \"{label}\" {verb} unknown part \"{abbreviation}\""),
                )));
        }
    }

    let omit_parts = match filter.kind {
        PartFilterKind::Omit => validated.clone(),
        PartFilterKind::Only => declarations
            .iter()
            .map(|decl| decl.abbreviation.clone())
            .filter(|abbreviation| !validated.contains(abbreviation))
            .collect(),
    };

    let display = (!validated.is_empty()).then_some(PartFilterDisplay {
        kind: filter.kind,
        parts: validated,
    });

    ResolvedPartFilter {
        omit_parts,
        display,
    }
}

#[cfg(test)]
mod tests;
