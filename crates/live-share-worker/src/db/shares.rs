//! Queries against the `shares` table (Synced Share).

use serde::Deserialize;
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, Result};

use crate::folder_share::{FolderFileRow, FolderShareHeader, FolderStats};
use crate::model::{FileShareEntry, FolderShareEntry, ShareStatus};
use crate::share::ShareView;

const GET_SHARE_VIEW: &str = include_str!("../../queries/get_share_view.sql");
const UPSERT_SHARE: &str = include_str!("../../queries/upsert_share.sql");
const END_SHARE: &str = include_str!("../../queries/end_share.sql");
const GET_SHARE_STATUS_BY_FILE: &str = include_str!("../../queries/get_share_status_by_file.sql");
const UPSERT_FOLDER_SHARE: &str = include_str!("../../queries/upsert_folder_share.sql");
const END_FOLDER_SHARE: &str = include_str!("../../queries/end_folder_share.sql");
const GET_FOLDER_SHARE_STATUS: &str = include_str!("../../queries/get_folder_share_status.sql");
const GET_FOLDER_SHARE_HEADER: &str = include_str!("../../queries/get_folder_share_header.sql");
const GET_FOLDER_FILES_STATS: &str = include_str!("../../queries/get_folder_files_stats.sql");
const LIST_FOLDER_FILES: &str = include_str!("../../queries/list_folder_files.sql");
const LIST_LIVE_FILE_SHARES_BY_OWNER: &str =
    include_str!("../../queries/list_live_file_shares_by_owner.sql");
const LIST_LIVE_FOLDER_SHARES_BY_OWNER: &str =
    include_str!("../../queries/list_live_folder_shares_by_owner.sql");
const SHARE_ID_EXISTS: &str = include_str!("../../queries/share_id_exists.sql");

/// Raw row shape from `get_share_view.sql` -- kept separate from
/// `ShareView` so `crate::share` (the pure, D1-free module) never has to
/// derive `serde::Deserialize` for D1's sake, same as `StoredFileRow`.
#[derive(Deserialize)]
struct ShareViewRow {
    ended_at: Option<i64>,
    trashed_at: Option<i64>,
    filename: String,
    content: String,
    revision: i64,
}

impl From<ShareViewRow> for ShareView {
    fn from(row: ShareViewRow) -> Self {
        ShareView {
            ended_at: row.ended_at,
            trashed_at: row.trashed_at,
            filename: row.filename,
            content: row.content,
            revision: row.revision,
        }
    }
}

/// Raw row shape from `get_share_status_by_file.sql`.
#[derive(Deserialize)]
struct ShareStatusRow {
    share_id: String,
    ended_at: Option<i64>,
}

/// The anonymous viewer read -- `None` when no share has this id.
pub(crate) async fn get_share_view(db: &D1Database, share_id: &str) -> Result<Option<ShareView>> {
    let row: Option<ShareViewRow> = db
        .prepare(GET_SHARE_VIEW)
        .bind(&[JsValue::from_str(share_id)])?
        .first(None)
        .await?;
    Ok(row.map(ShareView::from))
}

/// Starts or resumes sharing `file_id`, returning the file's share id --
/// the existing one if it was ever shared before, else `new_share_id`. The
/// caller must have already checked that `owner_user_id` owns `file_id`.
pub(crate) async fn upsert_share(
    db: &D1Database,
    new_share_id: &str,
    owner_user_id: &str,
    file_id: &str,
    now: i64,
) -> Result<Option<String>> {
    db.prepare(UPSERT_SHARE)
        .bind(&[
            JsValue::from_str(new_share_id),
            JsValue::from_str(owner_user_id),
            JsValue::from_str(file_id),
            JsValue::from_f64(now as f64),
        ])?
        .first(Some("share_id"))
        .await
}

/// Marks `file_id`'s share ended. `false` -> 404: no share for a file this
/// caller owns.
pub(crate) async fn end_share(
    db: &D1Database,
    file_id: &str,
    owner_user_id: &str,
    now: i64,
) -> Result<bool> {
    let result = db
        .prepare(END_SHARE)
        .bind(&[
            JsValue::from_f64(now as f64),
            JsValue::from_str(file_id),
            JsValue::from_str(owner_user_id),
        ])?
        .run()
        .await?;
    let changes = result.meta()?.and_then(|meta| meta.changes).unwrap_or(0);
    Ok(changes > 0)
}

/// The owner's view of `file_id`'s share -- `None` when this caller has
/// never shared it (or doesn't own it).
pub(crate) async fn get_share_status_by_file(
    db: &D1Database,
    file_id: &str,
    owner_user_id: &str,
) -> Result<Option<ShareStatus>> {
    let row: Option<ShareStatusRow> = db
        .prepare(GET_SHARE_STATUS_BY_FILE)
        .bind(&[JsValue::from_str(file_id), JsValue::from_str(owner_user_id)])?
        .first(None)
        .await?;
    Ok(row.map(|row| ShareStatus {
        share_id: row.share_id,
        ended: row.ended_at.is_some(),
    }))
}

pub(crate) async fn share_id_exists(db: &D1Database, share_id: &str) -> Result<bool> {
    let present: Option<i64> = db
        .prepare(SHARE_ID_EXISTS)
        .bind(&[JsValue::from_str(share_id)])?
        .first(Some("present"))
        .await?;
    Ok(present.is_some())
}

/// Raw row shape from `get_folder_share_header.sql`.
#[derive(Deserialize)]
struct FolderShareHeaderRow {
    ended_at: Option<i64>,
    owner_user_id: String,
    folder_prefix: String,
}

