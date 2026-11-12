//! API keys: what an account owner's AI agent sends to `POST /agent` as
//! `Authorization: Bearer poe_…`. The worker stores only the key's SHA-256
//! hash (`auth::session::hash_token`), and shows the key once, when it is
//! created. A key acts as its owner, on the owner's own files only.
//!
//! D1-free and `pub` so `tests/api_keys.rs` can test it; the queries live in
//! `db::api_keys` and the routes in `handlers::api_keys`.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// What every key starts with, so a leaked key is recognisable.
pub const KEY_PREFIX: &str = "poe_";

/// Random bytes in a key.
pub const KEY_BYTES: usize = 32;

/// Characters of a key kept as its `prefix`, which names it in the account
/// menu and in `app_log`: `poe_` and 8 more.
const SHOWN_LENGTH: usize = 12;

/// `last_used_at` is only written once it is this old, so an agent's burst
/// of calls writes to D1 at most once a minute.
pub const TOUCH_EVERY_MILLIS: i64 = 60 * 1000;

/// A key as the account menu lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredApiKey {
    pub id: String,
    pub prefix: String,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
}

/// The key a request was sent with, and whose it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiKeyOwner {
    pub id: String,
    pub user_id: String,
    pub prefix: String,
    pub last_used_at: Option<i64>,
}

/// The key in an `Authorization` header: `Bearer poe_…`.
pub fn bearer_key(authorization: Option<&str>) -> Option<&str> {
    let key = authorization?.trim().strip_prefix("Bearer ")?.trim();
    key.starts_with(KEY_PREFIX).then_some(key)
}

/// The start of `key` that names it.
pub fn shown_prefix(key: &str) -> String {
    key.chars().take(SHOWN_LENGTH).collect()
}

/// Whether a use at `now` should write a new `last_used_at`.
pub fn should_touch(last_used_at: Option<i64>, now: i64) -> bool {
    last_used_at.is_none_or(|last| now - last >= TOUCH_EVERY_MILLIS)
}

/// What the account owner pastes into their AI agent, with the key and
/// this worker's `origin` filled in: it sends the agent to `GET /agent`.
pub fn agent_prompt(origin: &str, key: &str) -> String {
    format!(
        "Help me with my music scores on Poentaro. Before anything else, run\n\
         curl -s {origin}/agent -H \"Authorization: Bearer {key}\"\n\
         and follow what it says.\n"
    )
}

/// A key in a response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeySummary {
    pub id: String,
    /// The start of the key, such as `poe_AbCd1234`.
    pub prefix: String,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
}

impl From<StoredApiKey> for ApiKeySummary {
    fn from(key: StoredApiKey) -> Self {
        ApiKeySummary {
            id: key.id,
            prefix: key.prefix,
            created_at: key.created_at,
            last_used_at: key.last_used_at,
        }
    }
}

/// Body of `POST /api-keys`, `POST /api-keys/list` and
/// `POST /api-keys/{id}/revoke`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeysRequest {
    pub identity_token: String,
}

/// Response of `POST /api-keys/list`: the caller's keys, newest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListApiKeysResponse {
    pub keys: Vec<ApiKeySummary>,
}

/// Response of `POST /api-keys`. `prompt` holds the whole key; neither is
/// shown again.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatedApiKey {
    pub key: ApiKeySummary,
    /// What the owner pastes into their agent (`agent_prompt`).
    pub prompt: String,
}

/// Path parameters of `POST /api-keys/{id}/revoke`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct ApiKeyIdPath {
    pub id: String,
}
