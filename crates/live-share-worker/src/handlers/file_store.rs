//! What each write to an owner's files does, whoever asks: the `/files/*`
//! REST routes and `poentaro app` both call these once they know the owner.

use worker::D1Database;

use crate::api_error::ApiError;
use crate::db;
use crate::file_history::{self, VersionRow};
use crate::files::{self, ContentWriteAttempt, ContentWriteOutcome, PublicFile};
use crate::folder_share;
use crate::protocol::{FileVersionSummary, RenameBatchMove};

use super::is_unique_constraint_violation;
use super::routes::HandlerResult;

fn now() -> i64 {
    worker::Date::now().as_millis() as i64
}

/// Refuses `proposed` (the names a write is about to give, by file id) when
/// the owner's live files, with those renamed, would have a path that is both
/// a file and a folder.
async fn ensure_no_file_folder_conflict(
    d1: &D1Database,
    owner_user_id: &str,
    proposed: &[files::FileNameRow],
) -> Result<(), ApiError> {
    let live = db::list_live_file_names(d1, owner_user_id).await?;
    match files::find_file_folder_conflict(&live, proposed) {
        Some(path) => Err(ApiError::FileFolderConflict { path }),
        None => Ok(()),
    }
}

pub(crate) fn proposed_name(id: &str, name: &str) -> Vec<files::FileNameRow> {
    vec![files::FileNameRow {
        id: id.to_string(),
        name: name.to_string(),
    }]
}

/// A new file. Idempotent: repeating a create with the same id, owner and
/// name returns the existing file instead of `NameTaken`.
pub(super) async fn create(
    d1: &D1Database,
    owner_user_id: String,
    id: String,
    name: String,
    content: String,
) -> HandlerResult<PublicFile> {
    files::validate_file_name(&name)?;
    ensure_no_file_folder_conflict(d1, &owner_user_id, &proposed_name(&id, &name)).await?;
    let now = now();
    let created = files::StoredFile {
        id,
        owner_user_id,
        name,
        content,
        revision: 0,
        trashed_at: None,
        created_at: now,
        updated_at: now,
    };
    match db::insert_file(d1, &created).await {
        Ok(()) => Ok(files::to_public_file(&created)),
        Err(error) if is_unique_constraint_violation(&error) => {
            let existing = db::get_file_by_id(d1, &created.id, &created.owner_user_id).await?;
            files::idempotent_create_match(existing.as_ref(), &created.name)
                .ok_or(ApiError::NameTaken)
        }
        Err(error) => Err(error.into()),
    }
}

/// The atomic CAS content save: the new revision, `RevisionConflict` when
/// the row moved on since `expected_revision` (or was trashed out from under
/// the write), `NotFound` when the owner has no such file -- see
/// `files::classify_content_write`. An applied save is also recorded in the
/// file's version history (`record_version`).
pub(super) async fn write_content(
    d1: &D1Database,
    owner_user_id: &str,
    id: &str,
    expected_revision: i64,
    content: &str,
) -> HandlerResult<i64> {
    let previous = db::get_file_by_id(d1, id, owner_user_id).await?;
    let attempt =
        db::update_file_content(d1, id, owner_user_id, content, expected_revision, now()).await?;
    let current = match attempt {
        ContentWriteAttempt::Applied => None,
        ContentWriteAttempt::RowsAffectedZero => db::get_file_by_id(d1, id, owner_user_id).await?,
    };
    match files::classify_content_write(attempt, expected_revision, current.as_ref()) {
        ContentWriteOutcome::Applied { new_revision } => {
            // The text at `expected_revision` is the one read above: every
            // content write bumps the revision, so the CAS pins it.
            if let Some(previous) = previous.filter(|file| file.revision == expected_revision) {
                if let Err(error) = record_version(d1, &previous, content, new_revision).await {
                    worker::console_error!("file_versions: failed to record: {error}");
                }
            }
            Ok(new_revision)
        }
        ContentWriteOutcome::Conflict { current_revision } => {
            Err(ApiError::RevisionConflict { current_revision })
        }
        ContentWriteOutcome::NotFound => Err(ApiError::NotFound),
    }
}

/// Records the save that took `previous` to `content` at `new_revision`. A
/// failure here never fails the save: the caller only logs it, and the gap
/// it leaves hides older versions rather than corrupting them (see
/// `crate::file_history`).
pub(crate) async fn record_version(
    d1: &D1Database,
    previous: &files::StoredFile,
    content: &str,
    new_revision: i64,
) -> worker::Result<()> {
    if previous.content == content {
        return db::advance_file_version_link(d1, &previous.id, previous.revision, new_revision)
            .await;
    }
    let version = VersionRow {
        revision: previous.revision,
        newer_revision: new_revision,
        reverse_patch: file_history::reverse_patch(content, &previous.content),
        saved_at: previous.updated_at,
    };
    db::insert_file_version(
        d1,
        &previous.id,
        &version,
        file_history::MAX_VERSIONS_PER_FILE,
    )
    .await
}

