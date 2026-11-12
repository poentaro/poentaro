//! What `POST /agent` decides without D1: which errors stop a score from
//! being saved, whether a path can name a new file, and the id a new file
//! gets. The methods themselves are `handlers::agent`, an implementation of
//! the `Server` that `poentaro-protocol` generates from
//! `poentaro.protocol.anyx`.
//!
//! D1-free and `pub` so `tests/agent.rs` can test it.

use poentaro::document::anyx_errors;
use poentaro::error::Diagnostic as ScoreDiagnostic;
use poentaro::file_tree::{TreePath, FILE_EXTENSION};
use poentaro_protocol::Diagnostic;

/// The name a score is compiled under; it only appears in messages.
const SCORE_FILE_NAME: &str = "score.poentaro";

/// The errors that stop `content` from being saved, in the order they
/// appear: what doesn't fit a document's shape, then what the score's
/// compiler rejects. Warnings never stop a save. Nothing names instruments
/// here, so a part's `sound` is never checked against a soundfont.
pub fn score_errors(content: &str) -> Vec<Diagnostic> {
    let shape_errors = anyx_errors(content, &[])
        .into_iter()
        .map(|error| ByteError {
            start: error.span.start,
            message: error.message(),
        });
    let compile_errors: Vec<ByteError> = match poentaro::compile(content, SCORE_FILE_NAME, &[]) {
        Err(error) => vec![ByteError {
            start: error.span().map_or(0, |span| span.start),
            message: error.message(),
        }],
        Ok(score) => poentaro::collect_measure_diagnostics(&score)
            .into_iter()
            .filter_map(|diagnostic| match diagnostic {
                ScoreDiagnostic::Error(error) => Some(ByteError {
                    start: error.span.start,
                    message: error.message(),
                }),
                ScoreDiagnostic::Warning(_) => None,
            })
            .collect(),
    };
    let mut errors: Vec<ByteError> = shape_errors.chain(compile_errors).collect();
    errors.sort_by_key(|error| error.start);
    errors.dedup();
    errors
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
