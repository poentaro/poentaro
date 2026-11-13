//! What `POST /agent` decides without D1: which errors stop a score from
//! being saved and which warnings come back with it, a score's SVG and MIDI
//! exports, whether a path can name a new file, and the id a new file gets.
//! The methods themselves are `handlers::agent`, an implementation of
//! the `Server` that `poentaro-protocol` generates from
//! `poentaro.protocol.anyx`.
//!
//! D1-free and `pub` so `tests/agent.rs` can test it.

use itertools::{Either, Itertools};
use poentaro::document::anyx_errors;
use poentaro::error::{Diagnostic as ScoreDiagnostic, IrrecoverableError};
use poentaro::file_tree::{TreePath, FILE_EXTENSION};
use poentaro_protocol::Diagnostic;

/// The name a score is compiled under; it only appears in messages.
const SCORE_FILE_NAME: &str = "score.poentaro";

/// What is wrong with a score, each list in the order it appears.
#[derive(Debug, Default, PartialEq)]
pub struct ScoreDiagnostics {
    /// What stops it from being saved: what doesn't fit a document's shape,
    /// then what the score's compiler rejects.
    pub errors: Vec<Diagnostic>,
    /// What the compiler warns about, which never stops a save.
    pub warnings: Vec<Diagnostic>,
}

/// The errors and warnings in `content`. Nothing names instruments here,
/// so a part's `sound` is never checked against a soundfont.
pub fn score_diagnostics(content: &str) -> ScoreDiagnostics {
    let shape_errors = anyx_errors(content, &[])
        .into_iter()
        .map(|error| ByteError {
            start: error.span.start,
            message: error.message(),
        });
    let (compile_errors, warnings): (Vec<ByteError>, Vec<ByteError>) =
        match poentaro::compile(content, SCORE_FILE_NAME, &[]) {
            Err(error) => (
                vec![ByteError {
                    start: error.span().map_or(0, |span| span.start),
                    message: error.message(),
                }],
                vec![],
            ),
            Ok(score) => poentaro::collect_measure_diagnostics(&score)
                .into_iter()
                .partition_map(|diagnostic| {
                    let found = ByteError {
                        start: diagnostic.span().start,
                        message: diagnostic.message(),
                    };
                    match diagnostic {
                        ScoreDiagnostic::Error(_) => Either::Left(found),
                        ScoreDiagnostic::Warning(_) => Either::Right(found),
                    }
                }),
        };
    ScoreDiagnostics {
        errors: located(content, shape_errors.chain(compile_errors).collect()),
        warnings: located(content, warnings),
    }
}

/// `content` as SVG pages, unless it has errors: the renderer recovers from
/// most of them, so a broken score would otherwise export as a blank page.
/// The worker has no fonts, so text widths are estimated
/// (`poentaro::set_layout_font_bytes` is never called).
pub fn export_svg(content: &str) -> Result<Vec<String>, Vec<Diagnostic>> {
    exported(content, |content| {
        poentaro::render_svgs_from_source(content, SCORE_FILE_NAME, &[]).map(|output| output.svgs)
    })
}

/// `content` as a Standard MIDI File, unless it has errors.
pub fn export_midi(content: &str) -> Result<Vec<u8>, Vec<Diagnostic>> {
    exported(content, |content| {
        poentaro::write_midi_from_source_filtered(content, SCORE_FILE_NAME, None, &[])
    })
}

/// What `export` makes of `content`, or the errors that stop it, the same
/// errors that would stop a save.
fn exported<T>(
    content: &str,
    export: impl FnOnce(&str) -> Result<T, IrrecoverableError>,
) -> Result<T, Vec<Diagnostic>> {
    let errors = score_diagnostics(content).errors;
    if !errors.is_empty() {
        return Err(errors);
    }
    export(content).map_err(|error| {
        located(
            content,
            vec![ByteError {
                start: error.span().map_or(0, |span| span.start),
                message: error.message(),
            }],
        )
    })
}

/// `found` sorted, without repeats, with each start as a line and column.
fn located(content: &str, mut found: Vec<ByteError>) -> Vec<Diagnostic> {
    found.sort_by_key(|error| error.start);
    found.dedup();
    found
        .into_iter()
        .map(|error| {
            let position = line_column(content, error.start);
            Diagnostic {
                line: position.line,
                column: position.column,
                message: error.message,
            }
        })
        .collect()
}

#[derive(Debug, PartialEq, Eq)]
struct ByteError {
    start: usize,
    message: String,
}

/// Where a byte offset is, counted from 1, with the column in characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineColumn {
    pub line: i64,
    pub column: i64,
}

/// The line and column of `offset` in `source`. An offset past the end, or
/// inside a character, counts as the character it is in.
pub fn line_column(source: &str, offset: usize) -> LineColumn {
    let before = source
        .char_indices()
        .take_while(|(index, character)| index + character.len_utf8() <= offset)
        .map(|(_, character)| character);
    let (line, column) = before.fold((1, 1), |(line, column), character| match character {
        '\n' => (line + 1, 1),
        _ => (line, column + 1),
    });
    LineColumn { line, column }
}

/// Why `path` can't name a new file, if it can't: it must be a valid slash
/// path ending in `.poentaro`, the extension the app opens.
pub fn new_file_path_problem(path: &str) -> Option<String> {
    if let Err(error) = TreePath::parse(path) {
        return Some(error.to_string());
    }
    (!path.ends_with(FILE_EXTENSION))
        .then(|| format!("found `{path}`; expected a path ending in `{FILE_EXTENSION}`"))
}

/// `bytes` as a version 4 UUID, the form the web app gives a new file's id
/// (`crypto.randomUUID()`).
pub fn file_id(bytes: [u8; 16]) -> String {
    let mut bytes = bytes;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}
