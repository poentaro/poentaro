//! Wire types for the Synced Share worker's HTTP API. These are the single
//! source of truth: `ToSchema`/`IntoParams` put them in the OpenAPI spec
//! (`crate::openapi_json`) that `web`'s `build:worker-types` script turns
//! into `web/src/generated/live-share-worker/schema.ts` and its typed
//! client -- never hand-mirror them in TS.
//!
//! JSON field names use `camelCase` on the wire (matching the old TS
//! convention, e.g. `identityToken`), even though Rust field names stay
//! `snake_case`.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

mod origin_migration;
pub use origin_migration::*;

/// What `GET /shares/{share_id}` returns. `filename`/`content`/`revision`
/// are the pointed-at `files` row's current `name`/`content`/`revision` --
/// a share holds no copy of its own (see `crate::share`). `ended` is true
/// when the owner pressed "Stop Sync" or the file is in the bin; the
/// `shares` row keeps existing either way so a later start reproduces the
/// same link, and while ended `filename`/`content` are empty so a stopped
/// link can't leak the live file. Never carries `owner_user_id`,
/// `share_id`, the file id, any token/hash, or any other internal id -- see
/// `crate::share::to_public_doc`. Nor does it say who the owner is: an
/// owner's only name is their email, which a link must not leak.
///
/// A FOLDER share also answers with this shape: `folder` is then set to the
/// whole tree, and `filename` is the folder's name, `content` empty and
/// `revision` 0, so a client that predates folder shares sees an empty file
/// rather than an error. `folder` is absent (not `null`) for a file share,
/// an ended share and an unknown id, so those responses are byte-for-byte
/// what they were before folder shares existed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SyncedDoc {
    pub ended: bool,
    pub filename: String,
    pub content: String,
    pub revision: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<SharedFolder>,
}

/// The tree a folder share shows its viewers (read-only). Paths are relative
/// to the shared folder; the owner's ancestor folders are never exposed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SharedFolder {
    /// The shared folder's own name (last segment of its path).
    pub name: String,
    /// Every active file, in path order, `.keep` placeholders excluded.
    pub files: Vec<SharedFolderFile>,
    /// Relative folders that exist only as an empty folder (a `.keep`
    /// placeholder, no file under them). Folders implied by `files` paths
    /// are not repeated here.
    pub empty_folders: Vec<String>,
}

/// One file of a shared folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SharedFolderFile {
    /// Path relative to the shared folder, e.g. `Advent/O Come.poentaro`.
    pub path: String,
    pub content: String,
    pub revision: i64,
}

/// Body of `POST /files/{id}/share`, `POST /files/{id}/share/stop` and `POST
/// /files/{id}/share/status` -- all three only need the caller's identity;
/// the file is named by the path, and ownership is checked server-side
/// against the resolved identity (never the client's own claim).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileShareRequest {
    pub identity_token: String,
}

/// Response of `POST /files/{id}/share` -- the file's one share id, the same
/// one every time the file is shared (`shares.file_id` is unique).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileShareResponse {
    pub share_id: String,
}

/// Response of `POST /files/{id}/share/status` -- `share` is `None` when the
/// caller has never shared this file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShareStatusResponse {
    pub share: Option<ShareStatus>,
}

/// A file's share as its owner sees it: the link's id and whether it's been
/// stopped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShareStatus {
    pub share_id: String,
    pub ended: bool,
}

/// Body of `POST /folders/share`, `/folders/share/stop` and
/// `/folders/share/status`. `folder` is the folder's path without a trailing
/// slash (`Hymns/Advent`), validated with the shared `TreePath` rules
/// (`ApiError::InvalidName` otherwise).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FolderShareRequest {
    pub identity_token: String,
    pub folder: String,
}

/// Body of `POST /shares/overview`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShareOverviewRequest {
    pub identity_token: String,
}

/// A live synced link on one file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileShareEntry {
    pub share_id: String,
    pub file_id: String,
    /// The file's current path.
    pub name: String,
}

/// A live synced link on one folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FolderShareEntry {
    pub share_id: String,
    /// The folder's current path, no trailing slash.
    pub folder: String,
}

/// Response of `POST /shares/overview`: every LIVE synced link the caller
/// has (stopped links and links of binned files are left out). One call
/// feeds all the share markers: a folder or file has its own link when it
/// appears here, a file or folder is "shared via" an ancestor when a
/// `folders` entry is a proper path-segment prefix of it, and a folder's
/// files that have a link of their own are the `files` entries under it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShareOverviewResponse {
    pub files: Vec<FileShareEntry>,
    pub folders: Vec<FolderShareEntry>,
}

// -- `/files/*` (cloud storage backend) -------------------------------------
//
// Unlike `SyncedDoc` above (a wire shape distinct from the pure
// `crate::share::ShareView`), the public wire shape for a file is
// `crate::files::PublicFile` itself -- it already derives `Serialize` +
// `Deserialize` with `camelCase` renaming, so these request/response types
// reuse it directly instead of duplicating an identical `PublicFileWire`
// struct here.
//
// Every `/files/*` route requires a resolved identity (see
// `crate::handlers`'s module doc comment) -- `identity_token` appears in
// every request body below, same convention as `FileShareRequest` above.

/// Body of `POST /files/list`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListFilesRequest {
    pub identity_token: String,
}

