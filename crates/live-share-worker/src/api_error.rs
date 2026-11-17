//! The one error body every route answers a failure with. A tagged union
//! (`code`) rather than ad-hoc status/body pairs, so the web client's
//! generated types (see `crate::handlers::routes`) let it `switch` on
//! `code` exhaustively instead of re-matching raw HTTP statuses or
//! hand-typed `{code: "..."}` literals. The HTTP status is derived from the
//! variant (`status_code`), never chosen separately at a call site.
//!
//! On the wire every failure is an `ApiErrorBody`: the `ApiError` plus the
//! request's trace ID, which the web app shows and `app_log` is keyed by.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::session::VerificationFailure;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum ApiError {
    /// The request body or a path parameter didn't parse.
    BadRequest,
    /// No such row for the caller (or not the caller's).
    NotFound,
    /// A create/rename/restore collided with another of the caller's file
    /// names (`idx_files_owner_name`).
    NameTaken,
    /// A batch rename was refused as a whole because these destinations are
    /// held by other files (`names`); nothing was applied.
    NamesTaken { names: Vec<String> },
    /// A create/rename/restore/batch rename was refused because `path` would
    /// be both a file and a folder: it equals the folder of another live
    /// file, or lies inside a live file. Nothing was applied.
    FileFolderConflict { path: String },
    /// A create/rename/restore asked for a file name that is not a valid
    /// path (`poentaro::file_tree::TreePath` rules); `message` says why.
    InvalidName { message: String },
    /// A folder share would cover more files or content than
    /// `folder_share::MAX_SHARED_FOLDER_FILES`/`MAX_SHARED_FOLDER_BYTES`
    /// allow: refused when starting the share, and answered to a viewer
    /// whose folder has grown past the limits since.
    SharedFolderTooLarge { message: String },
    /// An origin-migration upload (`POST /origin-migrations`) larger than
    /// `origin_migration::MAX_ORIGIN_MIGRATION_PAYLOAD_BYTES`; `message`
    /// says by how much.
    OriginMigrationTooLarge { message: String },
    /// A content save's `expectedRevision` lost the race -- feeds the
    /// "Overwrite mine"/"Discard mine" UI with the row's actual revision.
    RevisionConflict {
        #[serde(rename = "currentRevision")]
        current_revision: i64,
    },
    /// The caller's `identity_token` names no live session (never existed,
    /// expired, signed out, or ended by a password change). Carries exactly
    /// `VerificationFailure`'s fields, which by construction never include
    /// the token or its hash.
    Unauthorized(VerificationFailure),
    /// Sign-up with an email that already has an account.
    EmailTaken,
    /// Sign-up or sign-in with something that isn't shaped like an email
    /// (`auth::email::normalize_email`).
    InvalidEmail,
    /// Sign-up or change password with a password the rules refuse
    /// (`auth::password::password_weakness`); `message` says why.
    WeakPassword { message: String },
    /// Sign-in with an unknown email or a wrong password (deliberately the
    /// same answer for both), or change password with a wrong current
    /// password.
    IncorrectCredentials,
    /// Too many wrong passwords for this email (`auth::lockout`): refused
    /// without checking the password until `retryAfterSeconds` from now.
    SignInLocked {
        #[serde(rename = "retryAfterSeconds")]
        retry_after_seconds: i64,
    },
    /// Too many sign-in or sign-up requests from this IP address in the
    /// last minute.
    RateLimited,
    /// Anything else (a D1 error, ...).
    Internal { message: String },
}

impl ApiError {
    pub fn status_code(&self) -> u16 {
        match self {
            ApiError::BadRequest
            | ApiError::InvalidName { .. }
            | ApiError::InvalidEmail
            | ApiError::WeakPassword { .. } => 400,
            ApiError::Unauthorized(_) | ApiError::IncorrectCredentials => 401,
            ApiError::NotFound => 404,
            ApiError::EmailTaken
            | ApiError::NameTaken
            | ApiError::NamesTaken { .. }
            | ApiError::FileFolderConflict { .. }
            | ApiError::RevisionConflict { .. } => 409,
            ApiError::SharedFolderTooLarge { .. } | ApiError::OriginMigrationTooLarge { .. } => 413,
            ApiError::SignInLocked { .. } | ApiError::RateLimited => 429,
            ApiError::Internal { .. } => 500,
        }
    }
}

impl ApiError {
    /// The `code` tag it serializes with, e.g. `"revision_conflict"`.
    pub fn code(&self) -> String {
        serde_json::to_value(self)
            .ok()
            .and_then(|value| value.get("code")?.as_str().map(str::to_owned))
            .unwrap_or_default()
    }

    /// The variant's free-text `message`, for the variants that carry one.
    pub fn message(&self) -> Option<&str> {
        match self {
            ApiError::InvalidName { message }
            | ApiError::SharedFolderTooLarge { message }
            | ApiError::WeakPassword { message }
            | ApiError::Internal { message } => Some(message),
            _ => None,
        }
    }
}

/// What a failed request answers with: the `ApiError`, with the request's
/// trace ID next to its `code` (and in the `X-Trace-Id` header). The ID is
/// also the `trace_id` of the request's `app_log` row, so a user can send it
/// and the failure can be looked up directly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ApiErrorBody {
    #[serde(flatten)]
    pub error: ApiError,
    #[serde(rename = "traceId")]
    pub trace_id: String,
}

impl From<worker::Error> for ApiError {
    fn from(error: worker::Error) -> Self {
        ApiError::Internal {
            message: error.to_string(),
        }
    }
}
