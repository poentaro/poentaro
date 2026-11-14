//! Synced Share route handlers -- see the parent module's doc comment.
//!
//! A share points at a cloud `files` row (see `crate::share`), so there is
//! no share-side write of content at all: the owner's ordinary autosave is
//! what a viewer sees. These routes only start, stop and report on the
//! pointer itself.

use worker::{D1Database, RouteContext};

use crate::api_error::ApiError;
use crate::app_log::RequestMeta;
use crate::db;
use crate::folder_share;
use crate::protocol::{
    FileIdPath, FileShareRequest, FileShareResponse, FolderShareRequest, ShareIdPath,
    ShareOverviewRequest, ShareOverviewResponse, ShareStatusResponse, SyncedDoc,
};
use crate::share;
use crate::share_id;

use super::routes::{HandlerResult, Json, NoContent, NoPathParams};
use super::{now_millis, resolve_files_caller, D1_BINDING};

/// `GET /shares/{share_id}` -- fully anonymous: the unguessable `share_id`
/// is the sole credential needed to view (TODO §0). Reads the pointed-at
/// file directly; an ended share returns no content (see
/// `share::to_public_doc`). Never cached -- see `routes::respond`.
pub(super) async fn get_share(
    ctx: RouteContext<()>,
    path: ShareIdPath,
) -> HandlerResult<Json<SyncedDoc>> {
    let d1 = ctx.d1(D1_BINDING)?;
    read_share(&d1, &path.share_id).await.map(Json)
}

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

/// `POST /files/{id}/share` -- starts (or resumes) sharing one of the
/// caller's active files. Idempotent: a file that was ever shared keeps its
/// existing `share_id` (`queries/upsert_share.sql`'s `ON CONFLICT
/// (file_id)`), so re-sharing always reproduces the same link, from any
/// device.
pub(super) async fn start_share(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: FileShareRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<FileShareResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;
    let share_id = start_file_link(&d1, &owner_user_id, &path.id).await?;
    Ok(Json(FileShareResponse { share_id }))
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

/// `POST /files/{id}/share/stop` -- ends the caller's share of this file.
/// The row is kept so a later start reproduces the same link. `NoContent`,
/// or `NotFound` when the caller has no share for this file.
pub(super) async fn stop_share(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: FileShareRequest,
    request_meta: RequestMeta,
) -> HandlerResult<NoContent> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;

    stop_file_link(&d1, &owner_user_id, &path.id).await?;
    Ok(NoContent)
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

/// `POST /files/{id}/share/status` -- whether the caller's file is shared,
/// and under which link. The server is the single source of truth, so every
/// device the owner signs in on sees the same live/stopped state.
pub(super) async fn share_status(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: FileShareRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<ShareStatusResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;

    let share = db::get_share_status_by_file(&d1, &path.id, &owner_user_id).await?;
    Ok(Json(ShareStatusResponse { share }))
}

/// `POST /folders/share` -- starts (or resumes) sharing one of the caller's
/// folders, named by path. The folder must hold at least one active file
/// (an empty folder is kept alive by its `.keep` placeholder, which counts)
/// and stay within the size limits; otherwise `NotFound` /
/// `SharedFolderTooLarge`. Idempotent like a file share: the folder keeps its
/// `share_id` across stop and start. Sharing does not look at ancestors or
/// descendants: a file or subfolder with its own link keeps it (the client
/// warns from `POST /shares/overview`).
pub(super) async fn start_folder_share(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: FolderShareRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<FileShareResponse>> {
    let folder = folder_share::validate_folder_path(&body.folder)?;
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;

    let share_id = start_folder_link(&d1, &owner_user_id, folder.as_str()).await?;
    Ok(Json(FileShareResponse { share_id }))
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

/// `POST /folders/share/stop` -- ends the caller's share of this folder
/// (the row stays so a later start reproduces the link). `NotFound` when the
/// caller never shared it; works even if the folder no longer has files.
pub(super) async fn stop_folder_share(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: FolderShareRequest,
    request_meta: RequestMeta,
) -> HandlerResult<NoContent> {
    let folder = folder_share::validate_folder_path(&body.folder)?;
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;

    stop_folder_link(&d1, &owner_user_id, folder.as_str()).await?;
    Ok(NoContent)
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

/// `POST /folders/share/status` -- the caller's link on this folder, if any.
pub(super) async fn folder_share_status(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: FolderShareRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<ShareStatusResponse>> {
    let folder = folder_share::validate_folder_path(&body.folder)?;
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;

    let share = db::get_folder_share_status(&d1, &owner_user_id, folder.as_str()).await?;
    Ok(Json(ShareStatusResponse { share }))
}

/// `POST /shares/overview` -- every live link the caller has, on files and
/// on folders (see `protocol::ShareOverviewResponse`).
pub(super) async fn share_overview(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: ShareOverviewRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<ShareOverviewResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;

    let files = db::list_live_file_shares(&d1, &owner_user_id).await?;
    let folders = db::list_live_folder_shares(&d1, &owner_user_id).await?;
    Ok(Json(ShareOverviewResponse { files, folders }))
}
