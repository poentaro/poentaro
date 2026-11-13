//! Queries against the `files` table (cloud storage backend).

use serde::Deserialize;
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, Result};

use crate::files::{ContentWriteAttempt, FileNameRow, StoredFile};
use crate::folder_share::{ActiveFileName, PrefixRewrite};
use crate::protocol::RenameBatchMove;

const LIST_FILES_BY_OWNER: &str = include_str!("../../queries/list_files_by_owner.sql");
const INSERT_FILE: &str = include_str!("../../queries/insert_file.sql");
const GET_FILE_BY_ID: &str = include_str!("../../queries/get_file_by_id.sql");
const GET_FILE_BY_NAME: &str = include_str!("../../queries/get_file_by_name.sql");
const UPDATE_FILE_CONTENT: &str = include_str!("../../queries/update_file_content.sql");
const RENAME_FILE: &str = include_str!("../../queries/rename_file.sql");
const LIST_LIVE_FILE_NAMES: &str = include_str!("../../queries/list_live_file_names.sql");
const GET_FILE_NAMES_BY_IDS_OR_NAMES: &str =
    include_str!("../../queries/get_file_names_by_ids_or_names.sql");
const RENAME_FILES_BATCH: &str = include_str!("../../queries/rename_files_batch.sql");
const LIST_FOLDER_SHARE_PREFIXES: &str =
    include_str!("../../queries/list_folder_share_prefixes.sql");
const LIST_ACTIVE_FILES_UNDER_PREFIXES: &str =
    include_str!("../../queries/list_active_files_under_prefixes.sql");
const REWRITE_FOLDER_SHARE_PREFIX: &str =
    include_str!("../../queries/rewrite_folder_share_prefix.sql");
const TRASH_FILE: &str = include_str!("../../queries/trash_file.sql");
const RESTORE_FILE: &str = include_str!("../../queries/restore_file.sql");

/// Raw row shape from the `files` queries below -- no column needs
/// converting (D1/SQLite's `INTEGER` maps straight onto
/// `Option<i64>` for `trashed_at`), but a separate row type is kept anyway
/// so `crate::files` (the pure, D1-free module) never has to derive
/// `serde::Deserialize` for D1's sake.
#[derive(Deserialize)]
struct StoredFileRow {
    id: String,
    owner_user_id: String,
    name: String,
    content: String,
    revision: i64,
    trashed_at: Option<i64>,
    created_at: i64,
    updated_at: i64,
}

