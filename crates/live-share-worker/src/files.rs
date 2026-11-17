//! Pure, D1-free logic for the cloud (D1) storage backend's `files` table --
//! the replacement for the old GitHub Contents API storage backend, unified
//! under the same account identity `crate::identity` already resolves for
//! Synced Share. See `live-share-worker/migrations/0003_files.sql` for the
//! schema this mirrors.
//!
//! There is no single "apply a write" entry point --
//! `crate::handlers`'s `/files/*` routes each perform their own atomic D1
//! `UPDATE`/`INSERT` (see `crate::db`), because rename/delete/restore are
//! unconditional (no revision gate, per the old GitHub backend's actual
//! behavior) while a content save is gated on `expected_revision` as an
//! optimistic-concurrency guard. What stays pure and unit-testable here is
//! *classifying* the outcome of a content-save attempt -- the one op with
//! real conflict semantics to get right.

use std::collections::{HashMap, HashSet};

use poentaro::file_tree::TreePath;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api_error::ApiError;
use crate::protocol::RenameBatchMove;

/// Mirrors a `files` row (see `live-share-worker/migrations/0003_files.sql`).
/// `owner_user_id` is an internal, provider-agnostic `users.id`, never sent back to a client -- see
/// `to_public_file`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredFile {
    pub id: String,
    pub owner_user_id: String,
    pub name: String,
    pub content: String,
    pub revision: i64,
    pub trashed_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// The public shape of a `files` row -- strips `owner_user_id` and the
/// timestamps before a file goes back over the wire, same reasoning as
/// `crate::share::to_public_doc`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublicFile {
    pub id: String,
    pub name: String,
    pub content: String,
    pub revision: i64,
    pub trashed_at: Option<i64>,
}

pub fn to_public_file(file: &StoredFile) -> PublicFile {
    PublicFile {
        id: file.id.clone(),
        name: file.name.clone(),
        content: file.content.clone(),
        revision: file.revision,
        trashed_at: file.trashed_at,
    }
}

/// What a content-write `UPDATE` attempt reported back -- see
/// `queries/update_file_content.sql`'s `WHERE id=? AND owner_user_id=? AND
/// revision=? AND trashed_at IS NULL` guard. `crate::handlers` is
/// responsible for turning a real D1 `D1Result` into one of these (its
/// `meta().changes`, or a follow-up read if that isn't available -- see
/// that module's doc comment) so this classification stays D1-free.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentWriteAttempt {
    Applied,
    RowsAffectedZero,
}

/// The outcome `crate::handlers`'s content-save route turns into an HTTP
/// response: `Applied` -> 200, `Conflict` -> `ApiError::RevisionConflict`
/// (existing "Overwrite mine"/"Discard mine" UI), `NotFound` ->
/// `ApiError::NotFound`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentWriteOutcome {
    Applied { new_revision: i64 },
    Conflict { current_revision: i64 },
    NotFound,
}

/// Classifies a content-write attempt against the row's state *after* the
/// attempt (`current`, from `queries/get_file_by_id.sql` -- `None` if the
/// row doesn't exist or the caller isn't its owner, since that query is
/// scoped by `id` + `owner_user_id`). Pure so the CAS logic is
/// unit-testable without a real D1 database.
///
/// - `Applied`: the `UPDATE` matched a row (it was at `expected_revision`,
///   active, and owned by the caller) -- `new_revision` is
///   `expected_revision + 1`, matching `update_file_content.sql`'s
///   `revision=revision+1`.
/// - `RowsAffectedZero` + a `current` row still present: the row moved on
///   (its `revision` no longer matches `expected_revision`) or was trashed
///   out from under the write -- either way, the client's edit conflicts
///   with the server's current state, so this reports `Conflict` with
///   whatever revision the row is actually at now, for the "Overwrite
///   mine"/"Discard mine" UI.
/// - `RowsAffectedZero` + no `current` row: the file doesn't exist, or
///   isn't owned by the caller -- `NotFound`, not a conflict (there's
///   nothing to reconcile against).
pub fn classify_content_write(
    attempt: ContentWriteAttempt,
    expected_revision: i64,
    current: Option<&StoredFile>,
) -> ContentWriteOutcome {
    match attempt {
        ContentWriteAttempt::Applied => ContentWriteOutcome::Applied {
            new_revision: expected_revision + 1,
        },
        ContentWriteAttempt::RowsAffectedZero => match current {
            Some(file) => ContentWriteOutcome::Conflict {
                current_revision: file.revision,
            },
            None => ContentWriteOutcome::NotFound,
        },
    }
}

