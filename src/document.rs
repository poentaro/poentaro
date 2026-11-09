//! What depends on a document's text as a whole.

use crate::error::{Diagnostic, RecoverableError, RecoverableErrorKind};
use crate::parser::parts_parser::InstrumentInfo;
use crate::parser::read_document;

/// What in `text` doesn't fit the shape of an anyfix document, so part of it
/// can't be read: anything that writes a file from it must stop here.
pub fn anyfix_errors(text: &str, instruments: &[InstrumentInfo]) -> Vec<RecoverableError> {
    let read = read_document(text, instruments);
    read.section_structure_errors
        .into_iter()
        .chain(read.metadata_parse_errors)
        .chain(read.parts_parse_errors)
        .chain(read.sequence_parse_errors)
        .chain(
            read.presets
                .diagnostics
                .into_iter()
                .filter_map(|diagnostic| match diagnostic {
                    Diagnostic::Error(error) => Some(error),
                    Diagnostic::Warning(_) => None,
                }),
        )
        .filter(|error| matches!(error.kind, RecoverableErrorKind::AnyfixInvalid { .. }))
        .collect()
}

#[cfg(test)]
#[path = "document_tests.rs"]
mod document_tests;
