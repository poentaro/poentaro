//! The origin-migration handoff routes -- see `crate::origin_migration`.
//! Both are anonymous: the upload is rate limited per IP, and the claim's
//! only credential is the unguessable id.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use worker::{D1Database, Env, RouteContext};

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

/// `POST /origin-migrations`.
pub(super) async fn upload(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: OriginMigrationUploadRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<OriginMigrationUploadResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let id = store(&ctx.env, &d1, &body.payload, &request_meta).await?;
    Ok(Json(OriginMigrationUploadResponse { id }))
}

/// Stores a handoff and answers its id.
pub(super) async fn store(
    env: &Env,
    d1: &D1Database,
    payload: &str,
    request_meta: &RequestMeta,
) -> HandlerResult<String> {
    check_rate_limit(env, ORIGIN_MIGRATION_RATE_LIMITER, request_meta).await?;
    if let Some(message) = payload_rejection(payload) {
        return Err(ApiError::OriginMigrationTooLarge { message });
    }
    let id = URL_SAFE_NO_PAD.encode(crypto::random_bytes(ORIGIN_MIGRATION_ID_BYTES)?);
    db::insert_origin_migration(d1, &id, payload, now_millis()).await?;
    Ok(id)
}

/// `POST /origin-migrations/claim`.
pub(super) async fn claim(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: OriginMigrationClaimRequest,
    _request_meta: RequestMeta,
) -> HandlerResult<Json<OriginMigrationClaimResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let payload = take(&d1, &body.id).await?;
    Ok(Json(OriginMigrationClaimResponse { payload }))
}

/// Answers a handoff's payload once and deletes it. An unknown, already
/// claimed or expired id is `NotFound`.
pub(super) async fn take(d1: &D1Database, id: &str) -> HandlerResult<String> {
    match db::claim_origin_migration(d1, id).await? {
        Some(claimed) if is_claimable(claimed.created_at, now_millis()) => Ok(claimed.payload),
        _ => Err(ApiError::NotFound),
    }
}
