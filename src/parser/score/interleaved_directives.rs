use crate::ast::parsed::{KeyChange, ScoreEvent};
use crate::desugar::SourceLine;
use crate::error::{Diagnostic, RecoverableError, Span, Spanned, Warning};
use crate::parser::score::directive_keyword::{
    classify_directive_token, tokenize_directive_tokens, DirectiveKey, DirectiveToken,
};
use crate::parser::score::measure_group::{collect_groups, is_directive_line};

#[path = "interleaved_key_directive.rs"]
mod key_directive;

use key_directive::parse_key_value;

pub(super) struct SplitDirective<'a> {
    pub(super) events: Vec<Spanned<ScoreEvent>>,
    /// The group's lines after its directive line (all of them if it has none).
    pub(super) data_lines: &'a [SourceLine],
    /// Errors first, then warnings.
    pub(super) diagnostics: Vec<Diagnostic>,
}

/// Splits a measure group's directive line off its data lines. `active_key`
/// is the key in effect before this group: a relative `key=+N` resolves
/// against it, and every `key=` on the line updates it.
pub(super) fn split_directive<'a>(
    lines: &'a [SourceLine],
    base_offset: usize,
    active_key: &mut KeyChange,
) -> SplitDirective<'a> {
    match lines.first() {
        Some(directive_line) if is_directive_line(&directive_line.content) => {
            let absolute_offset = base_offset + directive_line.offset;
            let mut diagnostics = DirectiveDiagnostics::default();
            let events = parse_directive_line(
                &directive_line.content,
                absolute_offset,
                active_key,
                &mut diagnostics,
            );
            SplitDirective {
                events,
                data_lines: lines.get(1..).unwrap_or(&[]),
                diagnostics: diagnostics.into_ordered(),
            }
        }
        _ => SplitDirective {
            events: Vec::new(),
            data_lines: lines,
            diagnostics: Vec::new(),
        },
    }
}

#[derive(Default)]
struct DirectiveDiagnostics {
    errors: Vec<RecoverableError>,
    warnings: Vec<Warning>,
}

impl DirectiveDiagnostics {
    fn into_ordered(self) -> Vec<Diagnostic> {
        self.errors
            .into_iter()
            .map(Diagnostic::Error)
            .chain(self.warnings.into_iter().map(Diagnostic::Warning))
            .collect()
    }
}

fn parse_directive_line(
    line: &str,
    line_offset: usize,
    active_key: &mut KeyChange,
    diagnostics: &mut DirectiveDiagnostics,
) -> Vec<Spanned<ScoreEvent>> {
    let inner = line;
    let inner_offset = line_offset;

    let tokens = match tokenize_directive_tokens(inner) {
        Ok(tokens) => tokens,
        Err(_) => {
            let span = Span::new(line_offset, line_offset + line.len());
            diagnostics.errors.push(RecoverableError::general(
                span,
                "unclosed quote in directive line",
            ));
            return Vec::new();
        }
    };

    let mut events = Vec::new();

    for (token, token_inner_offset) in &tokens {
        let token_file_offset = inner_offset + token_inner_offset;
        let span = Span::new(token_file_offset, token_file_offset + token.len());
        let Some(directive) = classify_directive_token(token) else {
            diagnostics.errors.push(RecoverableError::general(
                span,
                format!("unknown directive: '{token}'"),
            ));
            continue;
        };
        if let Some(event) = directive_event(directive, span, active_key, diagnostics) {
            events.push(event);
        }
    }

    events
}

fn directive_event(
    directive: DirectiveToken,
    span: Span,
    active_key: &mut KeyChange,
    diagnostics: &mut DirectiveDiagnostics,
) -> Option<Spanned<ScoreEvent>> {
    let errors = &mut diagnostics.errors;
    use DirectiveToken::{Assignment, Break};
    let event = match directive {
        Break => Some(ScoreEvent::SystemBreak),
        Assignment {
            key: DirectiveKey::Bpm,
            value,
        } => match value.parse::<u32>() {
            Ok(bpm) => Some(ScoreEvent::BpmChange(bpm)),
            Err(_) => {
                errors.push(RecoverableError::general(
                    span,
                    format!("invalid bpm value: {value}"),
                ));
                None
            }
        },
        Assignment {
            key: DirectiveKey::Key,
            value,
        } => parse_key_value(value, span, active_key, diagnostics),
        Assignment {
            key: DirectiveKey::Time,
            value,
        } => parse_time_value(value, span, errors),
        Assignment {
            key: DirectiveKey::Label,
            value,
        } => return parse_label_value(value, span, errors),
        Assignment {
            key: DirectiveKey::MergeDuplicateMeasuresAcrossParts,
            value,
        } => parse_bool_directive_value(value, span, errors)
            .map(ScoreEvent::MergeDuplicateMeasuresAcrossPartsChange),
        Assignment {
            key: DirectiveKey::HideRestingParts,
            value,
        } => {
            parse_bool_directive_value(value, span, errors).map(ScoreEvent::HideRestingPartsChange)
        }
    };
    event.map(|event| Spanned::new(event, span))
}

