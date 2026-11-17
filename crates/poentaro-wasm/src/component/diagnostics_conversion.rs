use super::*;

pub(super) fn span_to_wit(span: &crate::types::SpanOut) -> Span {
    Span {
        start: span.start as u32,
        end: span.end as u32,
    }
}

pub(super) fn diagnostic_severity_to_wit(
    severity: &crate::types::DiagnosticSeverity,
) -> DiagnosticSeverity {
    match severity {
        crate::types::DiagnosticSeverity::Error => DiagnosticSeverity::Error,
        crate::types::DiagnosticSeverity::Warning => DiagnosticSeverity::Warning,
    }
}

pub(super) fn diagnostic_to_wit(diagnostic: &crate::types::DiagnosticOut) -> Diagnostic {
    Diagnostic {
        severity: diagnostic_severity_to_wit(&diagnostic.severity),
        message: diagnostic.message.clone(),
        span: span_to_wit(&diagnostic.span),
    }
}

pub(super) fn diagnostics_error_to_wit(
    diagnostics: &[crate::types::DiagnosticOut],
) -> DiagnosticsError {
    DiagnosticsError {
        diagnostics: diagnostics.iter().map(diagnostic_to_wit).collect(),
    }
}

pub(super) fn diagnostic_message_to_wit(
    message: &crate::types::DiagnosticMessageOut,
) -> DiagnosticMessage {
    DiagnosticMessage {
        message: message.message.clone(),
    }
}

pub(super) fn diagnostic_view_zone_to_wit(
    zone: &crate::types::DiagnosticViewZoneOut,
) -> DiagnosticViewZone {
    DiagnosticViewZone {
        severity: diagnostic_severity_to_wit(&zone.severity),
        after_line_number: zone.after_line_number as u32,
        messages: zone
            .messages
            .iter()
            .map(diagnostic_message_to_wit)
            .collect(),
    }
}