/// The file's earlier versions, newest first
/// (`file_history::reachable_versions`).
pub(super) async fn versions(
    d1: &D1Database,
    owner_user_id: &str,
    id: &str,
) -> HandlerResult<Vec<FileVersionSummary>> {
    let file = db::get_file_by_id(d1, id, owner_user_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let rows = db::list_file_versions(d1, id, owner_user_id).await?;
    Ok(file_history::reachable_versions(file.revision, &rows)
        .into_iter()
        .map(|row| FileVersionSummary {
            revision: row.revision,
            saved_at: row.saved_at,
        })
        .collect())
}

/// One earlier version's text, rebuilt from the head. `NotFound` for a
/// missing file or a revision no longer kept.
pub(super) async fn version_content(
    d1: &D1Database,
    owner_user_id: &str,
    id: &str,
    revision: i64,
) -> HandlerResult<String> {
    let file = db::get_file_by_id(d1, id, owner_user_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let rows = db::list_file_versions(d1, id, owner_user_id).await?;
    file_history::rebuild(&file.content, file.revision, &rows, revision)
        .ok_or(ApiError::NotFound)?
        .map_err(|error| ApiError::Internal {
            message: error.message,
        })
}

/// Atomic rename, no revision gate. `NotFound` on zero rows affected; a
/// name collision is `NameTaken`.
pub(super) async fn rename(
    d1: &D1Database,
    owner_user_id: &str,
    id: &str,
    name: &str,
) -> HandlerResult<()> {
    files::validate_file_name(name)?;
    ensure_no_file_folder_conflict(d1, owner_user_id, &proposed_name(id, name)).await?;
    match db::rename_file(d1, id, owner_user_id, name, now()).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(ApiError::NotFound),
        Err(error) if is_unique_constraint_violation(&error) => Err(ApiError::NameTaken),
        Err(error) => Err(error.into()),
    }
}

/// Renames every listed file or none (a folder move). Every name is
/// validated first; then `files::plan_batch_rename` decides from the current
/// rows: `NotFound` if a file is missing, success without writing when every
/// file already has its name (a retry whose answer was lost), `NamesTaken`
/// listing the destinations held by other files. Otherwise one statement
/// writes every move, so a race into a unique-index violation still applies
/// nothing.
pub(super) async fn rename_batch(
    d1: &D1Database,
    owner_user_id: &str,
    moves: &[RenameBatchMove],
) -> HandlerResult<()> {
    moves
        .iter()
        .try_for_each(|rename| files::validate_file_name(&rename.name))?;
    let rows = db::file_names_for_batch(d1, owner_user_id, moves).await?;
    match files::plan_batch_rename(moves, &rows) {
        files::BatchRenamePlan::AlreadyApplied => return Ok(()),
        files::BatchRenamePlan::NotFound => return Err(ApiError::NotFound),
        files::BatchRenamePlan::Invalid => return Err(ApiError::BadRequest),
        files::BatchRenamePlan::NamesTaken(names) => return Err(ApiError::NamesTaken { names }),
        files::BatchRenamePlan::Apply => {}
    }
    let proposed: Vec<files::FileNameRow> = moves
        .iter()
        .map(|rename| files::FileNameRow {
            id: rename.id.clone(),
            name: rename.name.clone(),
        })
        .collect();
    ensure_no_file_folder_conflict(d1, owner_user_id, &proposed).await?;
    // A folder with a synced link that this batch moves as a whole takes the
    // link along: its prefix is re-pointed in the same transaction.
    let prefixes = db::list_folder_share_prefixes(d1, owner_user_id).await?;
    let rewrites = if prefixes.is_empty() {
        Vec::new()
    } else {
        let current = db::active_files_under_prefixes(d1, owner_user_id, &prefixes).await?;
        folder_share::plan_prefix_rewrites(&prefixes, moves, &current)
    };
    match db::rename_files_batch(d1, owner_user_id, moves, &rewrites, now()).await {
        Ok(()) => Ok(()),
        Err(error) if is_unique_constraint_violation(&error) => {
            // Lost a race with another writer: report what is taken now.
            let rows = db::file_names_for_batch(d1, owner_user_id, moves).await?;
            match files::plan_batch_rename(moves, &rows) {
                files::BatchRenamePlan::NamesTaken(names) => Err(ApiError::NamesTaken { names }),
                _ => Err(ApiError::NameTaken),
            }
        }
        Err(error) => Err(error.into()),
    }
}

/// Atomic move to the bin. `NotFound` on zero rows affected (already
/// trashed, wrong owner, or never existed).
pub(super) async fn trash(d1: &D1Database, owner_user_id: &str, id: &str) -> HandlerResult<()> {
    if db::trash_file(d1, id, owner_user_id, now()).await? {
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

/// Atomic restore out of the bin under `name`. `NotFound` on zero rows
/// affected (no such trashed file); a name collision is `NameTaken`; a name
/// that would make a path both a file and a folder is `FileFolderConflict`.
pub(super) async fn restore(
    d1: &D1Database,
    owner_user_id: &str,
    id: &str,
    name: &str,
) -> HandlerResult<()> {
    files::validate_file_name(name)?;
    ensure_no_file_folder_conflict(d1, owner_user_id, &proposed_name(id, name)).await?;
    match db::restore_file(d1, id, owner_user_id, name, now()).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(ApiError::NotFound),
        Err(error) if is_unique_constraint_violation(&error) => Err(ApiError::NameTaken),
        Err(error) => Err(error.into()),
    }
}
