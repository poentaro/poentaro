//! `/files/*` route handlers (the cloud storage backend) -- see the parent
//! module's doc comment. Each resolves the caller, then runs its core in
//! `file_store`, which `poentaro app` shares.

use worker::RouteContext;

use crate::app_log::RequestMeta;
use crate::db;
use crate::files::{self, PublicFile};
use crate::protocol::{
    CreateFileRequest, DeleteFileRequest, FileIdPath, FileVersionResponse, GetFileVersionRequest,
    ListFileVersionsRequest, ListFileVersionsResponse, ListFilesRequest, ListFilesResponse,
    RenameBatchRequest, RenameFileRequest, RestoreFileRequest, UpdateFileContentRequest,
    UpdateFileContentResponse,
};

use super::file_store;
use super::routes::{HandlerResult, Json, NoContent, NoPathParams};
use super::{resolve_files_caller, D1_BINDING};

/// `POST /files/list` -- every file (active and trashed alike) owned by the
/// resolved caller; the client partitions the result by `trashedAt` (see
/// `crate::protocol::ListFilesResponse`'s doc comment).
pub(super) async fn list_files(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: ListFilesRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<ListFilesResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;

    let stored = db::list_files_by_owner(&d1, &owner_user_id).await?;
    let files = stored.iter().map(files::to_public_file).collect();
    Ok(Json(ListFilesResponse { files }))
}

/// `POST /files` -- `file_store::create`.
pub(super) async fn create_file(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: CreateFileRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<PublicFile>> {
    files::validate_file_name(&body.name)?;
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;
    file_store::create(&d1, owner_user_id, body.id, body.name, body.content)
        .await
        .map(Json)
}

/// `POST /files/{id}/content` -- `file_store::write_content`.
pub(super) async fn update_file_content(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: UpdateFileContentRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<UpdateFileContentResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;
    let revision = file_store::write_content(
        &d1,
        &owner_user_id,
        &path.id,
        body.expected_revision,
        &body.content,
    )
    .await?;
    Ok(Json(UpdateFileContentResponse { revision }))
}

/// `POST /files/{id}/versions` -- `file_store::versions`.
pub(super) async fn list_file_versions(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: ListFileVersionsRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<ListFileVersionsResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;
    let versions = file_store::versions(&d1, &owner_user_id, &path.id).await?;
    Ok(Json(ListFileVersionsResponse { versions }))
}

/// `POST /files/{id}/versions/get` -- one earlier version rebuilt from the
/// head. Restoring is an ordinary content save of that text, so it becomes a
/// new head and is itself undoable.
pub(super) async fn get_file_version(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: GetFileVersionRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<FileVersionResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;
    let content = file_store::version_content(&d1, &owner_user_id, &path.id, body.revision).await?;
    Ok(Json(FileVersionResponse { content }))
}

/// `POST /files/{id}/rename` -- `file_store::rename`.
pub(super) async fn rename_file(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: RenameFileRequest,
    request_meta: RequestMeta,
) -> HandlerResult<NoContent> {
    files::validate_file_name(&body.name)?;
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;
    file_store::rename(&d1, &owner_user_id, &path.id, &body.name).await?;
    Ok(NoContent)
}

/// `POST /files/rename-batch` -- `file_store::rename_batch`.
pub(super) async fn rename_files_batch(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: RenameBatchRequest,
    request_meta: RequestMeta,
) -> HandlerResult<NoContent> {
    body.moves
        .iter()
        .try_for_each(|rename| files::validate_file_name(&rename.name))?;
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;
    file_store::rename_batch(&d1, &owner_user_id, &body.moves).await?;
    Ok(NoContent)
}

/// `POST /files/{id}/delete` -- `file_store::trash`.
pub(super) async fn delete_file(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: DeleteFileRequest,
    request_meta: RequestMeta,
) -> HandlerResult<NoContent> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;
    file_store::trash(&d1, &owner_user_id, &path.id).await?;
    Ok(NoContent)
}

/// `POST /files/{id}/restore` -- `file_store::restore` under the name the
/// client picked.
pub(super) async fn restore_file(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: RestoreFileRequest,
    request_meta: RequestMeta,
) -> HandlerResult<NoContent> {
    files::validate_file_name(&body.name)?;
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;
    file_store::restore(&d1, &owner_user_id, &path.id, &body.name).await?;
    Ok(NoContent)
}
