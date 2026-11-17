//! `/api-keys` routes: the account menu creates, lists and revokes the keys
//! an AI agent sends to `POST /agent` (see `crate::api_keys`). Each needs a
//! signed-in session, like `/files`.

use worker::D1Database;

use super::now_millis;
use super::HandlerResult;
use crate::api_error::ApiError;
use crate::api_keys::{agent_prompt, shown_prefix, ApiKeySummary, CreatedApiKey, StoredApiKey};
use crate::app_log::{AgentEvent, KeyEvent, LogEvent, RequestMeta};
use crate::auth::session::hash_token;
use crate::{crypto, db};

async fn log(db: &D1Database, event: AgentEvent, user_id: &str, request_meta: &RequestMeta) {
    db::record_app_log(db, &LogEvent::Agent(event), Some(user_id), request_meta).await;
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