/// Parses `label="..."`'s value. The event span is narrowed to just the
/// quoted text (not the whole token), so rename-symbol can replace it in place.
fn parse_label_value(
    rest: &str,
    token_span: Span,
    errors: &mut Vec<RecoverableError>,
) -> Option<Spanned<ScoreEvent>> {
    if rest.len() < 2 || !rest.starts_with('"') || !rest.ends_with('"') {
        errors.push(RecoverableError::general(
            token_span,
            format!("label value must be a quoted string, got: {rest}"),
        ));
        return None;
    }
    let text = rest.get(1..rest.len() - 1).unwrap_or_default();
    if text.is_empty() {
        errors.push(RecoverableError::general(
            token_span,
            "label value must not be empty",
        ));
        return None;
    }
    let text_start = token_span.end - rest.len() + 1;
    Some(Spanned::new(
        ScoreEvent::LabelChange(text.to_string()),
        Span::new(text_start, text_start + text.len()),
    ))
}

/// The section label a directive line sets with `label="..."`, if any.
pub(crate) fn directive_line_label(line: &str) -> Option<String> {
    tokenize_directive_tokens(line)
        .ok()?
        .iter()
        .find_map(|(token, _)| match classify_directive_token(token)? {
            DirectiveToken::Assignment {
                key: DirectiveKey::Label,
                value,
            } => value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .filter(|text| !text.is_empty())
                .map(str::to_string),
            _ => None,
        })
}

/// Spans of every directive keyword (`bpm` in `bpm=92`, `break`) on the
/// directive lines of a `score` section's `content`.
pub(crate) fn directive_keyword_spans(content: &str, base_offset: usize) -> Vec<Span> {
    collect_groups(content)
        .iter()
        .filter_map(|group| group.first())
        .filter(|line| is_directive_line(&line.0))
        .flat_map(|(line, line_offset)| {
            let line_start = base_offset + line_offset;
            tokenize_directive_tokens(line)
                .unwrap_or_default()
                .into_iter()
                .filter_map(move |(token, token_offset)| {
                    classify_directive_token(&token).map(|directive| {
                        let start = line_start + token_offset;
                        Span::new(start, start + directive.keyword_len())
                    })
                })
        })
        .collect()
}

fn parse_bool_directive_value(
    value: &str,
    span: Span,
    errors: &mut Vec<RecoverableError>,
) -> Option<bool> {
    match value {
        "yes" => Some(true),
        "no" => Some(false),
        _ => {
            errors.push(RecoverableError::general(
                span,
                format!("invalid value: '{value}', expected 'yes' or 'no'"),
            ));
            None
        }
    }
}

fn parse_time_value(
    value: &str,
    span: Span,
    errors: &mut Vec<RecoverableError>,
) -> Option<ScoreEvent> {
    let Some((numerator_str, denominator_str)) = value.split_once('/') else {
        errors.push(RecoverableError::general(
            span,
            format!("invalid time signature: '{value}'"),
        ));
        return None;
    };
    if numerator_str.contains('/') || denominator_str.contains('/') {
        errors.push(RecoverableError::general(
            span,
            format!("invalid time signature: '{value}'"),
        ));
        return None;
    }
    let numerator = match numerator_str.parse::<u8>() {
        Ok(n) => n,
        Err(_) => {
            errors.push(RecoverableError::general(
                span,
                format!("invalid time numerator: '{numerator_str}'"),
            ));
            return None;
        }
    };
    let denominator = match denominator_str.parse::<u8>() {
        Ok(d) => d,
        Err(_) => {
            errors.push(RecoverableError::general(
                span,
                format!("invalid time denominator: '{denominator_str}'"),
            ));
            return None;
        }
    };
    if denominator == 0 {
        errors.push(RecoverableError::general(
            span,
            "time denominator cannot be zero",
        ));
        return None;
    }
    Some(ScoreEvent::TimeSignatureChange {
        numerator,
        denominator,
    })
}
