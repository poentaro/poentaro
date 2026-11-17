use poentaro::error::{Diagnostic, IrrecoverableError, Warning};

use crate::types::{
    DiagnosticMessageOut, DiagnosticOut, DiagnosticSeverity, DiagnosticViewZoneOut,
};

pub(crate) fn diagnostic_from_error(e: &IrrecoverableError) -> DiagnosticOut {
    let span = e
        .span()
        .map(|s| crate::types::SpanOut {
            start: s.start,
            end: s.end,
        })
        .unwrap_or(crate::types::SpanOut { start: 0, end: 0 });
    DiagnosticOut {
        severity: DiagnosticSeverity::Error,
        message: e.message(),
        span,
    }
}

pub(crate) fn diagnostic_from_warning(e: Warning) -> DiagnosticOut {
    DiagnosticOut {
        severity: DiagnosticSeverity::Warning,
        message: e.message,
        span: crate::types::SpanOut {
            start: e.span.start,
            end: e.span.end,
        },
    }
}

pub(crate) fn diagnostic_from_diagnostic(d: Diagnostic) -> DiagnosticOut {
    match d {
        Diagnostic::Warning(w) => diagnostic_from_warning(w),
        Diagnostic::Error(e) => DiagnosticOut {
            severity: DiagnosticSeverity::Error,
            message: e.message(),
            span: crate::types::SpanOut {
                start: e.span.start,
                end: e.span.end,
            },
        },
    }
}

fn byte_offset_to_line_number(source: &str, byte_offset: usize) -> usize {
    source
        .as_bytes()
        .iter()
        .take(byte_offset.min(source.len()))
        .filter(|&&b| b == b'\n')
        .count()
        + 1
}

struct ViewZoneAccumulator {
    severity: DiagnosticSeverity,
    messages: Vec<DiagnosticMessageOut>,
}

pub(crate) fn group_diagnostics_into_view_zones(
    source: &str,
    diagnostics: &[DiagnosticOut],
) -> Vec<DiagnosticViewZoneOut> {
    use std::collections::BTreeMap;

    let mut groups: BTreeMap<(usize, u8), ViewZoneAccumulator> = BTreeMap::new();

    for d in diagnostics {
        let line = byte_offset_to_line_number(source, d.span.end);
        let severity_order = match d.severity {
            DiagnosticSeverity::Error => 0,
            DiagnosticSeverity::Warning => 1,
        };
        let entry = groups
            .entry((line, severity_order))
            .or_insert_with(|| ViewZoneAccumulator {
                severity: d.severity.clone(),
                messages: Vec::new(),
            });
        entry.messages.push(DiagnosticMessageOut {
            message: d.message.clone(),
        });
    }

    groups
        .into_iter()
        .map(|((line, _), accumulator)| DiagnosticViewZoneOut {
            severity: accumulator.severity,
            after_line_number: line,
            messages: accumulator.messages,
        })
        .collect()
}