#[derive(Deserialize)]
struct FolderStatsRow {
    file_count: i64,
    content_bytes: i64,
}

#[derive(Deserialize)]
struct FolderFileRowRaw {
    name: String,
    content: String,
    revision: i64,
}

#[derive(Deserialize)]
struct FileShareEntryRow {
    share_id: String,
    file_id: String,
    name: String,
}

#[derive(Deserialize)]
struct FolderShareEntryRow {
    share_id: String,
    folder_prefix: String,
}

/// Starts or resumes sharing the owner's folder `folder`; returns its share
/// id (the existing one if the folder was ever shared, else `new_share_id`).
pub(crate) async fn upsert_folder_share(
    db: &D1Database,
    new_share_id: &str,
    owner_user_id: &str,
    folder: &str,
    now: i64,
) -> Result<Option<String>> {
    db.prepare(UPSERT_FOLDER_SHARE)
        .bind(&[
            JsValue::from_str(new_share_id),
            JsValue::from_str(owner_user_id),
            JsValue::from_str(folder),
            JsValue::from_f64(now as f64),
        ])?
        .first(Some("share_id"))
        .await
}

/// Marks the owner's folder share ended. `false` -> 404: never shared.
pub(crate) async fn end_folder_share(
    db: &D1Database,
    owner_user_id: &str,
    folder: &str,
    now: i64,
) -> Result<bool> {
    let result = db
        .prepare(END_FOLDER_SHARE)
        .bind(&[
            JsValue::from_f64(now as f64),
            JsValue::from_str(owner_user_id),
            JsValue::from_str(folder),
        ])?
        .run()
        .await?;
    let changes = result.meta()?.and_then(|meta| meta.changes).unwrap_or(0);
    Ok(changes > 0)
}

pub(crate) async fn get_folder_share_status(
    db: &D1Database,
    owner_user_id: &str,
    folder: &str,
) -> Result<Option<ShareStatus>> {
    let row: Option<ShareStatusRow> = db
        .prepare(GET_FOLDER_SHARE_STATUS)
        .bind(&[JsValue::from_str(owner_user_id), JsValue::from_str(folder)])?
        .first(None)
        .await?;
    Ok(row.map(|row| ShareStatus {
        share_id: row.share_id,
        ended: row.ended_at.is_some(),
    }))
}

/// The anonymous viewer read of a folder share -- `None` when the id is not
/// a folder share.
pub(crate) async fn get_folder_share_header(
    db: &D1Database,
    share_id: &str,
) -> Result<Option<FolderShareHeader>> {
    let row: Option<FolderShareHeaderRow> = db
        .prepare(GET_FOLDER_SHARE_HEADER)
        .bind(&[JsValue::from_str(share_id)])?
        .first(None)
        .await?;
    Ok(row.map(|row| FolderShareHeader {
        ended_at: row.ended_at,
        owner_user_id: row.owner_user_id,
        folder_prefix: row.folder_prefix,
    }))
}

/// How many active files (placeholders included) and content bytes lie
/// under the owner's `folder`.
pub(crate) async fn get_folder_files_stats(
    db: &D1Database,
    owner_user_id: &str,
    folder: &str,
) -> Result<FolderStats> {
    let row: Option<FolderStatsRow> = db
        .prepare(GET_FOLDER_FILES_STATS)
        .bind(&[JsValue::from_str(owner_user_id), JsValue::from_str(folder)])?
        .first(None)
        .await?;
    Ok(row.map_or(
        FolderStats {
            file_count: 0,
            content_bytes: 0,
        },
        |row| FolderStats {
            file_count: row.file_count,
            content_bytes: row.content_bytes,
        },
    ))
}

/// Every active file under the owner's `folder`, in path order.
pub(crate) async fn list_folder_files(
    db: &D1Database,
    owner_user_id: &str,
    folder: &str,
) -> Result<Vec<FolderFileRow>> {
    let rows: Vec<FolderFileRowRaw> = db
        .prepare(LIST_FOLDER_FILES)
        .bind(&[JsValue::from_str(owner_user_id), JsValue::from_str(folder)])?
        .all()
        .await?
        .results()?;
    Ok(rows
        .into_iter()
        .map(|row| FolderFileRow {
            name: row.name,
            content: row.content,
            revision: row.revision,
        })
        .collect())
}

/// The owner's live links on files (not binned).
pub(crate) async fn list_live_file_shares(
    db: &D1Database,
    owner_user_id: &str,
) -> Result<Vec<FileShareEntry>> {
    let rows: Vec<FileShareEntryRow> = db
        .prepare(LIST_LIVE_FILE_SHARES_BY_OWNER)
        .bind(&[JsValue::from_str(owner_user_id)])?
        .all()
        .await?
        .results()?;
    Ok(rows
        .into_iter()
        .map(|row| FileShareEntry {
            share_id: row.share_id,
            file_id: row.file_id,
            name: row.name,
        })
        .collect())
}

/// The owner's live links on folders.
pub(crate) async fn list_live_folder_shares(
    db: &D1Database,
    owner_user_id: &str,
) -> Result<Vec<FolderShareEntry>> {
    let rows: Vec<FolderShareEntryRow> = db
        .prepare(LIST_LIVE_FOLDER_SHARES_BY_OWNER)
        .bind(&[JsValue::from_str(owner_user_id)])?
        .all()
        .await?
        .results()?;
    Ok(rows
        .into_iter()
        .map(|row| FolderShareEntry {
            share_id: row.share_id,
            folder: row.folder_prefix,
        })
        .collect())
}