/// Checks that `name` is a valid file path: slash-separated, no empty
/// segments, no `.`/`..`, at most 1024 UTF-8 bytes in total. The rules are
/// `poentaro::file_tree::TreePath`'s, shared with the web app. The
/// unique index `idx_files_owner_name` is on the whole string, so two
/// files in different folders may share a last segment.
pub fn validate_file_name(name: &str) -> Result<(), ApiError> {
    TreePath::parse(name)
        .map(drop)
        .map_err(|error| ApiError::InvalidName {
            message: error.to_string(),
        })
}

/// Decides what a unique-constraint violation on `POST /files` means.
/// `existing` is the row with the requested id *scoped to the caller*
/// (`queries/get_file_by_id.sql`), so a row owned by someone else is `None`.
/// A repeated create of the same id and name by the same owner is an
/// idempotent retry and returns the existing file; anything else is a name
/// collision (`None`).
pub fn idempotent_create_match(
    existing: Option<&StoredFile>,
    requested_name: &str,
) -> Option<PublicFile> {
    existing
        .filter(|file| file.name == requested_name)
        .map(to_public_file)
}

/// A file's id and current name: all `plan_batch_rename` needs to know about
/// the rows a batch touches or collides with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileNameRow {
    pub id: String,
    pub name: String,
}

/// What a batch rename should do, decided before anything is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchRenamePlan {
    /// Write every move in one statement.
    Apply,
    /// Every file already has its requested name: a repeat of a batch whose
    /// answer was lost.
    AlreadyApplied,
    /// A file is not the caller's (or does not exist).
    NotFound,
    /// The request itself is contradictory (a file or a name listed twice).
    Invalid,
    /// These requested names are held by files outside the batch.
    NamesTaken(Vec<String>),
}

/// Plans a batch rename. `rows` are the caller's files that are either one
/// of the moved files or hold one of the requested names.
pub fn plan_batch_rename(moves: &[RenameBatchMove], rows: &[FileNameRow]) -> BatchRenamePlan {
    let ids: HashSet<&str> = moves.iter().map(|rename| rename.id.as_str()).collect();
    let names: HashSet<&str> = moves.iter().map(|rename| rename.name.as_str()).collect();
    if ids.len() != moves.len() || names.len() != moves.len() {
        return BatchRenamePlan::Invalid;
    }
    let current: HashMap<&str, &str> = rows
        .iter()
        .map(|row| (row.id.as_str(), row.name.as_str()))
        .collect();
    if moves
        .iter()
        .any(|rename| !current.contains_key(rename.id.as_str()))
    {
        return BatchRenamePlan::NotFound;
    }
    if moves
        .iter()
        .all(|rename| current.get(rename.id.as_str()) == Some(&rename.name.as_str()))
    {
        return BatchRenamePlan::AlreadyApplied;
    }
    // A name held by another file of the batch counts as taken too: the
    // writes would otherwise depend on the order they are applied in.
    let taken: Vec<String> = moves
        .iter()
        .filter(|rename| {
            rows.iter()
                .any(|row| row.name == rename.name && row.id != rename.id)
        })
        .map(|rename| rename.name.clone())
        .collect();
    if taken.is_empty() {
        BatchRenamePlan::Apply
    } else {
        BatchRenamePlan::NamesTaken(taken)
    }
}

/// Finds a write that would make a path both a file and a folder. `live` are
/// the owner's non-trashed files; `proposed` are the names a write is about
/// to give (a new file's, a rename's, a whole batch's), each with its file
/// id. The check runs on the final state: the proposed files replace their
/// current rows in `live`. Returns the first proposed name that equals a
/// folder of another final file, or lies inside another final file.
/// `<folder>/.keep` placeholders are plain files, so they only conflict with
/// a file called `<folder>`.
pub fn find_file_folder_conflict(live: &[FileNameRow], proposed: &[FileNameRow]) -> Option<String> {
    let moved: HashSet<&str> = proposed.iter().map(|row| row.id.as_str()).collect();
    let others: Vec<&FileNameRow> = live
        .iter()
        .filter(|row| !moved.contains(row.id.as_str()))
        .chain(proposed)
        .collect();
    let folders: HashSet<String> = others
        .iter()
        .filter_map(|row| TreePath::parse(&row.name).ok())
        .flat_map(|path| path.ancestors())
        .map(|ancestor| ancestor.as_str().to_string())
        .collect();
    let file_names: HashSet<&str> = others.iter().map(|row| row.name.as_str()).collect();
    proposed
        .iter()
        .find(|row| {
            folders.contains(&row.name)
                || TreePath::parse(&row.name).is_ok_and(|path| {
                    path.ancestors()
                        .iter()
                        .any(|ancestor| file_names.contains(ancestor.as_str()))
                })
        })
        .map(|row| row.name.clone())
}
