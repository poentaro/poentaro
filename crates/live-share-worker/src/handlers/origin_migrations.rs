//! The origin-migration handoff routes -- see `crate::origin_migration`.
//! Both are anonymous: the upload is rate limited per IP, and the claim's
//! only credential is the unguessable id.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use worker::RouteContext;

use crate::api_error::ApiError;
use crate::app_log::RequestMeta;
use crate::crypto;
use crate::db;
use crate::origin_migration::{is_claimable, payload_rejection, ORIGIN_MIGRATION_ID_BYTES};
use crate::protocol::{
    OriginMigrationClaimRequest, OriginMigrationClaimResponse, OriginMigrationUploadRequest,
    OriginMigrationUploadResponse,
};

use super::routes::{HandlerResult, Json, NoPathParams};
use super::{check_rate_limit, now_millis, D1_BINDING};

/// Rate limit binding (`cloudflare.config.ts`), keyed per client IP.
const ORIGIN_MIGRATION_RATE_LIMITER: &str = "ORIGIN_MIGRATION_RATE_LIMITER";

/// `POST /origin-migrations` -- stores a handoff and answers its id.
pub(super) async fn upload(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: OriginMigrationUploadRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<OriginMigrationUploadResponse>> {
    check_rate_limit(&ctx, ORIGIN_MIGRATION_RATE_LIMITER, &request_meta).await?;
    if let Some(message) = payload_rejection(&body.payload) {
        return Err(ApiError::OriginMigrationTooLarge { message });
    }
    let id = URL_SAFE_NO_PAD.encode(crypto::random_bytes(ORIGIN_MIGRATION_ID_BYTES)?);
    let d1 = ctx.d1(D1_BINDING)?;
    db::insert_origin_migration(&d1, &id, &body.payload, now_millis()).await?;
    Ok(Json(OriginMigrationUploadResponse { id }))
}

/// `POST /origin-migrations/claim` -- answers a handoff's payload once and
/// deletes it. An unknown, already claimed or expired id is `not_found`.
pub(super) async fn claim(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: OriginMigrationClaimRequest,
    _request_meta: RequestMeta,
) -> HandlerResult<Json<OriginMigrationClaimResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    match db::claim_origin_migration(&d1, &body.id).await? {
        Some(claimed) if is_claimable(claimed.created_at, now_millis()) => {
            Ok(Json(OriginMigrationClaimResponse {
                payload: claimed.payload,
            }))
        }
        _ => Err(ApiError::NotFound),
    }
}
