//! `/api-keys` routes: the account menu creates, lists and revokes the keys
//! an AI agent sends to `POST /agent` (see `crate::api_keys`). Each needs a
//! signed-in session, like `/files`.

use worker::{D1Database, RouteContext};

use super::routes::{HandlerResult, Json, NoContent, NoPathParams};
use super::{now_millis, resolve_files_caller, D1_BINDING};
use crate::api_error::ApiError;
use crate::api_keys::{
    agent_prompt, shown_prefix, ApiKeyIdPath, ApiKeySummary, ApiKeysRequest, CreatedApiKey,
    ListApiKeysResponse, StoredApiKey,
};
use crate::app_log::{AgentEvent, KeyEvent, LogEvent, RequestMeta};
use crate::auth::session::hash_token;
use crate::{crypto, db};

async fn log(db: &D1Database, event: AgentEvent, user_id: &str, request_meta: &RequestMeta) {
    db::record_app_log(db, &LogEvent::Agent(event), Some(user_id), request_meta).await;
}

/// `POST /api-keys/list`.
pub(super) async fn list_api_keys(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: ApiKeysRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<ListApiKeysResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;
    let keys = db::list_api_keys(&d1, &user_id).await?;
    let now = now_millis();
    Ok(Json(ListApiKeysResponse {
        keys: keys
            .into_iter()
            .map(|key| ApiKeySummary::at(key, now))
            .collect(),
    }))
}

/// `POST /api-keys`: a new key, and the prompt that carries it, which this
/// response is the only place to see.
pub(super) async fn create_api_key(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: ApiKeysRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<CreatedApiKey>> {
    let d1 = ctx.d1(D1_BINDING)?;
    let user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;
    create_key(&d1, &user_id, &request_meta).await.map(Json)
}

/// A new key for `user_id`, and the prompt that carries it.
pub(super) async fn create_key(
    d1: &D1Database,
    user_id: &str,
    request_meta: &RequestMeta,
) -> HandlerResult<CreatedApiKey> {
    let secret = crypto::new_api_key()?;
    let now = now_millis();
    let key = StoredApiKey {
        id: crypto::new_api_key_id()?,
        prefix: shown_prefix(&secret),
        created_at: now,
        last_used_at: None,
    };
    db::insert_api_key(d1, &key, user_id, &hash_token(&secret)).await?;
    let event = AgentEvent::KeyCreated(KeyEvent {
        prefix: key.prefix.clone(),
    });
    log(d1, event, user_id, request_meta).await;
    let origin = request_meta.worker_origin.as_deref().unwrap_or_default();
    Ok(CreatedApiKey {
        key: ApiKeySummary::at(key, now),
        prompt: agent_prompt(origin, &secret),
    })
}

/// `POST /api-keys/{id}/revoke`: the key stops working at once.
pub(super) async fn revoke_api_key(
    ctx: RouteContext<()>,
    path: ApiKeyIdPath,
    body: ApiKeysRequest,
    request_meta: RequestMeta,
) -> HandlerResult<NoContent> {
    let d1 = ctx.d1(D1_BINDING)?;
    let user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;
    revoke_key(&d1, &user_id, &path.id, &request_meta).await?;
    Ok(NoContent)
}

/// Stops key `id` of `user_id` at once. `NotFound` when it has no such key.
pub(super) async fn revoke_key(
    d1: &D1Database,
    user_id: &str,
    id: &str,
    request_meta: &RequestMeta,
) -> HandlerResult<()> {
    let prefix = db::list_api_keys(d1, user_id)
        .await?
        .into_iter()
        .find(|key| key.id == id)
        .map(|key| key.prefix)
        .ok_or(ApiError::NotFound)?;
    if !db::delete_api_key(d1, id, user_id).await? {
        return Err(ApiError::NotFound);
    }
    let event = AgentEvent::KeyRevoked(KeyEvent { prefix });
    log(d1, event, user_id, request_meta).await;
    Ok(())
}
