//! Turns the `#` comments in stored scores into `##` comments, now that `#`
//! starts a line string, with the reverse patch History needs.
//!
//! `scripts/migrate-comments-d1.py` fetches the rows, runs the binary on
//! them, and writes the result to D1.

use live_share_worker::file_history::{apply_patch, reverse_patch};
use poentaro::document::anyx_errors;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct FileRow {
    pub id: String,
    pub name: String,
    pub content: String,
    pub revision: i64,
}

/// A file whose comments changed: its new content, and the patch from that
/// back to the old content, for its `file_versions` row.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct FileMigration {
    pub id: String,
    pub name: String,
    pub revision: i64,
    pub converted: String,
    pub reverse_patch: String,
}

/// What `migrate` made of a file.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// It has no `#` comments.
    Unchanged,
    /// It has `#` lines but doesn't read as a document even with them as
    /// comments, such as text in the removed section syntax, where
    /// `# metadata` is a header. Left for a person to fix.
    Held,
    Converted(FileMigration),
}

/// `row` with its comments turned into `##`. An error if the patch wouldn't
/// restore the old content, since History would then show a version that
/// never existed.
pub fn migrate(row: &FileRow) -> Result<Outcome, String> {
    let converted = anyx::migrate_comments(&row.content);
    if converted == row.content {
        return Ok(Outcome::Unchanged);
    }
    if !anyx_errors(&converted, &[]).is_empty() {
        return Ok(Outcome::Held);
    }
    let patch = reverse_patch(&converted, &row.content);
    if apply_patch(&patch, &converted).ok().as_deref() != Some(row.content.as_str()) {
        return Err(format!(
            "found a reverse patch for {} that doesn't restore its content",
            row.name
        ));
    }
    Ok(Outcome::Converted(FileMigration {
        id: row.id.clone(),
        name: row.name.clone(),
        revision: row.revision,
        converted,
        reverse_patch: patch,
    }))
}

/// The files to rewrite, and the names of those held back.
#[derive(Debug, Serialize, Default)]
pub struct Migrations {
    pub converted: Vec<FileMigration>,
    pub held: Vec<String>,
}

pub fn migrate_all(rows: &[FileRow]) -> Result<Migrations, String> {
    rows.iter()
        .try_fold(Migrations::default(), |mut migrations, row| {
            match migrate(row)? {
                Outcome::Unchanged => {}
                Outcome::Held => migrations.held.push(row.name.clone()),
                Outcome::Converted(migration) => migrations.converted.push(migration),
            }
            Ok(migrations)
        })
}
