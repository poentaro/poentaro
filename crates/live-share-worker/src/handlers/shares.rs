//! Synced Share route handlers -- see the parent module's doc comment.
//!
//! A share points at a cloud `files` row (see `crate::share`), so there is
//! no share-side write of content at all: the owner's ordinary autosave is
//! what a viewer sees. These routes only start, stop and report on the
//! pointer itself.

use worker::D1Database;

use crate::api_error::ApiError;
use crate::db;
use crate::folder_share;
use crate::model::SyncedDoc;
use crate::share;
use crate::share_id;

use super::now_millis;
use super::HandlerResult;

/// What the link `share_id` shows: a file, a folder, or nothing for an
/// ended link or an unknown id.
pub(super) async fn read_share(d1: &D1Database, share_id: &str) -> HandlerResult<SyncedDoc> {
    if let Some(view) = db::get_share_view(d1, share_id).await? {
        return Ok(share::to_public_doc(Some(&view)));
    }
    match db::get_folder_share_header(d1, share_id).await? {
        Some(header) => read_folder_share(d1, &header).await,
        None => Ok(share::to_public_doc(None)),
    }
}

/// The viewer's tree of a folder share: nothing is read for a stopped share;
/// a live one is refused (`SharedFolderTooLarge`) rather than truncated when
/// the folder has grown past the limits.
async fn read_folder_share(
    d1: &D1Database,
    header: &folder_share::FolderShareHeader,
) -> HandlerResult<SyncedDoc> {
    if header.ended_at.is_some() {
        return Ok(folder_share::to_public_folder_doc(header, &[]));
    }
    let stats =
        db::get_folder_files_stats(d1, &header.owner_user_id, &header.folder_prefix).await?;
    folder_share::check_size(stats)?;
    let rows = db::list_folder_files(d1, &header.owner_user_id, &header.folder_prefix).await?;
    Ok(folder_share::to_public_folder_doc(header, &rows))
}

/// The link of the owner's file `file_id`, started or resumed. `NotFound`
/// when the owner has no such file outside the trash.
pub(super) async fn start_file_link(
    d1: &D1Database,
    owner_user_id: &str,
    file_id: &str,
) -> HandlerResult<String> {
    let is_active_owned_file = db::get_file_by_id(d1, file_id, owner_user_id)
        .await?
        .is_some_and(|file| file.trashed_at.is_none());
    if !is_active_owned_file {
        return Err(ApiError::NotFound);
    }
    let new_share_id = share_id::generate_unique_share_id(d1).await?;
    db::upsert_share(d1, &new_share_id, owner_user_id, file_id, now_millis())
        .await?
        .ok_or_else(|| ApiError::Internal {
            message: "upserting the share returned no row".to_string(),
        })
}

/// Ends the link of the owner's file `file_id`. `NotFound` when it was
/// never shared.
pub(super) async fn stop_file_link(
    d1: &D1Database,
    owner_user_id: &str,
    file_id: &str,
) -> HandlerResult<()> {
    if db::end_share(d1, file_id, owner_user_id, now_millis()).await? {
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

/// The link of the owner's folder `folder` (already validated), started or
/// resumed. `NotFound` when no file is in it; `SharedFolderTooLarge` past the
/// size limits.
pub(super) async fn start_folder_link(
    d1: &D1Database,
    owner_user_id: &str,
    folder: &str,
) -> HandlerResult<String> {
    let stats = db::get_folder_files_stats(d1, owner_user_id, folder).await?;
    if stats.file_count == 0 {
        return Err(ApiError::NotFound);
    }
    folder_share::check_size(stats)?;
    let new_share_id = share_id::generate_unique_share_id(d1).await?;
    db::upsert_folder_share(d1, &new_share_id, owner_user_id, folder, now_millis())
        .await?
        .ok_or_else(|| ApiError::Internal {
            message: "upserting the folder share returned no row".to_string(),
        })
}

/// Ends the link of the owner's folder `folder`. `NotFound` when it was
/// never shared.
pub(super) async fn stop_folder_link(
    d1: &D1Database,
    owner_user_id: &str,
    folder: &str,
) -> HandlerResult<()> {
    if db::end_folder_share(d1, owner_user_id, folder, now_millis()).await? {
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}
