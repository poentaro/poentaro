//! `/files/*` route handlers (the cloud storage backend) -- see the parent
//! module's doc comment.

use worker::RouteContext;

use crate::api_error::ApiError;
use crate::app_log::RequestMeta;
use crate::db;
use crate::file_history::{self, VersionRow};
use crate::files::{self, ContentWriteAttempt, ContentWriteOutcome, PublicFile};
use crate::folder_share;
use crate::protocol::{
    CreateFileRequest, DeleteFileRequest, FileIdPath, FileVersionResponse, FileVersionSummary,
    GetFileVersionRequest, ListFileVersionsRequest, ListFileVersionsResponse, ListFilesRequest,
    ListFilesResponse, RenameBatchRequest, RenameFileRequest, RestoreFileRequest,
    UpdateFileContentRequest, UpdateFileContentResponse,
};

use super::routes::{HandlerResult, Json, NoContent, NoPathParams};
use super::{is_unique_constraint_violation, resolve_files_caller, D1_BINDING};

/// Refuses `proposed` (the names a write is about to give, by file id) when
/// the owner's live files, with those renamed, would have a path that is both
/// a file and a folder.
async fn ensure_no_file_folder_conflict(
    d1: &worker::D1Database,
    owner_user_id: &str,
    proposed: &[files::FileNameRow],
) -> Result<(), ApiError> {
    let live = db::list_live_file_names(d1, owner_user_id).await?;
    match files::find_file_folder_conflict(&live, proposed) {
        Some(path) => Err(ApiError::FileFolderConflict { path }),
        None => Ok(()),
    }
}

pub(super) fn proposed_name(id: &str, name: &str) -> Vec<files::FileNameRow> {
    vec![files::FileNameRow {
        id: id.to_string(),
        name: name.to_string(),
    }]
}

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

