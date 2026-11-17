//! What stops the wasm boundary's exports and listings from reading a document.

use poentaro::document::anyx_errors;
use poentaro::parser::parts_parser::InstrumentInfo;

use crate::types::{DiagnosticOut, DiagnosticSeverity, SpanOut};

/// Nothing, or the errors in `source`'s anyx expressions. Part of a document
/// with those errors is missing from what's read, so it isn't exported.
pub(crate) fn open_checked(
    source: &str,
    instruments: &[InstrumentInfo],
) -> Result<(), Vec<DiagnosticOut>> {
    let errors = anyx_errors(source, instruments);
    if errors.is_empty() {
        return Ok(());
    }
    Err(errors
        .iter()
        .map(|error| DiagnosticOut {
            severity: DiagnosticSeverity::Error,
            message: error.message(),
            span: SpanOut {
                start: error.span.start,
                end: error.span.end,
            },
        })
        .collect())
}
