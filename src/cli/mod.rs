use crate::error::{Diagnostic, IrrecoverableError, IrrecoverableErrorKind, Span};
use std::path::Path;

pub mod generate;

pub use generate::GenerateInput;

#[cfg(feature = "wav")]
pub static SF2_BYTES: &[u8] = include_bytes!("../../fonts/GeneralUser_GS.sf2");

/// Outcome of checking a `.poentaro` file for diagnostics.
pub struct CheckOutcome {
    /// `true` when the file parses with no errors (warnings are still allowed).
    pub ok: bool,
    pub diagnostics: Vec<Diagnostic>,
}

/// Parse and group a `.poentaro` file, returning whether it has any errors and the
/// full list of collected diagnostics.
pub fn check(input: &Path) -> Result<CheckOutcome, IrrecoverableError> {
    let score = parse_and_group(input, None)?;
    let diagnostics = crate::collect_measure_diagnostics(&score);
    let ok = !diagnostics
        .iter()
        .any(|d| matches!(d, Diagnostic::Error(_)));
    Ok(CheckOutcome { ok, diagnostics })
}

pub fn read_source(input: &Path) -> Result<String, IrrecoverableError> {
    std::fs::read_to_string(input).map_err(|e| {
        IrrecoverableError::new(IrrecoverableErrorKind::IoReadFailed {
            span: Span::new(0, 0),
            path: input.to_path_buf(),
            source: e.to_string(),
        })
    })
}

/// Reads a document to compile. A document with anyfix errors is an error
/// listing each one, since part of what it says can't be read.
pub fn read_document_source(input: &Path) -> Result<String, IrrecoverableError> {
    let source = read_source(input)?;
    let errors = crate::document::anyfix_errors(&source, &[]);
    if errors.is_empty() {
        return Ok(source);
    }
    let report = errors
        .iter()
        .map(|error| {
            let position = anyfix_expr::line_column(&source, error.span.start);
            format!(
                "{}:{}:{}: {}",
                input.display(),
                position.line,
                position.column,
                error.message()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Err(IrrecoverableError::new(
        IrrecoverableErrorKind::AnyfixSourceInvalid {
            span: Span::new(0, 0),
            report,
        },
    ))
}

/// Formats a document in place.
pub fn format_file(input: &Path) -> Result<(), IrrecoverableError> {
    let source = read_source(input)?;
    let formatted = crate::format_anyfix::format_anyfix(&source);
    if formatted == source {
        return Ok(());
    }
    write_file(input, formatted.as_bytes())
}

/// Parse and group a `.poentaro` file, with `metadata` overlaid by the
/// `preset` named `preset_name`.
pub(crate) fn parse_and_group(
    input: &Path,
    preset_name: Option<&str>,
) -> Result<crate::ast::grouped::Score, IrrecoverableError> {
    let content = read_document_source(input)?;
    let filename = input.to_string_lossy().to_string();
    crate::compile_with_preset(&content, &filename, &[], preset_name)
        .map_err(|e| e.with_path(input))
}

pub fn write_file(path: &Path, bytes: &[u8]) -> Result<(), IrrecoverableError> {
    std::fs::write(path, bytes).map_err(|e| {
        IrrecoverableError::new(IrrecoverableErrorKind::IoWriteFailed {
            span: Span::new(0, 0),
            path: path.to_path_buf(),
            source: e.to_string(),
        })
    })
}