/// `POST /files` -- creates a new file row (new file / duplicate / import).
/// See `crate::protocol::CreateFileRequest`'s doc comment for the response
/// shapes. Idempotent: repeating a create with the same id, owner and name
/// returns the existing file instead of `NameTaken`.
pub(super) async fn create_file(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: CreateFileRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<PublicFile>> {
    files::validate_file_name(&body.name)?;
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;

    ensure_no_file_folder_conflict(&d1, &owner_user_id, &proposed_name(&body.id, &body.name))
        .await?;
    let now = worker::Date::now().as_millis() as i64;
    let created = files::StoredFile {
        id: body.id,
        owner_user_id,
        name: body.name,
        content: body.content,
        revision: 0,
        trashed_at: None,
        created_at: now,
        updated_at: now,
    };
    match db::insert_file(&d1, &created).await {
        Ok(()) => Ok(Json(files::to_public_file(&created))),
        Err(error) if is_unique_constraint_violation(&error) => {
            let existing = db::get_file_by_id(&d1, &created.id, &created.owner_user_id).await?;
            files::idempotent_create_match(existing.as_ref(), &created.name)
                .map(Json)
                .ok_or(ApiError::NameTaken)
        }
        Err(error) => Err(error.into()),
    }
}

/// `POST /files/{id}/content` -- the atomic CAS content save. `{revision}`
/// on success, `ApiError::RevisionConflict` when the row moved on since the
/// caller last saw it (or was trashed out from under the write),
/// `ApiError::NotFound` when the file doesn't exist or isn't the caller's --
/// see `crate::files::classify_content_write`. An applied save is also
/// recorded in the file's version history (`record_version`).
pub(super) async fn update_file_content(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: UpdateFileContentRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<UpdateFileContentResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;

    let previous = db::get_file_by_id(&d1, &path.id, &owner_user_id).await?;
    let now = worker::Date::now().as_millis() as i64;
    let attempt = db::update_file_content(
        &d1,
        &path.id,
        &owner_user_id,
        &body.content,
        body.expected_revision,
        now,
    )
    .await?;
    let current = match attempt {
        ContentWriteAttempt::Applied => None,
        ContentWriteAttempt::RowsAffectedZero => {
            db::get_file_by_id(&d1, &path.id, &owner_user_id).await?
        }
    };

    match files::classify_content_write(attempt, body.expected_revision, current.as_ref()) {
        ContentWriteOutcome::Applied { new_revision } => {
            // The text at `expected_revision` is the one read above: every
            // content write bumps the revision, so the CAS pins it.
            if let Some(previous) = previous.filter(|file| file.revision == body.expected_revision)
            {
                if let Err(error) =
                    record_version(&d1, &previous, &body.content, new_revision).await
                {
                    worker::console_error!("file_versions: failed to record: {error}");
                }
            }
            Ok(Json(UpdateFileContentResponse {
                revision: new_revision,
            }))
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
pub(super) async fn record_version(
    d1: &worker::D1Database,
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

/// `POST /files/{id}/versions` -- the caller's file's earlier versions,
/// newest first (`file_history::reachable_versions`).
pub(super) async fn list_file_versions(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: ListFileVersionsRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<ListFileVersionsResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;

    let file = db::get_file_by_id(&d1, &path.id, &owner_user_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let rows = db::list_file_versions(&d1, &path.id, &owner_user_id).await?;
    let versions = file_history::reachable_versions(file.revision, &rows)
        .into_iter()
        .map(|row| FileVersionSummary {
            revision: row.revision,
            saved_at: row.saved_at,
        })
        .collect();
    Ok(Json(ListFileVersionsResponse { versions }))
}

/// `POST /files/{id}/versions/get` -- one earlier version rebuilt from the
/// head, with the diff restoring it would apply. Restoring is an ordinary
/// content save of that text, so it becomes a new head and is itself undoable.
pub(super) async fn get_file_version(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: GetFileVersionRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<FileVersionResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;

    let file = db::get_file_by_id(&d1, &path.id, &owner_user_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let rows = db::list_file_versions(&d1, &path.id, &owner_user_id).await?;
    let content = file_history::rebuild(&file.content, file.revision, &rows, body.revision)
        .ok_or(ApiError::NotFound)?
        .map_err(|error| ApiError::Internal {
            message: error.message,
        })?;
    Ok(Json(FileVersionResponse { content }))
}

/// `POST /files/{id}/rename` -- atomic rename, no revision gate (matches the
/// old GitHub backend's actual behavior, decision #3). `NotFound` on zero
/// rows affected; a name collision maps to `ApiError::NameTaken`.
pub(super) async fn rename_file(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: RenameFileRequest,
    request_meta: RequestMeta,
) -> HandlerResult<NoContent> {
    files::validate_file_name(&body.name)?;
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;

    ensure_no_file_folder_conflict(&d1, &owner_user_id, &proposed_name(&path.id, &body.name))
        .await?;
    let now = worker::Date::now().as_millis() as i64;
    match db::rename_file(&d1, &path.id, &owner_user_id, &body.name, now).await {
        Ok(true) => Ok(NoContent),
        Ok(false) => Err(ApiError::NotFound),
        Err(error) if is_unique_constraint_violation(&error) => Err(ApiError::NameTaken),
        Err(error) => Err(error.into()),
    }
}

/// `POST /files/rename-batch` -- renames every listed file or none (a folder
/// move). Every name is validated first; then `files::plan_batch_rename`
/// decides from the current rows: `NotFound` if a file is missing, success
/// without writing when every file already has its name (a retry whose
/// answer was lost), `NamesTaken` listing the destinations held by other
/// files. Otherwise one statement writes every move, so a race into a
/// unique-index violation still applies nothing.
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

    let rows = db::file_names_for_batch(&d1, &owner_user_id, &body.moves).await?;
    match files::plan_batch_rename(&body.moves, &rows) {
        files::BatchRenamePlan::AlreadyApplied => return Ok(NoContent),
        files::BatchRenamePlan::NotFound => return Err(ApiError::NotFound),
        files::BatchRenamePlan::Invalid => return Err(ApiError::BadRequest),
        files::BatchRenamePlan::NamesTaken(names) => return Err(ApiError::NamesTaken { names }),
        files::BatchRenamePlan::Apply => {}
    }
    let proposed: Vec<files::FileNameRow> = body
        .moves
        .iter()
        .map(|rename| files::FileNameRow {
            id: rename.id.clone(),
            name: rename.name.clone(),
        })
        .collect();
    ensure_no_file_folder_conflict(&d1, &owner_user_id, &proposed).await?;
    // A folder with a synced link that this batch moves as a whole takes the
    // link along: its prefix is re-pointed in the same transaction.
    let prefixes = db::list_folder_share_prefixes(&d1, &owner_user_id).await?;
    let rewrites = if prefixes.is_empty() {
        Vec::new()
    } else {
        let current = db::active_files_under_prefixes(&d1, &owner_user_id, &prefixes).await?;
        folder_share::plan_prefix_rewrites(&prefixes, &body.moves, &current)
    };
    let now = worker::Date::now().as_millis() as i64;
    match db::rename_files_batch(&d1, &owner_user_id, &body.moves, &rewrites, now).await {
        Ok(()) => Ok(NoContent),
        Err(error) if is_unique_constraint_violation(&error) => {
            // Lost a race with another writer: report what is taken now.
            let rows = db::file_names_for_batch(&d1, &owner_user_id, &body.moves).await?;
            match files::plan_batch_rename(&body.moves, &rows) {
                files::BatchRenamePlan::NamesTaken(names) => Err(ApiError::NamesTaken { names }),
                _ => Err(ApiError::NameTaken),
            }
        }
        Err(error) => Err(error.into()),
    }
}

/// `POST /files/{id}/delete` -- atomic move to the bin. `NotFound` on zero
/// rows affected (already trashed, wrong owner, or never existed).
pub(super) async fn delete_file(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: DeleteFileRequest,
    request_meta: RequestMeta,
) -> HandlerResult<NoContent> {
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;

    let now = worker::Date::now().as_millis() as i64;
    if db::trash_file(&d1, &path.id, &owner_user_id, now).await? {
        Ok(NoContent)
    } else {
        Err(ApiError::NotFound)
    }
}

/// `POST /files/{id}/restore` -- atomic restore out of the bin. `NotFound`
/// on zero rows affected (no such trashed file for this caller); a name
/// collision (against the caller's other files) maps to
/// `ApiError::NameTaken`, same as `create_file`/`rename_file`. Like them, a name
/// that would make a path both a file and a folder is
/// `ApiError::FileFolderConflict`.
pub(super) async fn restore_file(
    ctx: RouteContext<()>,
    path: FileIdPath,
    body: RestoreFileRequest,
    request_meta: RequestMeta,
) -> HandlerResult<NoContent> {
    files::validate_file_name(&body.name)?;
    let d1 = ctx.d1(D1_BINDING)?;
    let owner_user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;

    ensure_no_file_folder_conflict(&d1, &owner_user_id, &proposed_name(&path.id, &body.name))
        .await?;
    let now = worker::Date::now().as_millis() as i64;
    match db::restore_file(&d1, &path.id, &owner_user_id, &body.name, now).await {
        Ok(true) => Ok(NoContent),
        Ok(false) => Err(ApiError::NotFound),
        Err(error) if is_unique_constraint_violation(&error) => Err(ApiError::NameTaken),
        Err(error) => Err(error.into()),
    }
}