impl From<StoredFileRow> for StoredFile {
    fn from(row: StoredFileRow) -> Self {
        StoredFile {
            id: row.id,
            owner_user_id: row.owner_user_id,
            name: row.name,
            content: row.content,
            revision: row.revision,
            trashed_at: row.trashed_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// All of an owner's files, active and trashed alike -- `crate::handlers`
/// partitions the result by `trashed_at` before it reaches the client.
pub(crate) async fn list_files_by_owner(
    db: &D1Database,
    owner_user_id: &str,
) -> Result<Vec<StoredFile>> {
    let rows: Vec<StoredFileRow> = db
        .prepare(LIST_FILES_BY_OWNER)
        .bind(&[JsValue::from_str(owner_user_id)])?
        .all()
        .await?
        .results()?;
    Ok(rows.into_iter().map(StoredFile::from).collect())
}

/// Creates a new file row (new file / duplicate / import). Propagates a D1
/// error as-is, including a `idx_files_owner_name` unique-constraint
/// violation on a name-collision race -- `crate::handlers` maps that one to
/// an `ApiError::NameTaken` response via `is_unique_constraint_violation`.
pub(crate) async fn insert_file(db: &D1Database, file: &StoredFile) -> Result<()> {
    db.prepare(INSERT_FILE)
        .bind(&[
            JsValue::from_str(&file.id),
            JsValue::from_str(&file.owner_user_id),
            JsValue::from_str(&file.name),
            JsValue::from_str(&file.content),
            JsValue::from_f64(file.created_at as f64),
            JsValue::from_f64(file.updated_at as f64),
        ])?
        .run()
        .await?;
    Ok(())
}

/// Scoped by `id` + `owner_user_id` -- `None` covers both "no such file"
/// and "not this caller's file", indistinguishable by design (this worker
/// never reveals whether a given id belongs to someone else).
pub(crate) async fn get_file_by_id(
    db: &D1Database,
    id: &str,
    owner_user_id: &str,
) -> Result<Option<StoredFile>> {
    let row: Option<StoredFileRow> = db
        .prepare(GET_FILE_BY_ID)
        .bind(&[JsValue::from_str(id), JsValue::from_str(owner_user_id)])?
        .first(None)
        .await?;
    Ok(row.map(StoredFile::from))
}

/// The owner's file at the path `name`, active or trashed.
pub(crate) async fn get_file_by_name(
    db: &D1Database,
    owner_user_id: &str,
    name: &str,
) -> Result<Option<StoredFile>> {
    let row: Option<StoredFileRow> = db
        .prepare(GET_FILE_BY_NAME)
        .bind(&[JsValue::from_str(owner_user_id), JsValue::from_str(name)])?
        .first(None)
        .await?;
    Ok(row.map(StoredFile::from))
}

/// The atomic CAS content-save write. Reads the D1 result's `meta().changes`
/// to tell "applied" from "zero rows affected" (revision moved on, file was
/// trashed, or it isn't this caller's file) -- see
/// `crate::files::classify_content_write`, which turns this into the
/// caller-facing outcome once combined with a `get_file_by_id` lookup on the
/// zero-rows case.
pub(crate) async fn update_file_content(
    db: &D1Database,
    id: &str,
    owner_user_id: &str,
    content: &str,
    expected_revision: i64,
    now: i64,
) -> Result<ContentWriteAttempt> {
    let result = db
        .prepare(UPDATE_FILE_CONTENT)
        .bind(&[
            JsValue::from_str(content),
            JsValue::from_f64(now as f64),
            JsValue::from_str(id),
            JsValue::from_str(owner_user_id),
            JsValue::from_f64(expected_revision as f64),
        ])?
        .run()
        .await?;
    rows_changed(&result)
}

/// Atomic rename, no revision gate. Returns whether a row was actually
/// affected (`false` -> 404: no such file for this caller) -- can also
/// propagate a `idx_files_owner_name` unique-constraint violation on a
/// name-collision race, same as `insert_file`.
pub(crate) async fn rename_file(
    db: &D1Database,
    id: &str,
    owner_user_id: &str,
    name: &str,
    now: i64,
) -> Result<bool> {
    let result = db
        .prepare(RENAME_FILE)
        .bind(&[
            JsValue::from_str(name),
            JsValue::from_f64(now as f64),
            JsValue::from_str(id),
            JsValue::from_str(owner_user_id),
        ])?
        .run()
        .await?;
    Ok(rows_changed(&result)? == ContentWriteAttempt::Applied)
}

#[derive(Deserialize)]
struct FileNameRowRaw {
    id: String,
    name: String,
}

fn to_json(value: &impl serde::Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(|error| worker::Error::RustError(error.to_string()))
}

/// The id and name of every active file of the owner (see
/// `files::find_file_folder_conflict`).
pub(crate) async fn list_live_file_names(
    db: &D1Database,
    owner_user_id: &str,
) -> Result<Vec<FileNameRow>> {
    let rows: Vec<FileNameRowRaw> = db
        .prepare(LIST_LIVE_FILE_NAMES)
        .bind(&[JsValue::from_str(owner_user_id)])?
        .all()
        .await?
        .results()?;
    Ok(rows
        .into_iter()
        .map(|row| FileNameRow {
            id: row.id,
            name: row.name,
        })
        .collect())
}

/// The caller's files that are one of `moves`' files or hold one of their
/// requested names (see `files::plan_batch_rename`).
pub(crate) async fn file_names_for_batch(
    db: &D1Database,
    owner_user_id: &str,
    moves: &[RenameBatchMove],
) -> Result<Vec<FileNameRow>> {
    let ids: Vec<&str> = moves.iter().map(|rename| rename.id.as_str()).collect();
    let names: Vec<&str> = moves.iter().map(|rename| rename.name.as_str()).collect();
    let rows: Vec<FileNameRowRaw> = db
        .prepare(GET_FILE_NAMES_BY_IDS_OR_NAMES)
        .bind(&[
            JsValue::from_str(owner_user_id),
            JsValue::from_str(&to_json(&ids)?),
            JsValue::from_str(&to_json(&names)?),
        ])?
        .all()
        .await?
        .results()?;
    Ok(rows
        .into_iter()
        .map(|row| FileNameRow {
            id: row.id,
            name: row.name,
        })
        .collect())
}

/// The folders the owner has a share row for, live or stopped.
pub(crate) async fn list_folder_share_prefixes(
    db: &D1Database,
    owner_user_id: &str,
) -> Result<Vec<String>> {
    #[derive(Deserialize)]
    struct PrefixRow {
        folder_prefix: String,
    }
    let rows: Vec<PrefixRow> = db
        .prepare(LIST_FOLDER_SHARE_PREFIXES)
        .bind(&[JsValue::from_str(owner_user_id)])?
        .all()
        .await?
        .results()?;
    Ok(rows.into_iter().map(|row| row.folder_prefix).collect())
}

/// The owner's active files lying under one of `prefixes`.
pub(crate) async fn active_files_under_prefixes(
    db: &D1Database,
    owner_user_id: &str,
    prefixes: &[String],
) -> Result<Vec<ActiveFileName>> {
    let rows: Vec<FileNameRowRaw> = db
        .prepare(LIST_ACTIVE_FILES_UNDER_PREFIXES)
        .bind(&[
            JsValue::from_str(owner_user_id),
            JsValue::from_str(&to_json(&prefixes)?),
        ])?
        .all()
        .await?
        .results()?;
    Ok(rows
        .into_iter()
        .map(|row| ActiveFileName {
            id: row.id,
            name: row.name,
        })
        .collect())
}

/// Renames every file of `moves` and re-points the folder shares of
/// `rewrites` in one D1 batch (a single transaction): all of it or, on a
/// `idx_files_owner_name` or `idx_shares_owner_folder_prefix` unique-constraint
/// violation (propagated as-is, like `rename_file`), none. The caller has
/// checked that every file exists.
pub(crate) async fn rename_files_batch(
    db: &D1Database,
    owner_user_id: &str,
    moves: &[RenameBatchMove],
    rewrites: &[PrefixRewrite],
    now: i64,
) -> Result<()> {
    let moves_json = to_json(
        &moves
            .iter()
            .map(|rename| serde_json::json!({ "id": rename.id, "name": rename.name }))
            .collect::<Vec<_>>(),
    )?;
    let rename = db.prepare(RENAME_FILES_BATCH).bind(&[
        JsValue::from_str(&moves_json),
        JsValue::from_f64(now as f64),
        JsValue::from_str(owner_user_id),
    ])?;
    let rewrite_statements = rewrites
        .iter()
        .map(|rewrite| {
            db.prepare(REWRITE_FOLDER_SHARE_PREFIX).bind(&[
                JsValue::from_str(&rewrite.to),
                JsValue::from_str(owner_user_id),
                JsValue::from_str(&rewrite.from),
            ])
        })
        .collect::<Result<Vec<_>>>()?;
    db.batch(std::iter::once(rename).chain(rewrite_statements).collect())
        .await?;
    Ok(())
}

/// Atomic delete (move to the bin). `false` -> 404: no such active file for
/// this caller (already trashed, wrong owner, or never existed).
pub(crate) async fn trash_file(
    db: &D1Database,
    id: &str,
    owner_user_id: &str,
    now: i64,
) -> Result<bool> {
    let result = db
        .prepare(TRASH_FILE)
        .bind(&[
            JsValue::from_f64(now as f64),
            JsValue::from_str(id),
            JsValue::from_str(owner_user_id),
        ])?
        .run()
        .await?;
    Ok(rows_changed(&result)? == ContentWriteAttempt::Applied)
}

/// Atomic restore. `false` -> 404: no such trashed file for this caller.
/// Can also propagate a `idx_files_owner_name` unique-constraint violation
/// on a name-collision race, same as `insert_file`/`rename_file`.
pub(crate) async fn restore_file(
    db: &D1Database,
    id: &str,
    owner_user_id: &str,
    name: &str,
    now: i64,
) -> Result<bool> {
    let result = db
        .prepare(RESTORE_FILE)
        .bind(&[
            JsValue::from_str(name),
            JsValue::from_f64(now as f64),
            JsValue::from_str(id),
            JsValue::from_str(owner_user_id),
        ])?
        .run()
        .await?;
    Ok(rows_changed(&result)? == ContentWriteAttempt::Applied)
}

/// Reads a `D1Result`'s `meta().changes` to classify a write as
/// `Applied`/`RowsAffectedZero`. `changes` being absent from the metadata
/// (rather than `Some(0)`) is treated the same as zero -- this worker never
/// runs a write whose success is ambiguous enough to need a third state.
/// Synchronous -- `D1Result::meta()` itself isn't async.
fn rows_changed(result: &worker::D1Result) -> Result<ContentWriteAttempt> {
    let changes = result.meta()?.and_then(|meta| meta.changes).unwrap_or(0);
    Ok(if changes > 0 {
        ContentWriteAttempt::Applied
    } else {
        ContentWriteAttempt::RowsAffectedZero
    })
}
