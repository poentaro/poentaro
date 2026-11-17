//! Request and response bodies of the origin-migration handoff routes (see
//! `crate::origin_migration`), re-exported from `crate::protocol`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Body of `POST /origin-migrations`: everything an old origin's bridge
/// page found in `localStorage`, as one opaque JSON text the worker never
/// parses (see `crate::origin_migration`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OriginMigrationUploadRequest {
    pub payload: String,
}

/// Response of `POST /origin-migrations`: the id the bridge page hands to
/// the new origin in its redirect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OriginMigrationUploadResponse {
    pub id: String,
}

/// Body of `POST /origin-migrations/claim`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OriginMigrationClaimRequest {
    pub id: String,
}

/// Response of `POST /origin-migrations/claim`: the payload exactly as it
/// was uploaded. The handoff is deleted by the claim, so a second claim of
/// the same id answers `not_found`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OriginMigrationClaimResponse {
    pub payload: String,
}
