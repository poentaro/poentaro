//! Reads a JSON array of `{ id, name, content, revision }` on stdin and
//! writes `{ converted, held }` to stdout: the files whose comments changed,
//! and the names of those held back. Never
//! touches a database.

use std::process::ExitCode;

use migrate_comments::{migrate_all, FileRow};

fn main() -> ExitCode {
    let rows: Vec<FileRow> = match serde_json::from_reader(std::io::stdin().lock()) {
        Ok(rows) => rows,
        Err(error) => {
            eprintln!("found stdin that isn't a JSON array of file rows: {error}");
            return ExitCode::FAILURE;
        }
    };
    let migrations = match migrate_all(&rows) {
        Ok(migrations) => migrations,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };
    match serde_json::to_writer(std::io::stdout().lock(), &migrations) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("failed to write to stdout: {error}");
            ExitCode::FAILURE
        }
    }
}