/// Response of `POST /files/list` -- every file (active and trashed alike)
/// owned by the resolved caller; the client partitions this by
/// `trashedAt` into its file list vs. bin (see `cloudBackend.ts`'s
/// `load()`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListFilesResponse {
    pub files: Vec<crate::files::PublicFile>,
}

/// Body of `POST /files` -- creates a new file row (new file / duplicate /
/// import). `id` is client-generated (`fileStore.ts`'s
/// `generateFileId()`) -- safe because every write is additionally gated on
/// the resolved `owner_user_id` server-side. Response is the created
/// `crate::files::PublicFile` itself; a name collision against the
/// caller's other files (active or trashed) is instead reported as
/// `crate::api_error::ApiError::NameTaken`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileRequest {
    pub identity_token: String,
    pub id: String,
    pub name: String,
    pub content: String,
}

/// Body of `POST /files/{id}/content` -- the atomic CAS content save. See
/// `crate::files::classify_content_write` for how `expected_revision` gates
/// the write.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFileContentRequest {
    pub identity_token: String,
    pub content: String,
    pub expected_revision: i64,
}

/// `200` response of `POST /files/{id}/content`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFileContentResponse {
    pub revision: i64,
}

/// Body of `POST /files/{id}/versions`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListFileVersionsRequest {
    pub identity_token: String,
}

/// One earlier version of a file, as the History panel lists it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileVersionSummary {
    pub revision: i64,
    /// When this version was saved (epoch millis).
    pub saved_at: i64,
}

/// `200` response of `POST /files/{id}/versions`: the file's earlier
/// versions, newest first. The current text is not one of them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListFileVersionsResponse {
    pub versions: Vec<FileVersionSummary>,
}

/// Body of `POST /files/{id}/versions/get`. `ApiError::NotFound` when the
/// file is not the caller's or `revision` is not in its history.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetFileVersionRequest {
    pub identity_token: String,
    pub revision: i64,
}

/// `200` response of `POST /files/{id}/versions/get`: the version's text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileVersionResponse {
    pub content: String,
}

/// Body of `POST /files/{id}/rename`. No revision gate -- rename has no
/// conflict semantics (matches the old GitHub backend's actual behavior,
/// decision #3). `ApiError::NotFound` on zero rows affected; a name
/// collision maps to `ApiError::NameTaken`, same as `CreateFileRequest`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RenameFileRequest {
    pub identity_token: String,
    pub name: String,
}

/// One file's new name in a `RenameBatchRequest`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RenameBatchMove {
    pub id: String,
    pub name: String,
}

/// Body of `POST /files/rename-batch` -- renames every listed file, or none
/// of them (a folder move). No revision gate, like `RenameFileRequest`.
/// `ApiError::NotFound` if any file is not the caller's; a destination held
/// by another file maps to `ApiError::NamesTaken` listing those
/// destinations, with nothing applied. Repeating a batch that was already
/// applied (its answer was lost) succeeds.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RenameBatchRequest {
    pub identity_token: String,
    pub moves: Vec<RenameBatchMove>,
}

/// Body of `POST /files/{id}/delete` -- moves the file to the bin
/// (`trashed_at`). No revision gate; `ApiError::NotFound` on zero rows
/// affected.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFileRequest {
    pub identity_token: String,
}

/// Body of `POST /files/{id}/restore`. `name` is supplied by the caller
/// (already recomputed client-side to be unique, per `fileStore.ts`'s
/// `uniqueName`/`reservedNames`) -- can still race into a `ApiError::NameTaken`,
/// same as `CreateFileRequest`/`RenameFileRequest`. `ApiError::NotFound` on
/// zero rows affected (no such trashed file for this caller).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RestoreFileRequest {
    pub identity_token: String,
    pub name: String,
}

/// Path parameters of every `/files/{id}/...` route.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct FileIdPath {
    pub id: String,
}

/// Path parameters of `GET /shares/{share_id}`. `share_id`'s schema
/// carries the `ShareIdFormat` constraints (`crate::share_id`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct ShareIdPath {
    #[param(schema_with = crate::share_id::share_id_schema)]
    pub share_id: String,
}

/// Body of `POST /auth/sign-up` and `POST /auth/sign-in`. `email` is
/// trimmed and lowercased before use (`auth::email::normalize_email`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CredentialsRequest {
    pub email: String,
    pub password: String,
}

/// Response of `POST /auth/sign-up` and `POST /auth/sign-in`: a new session.
/// The web app sends `session_token` as every other route's
/// `identity_token`, and shows `email` (normalized) as the account's name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    pub session_token: String,
    pub email: String,
}

/// Body of `POST /auth/sign-out`: the session to end.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignOutRequest {
    pub identity_token: String,
}

/// Body of `POST /auth/change-password`. The caller's own session survives;
/// every other session of the account is ended.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    pub identity_token: String,
    pub current_password: String,
    pub new_password: String,
}

/// Body of the test-only `POST /e2e/reset` (see `crate::e2e_reset`).
/// `emails` narrows the reset to some of the accounts the worker's own env
/// allows (every one of them must be in that list); absent, it resets them
/// all.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct E2eResetRequest {
    #[serde(default)]
    pub emails: Option<Vec<String>>,
}

/// Response of `POST /e2e/reset` -- how many rows it deleted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct E2eResetResponse {
    pub deleted_accounts: usize,
    pub deleted_shares: usize,
    pub deleted_files: usize,
}
