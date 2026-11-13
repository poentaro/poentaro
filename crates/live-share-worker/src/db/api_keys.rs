//! Queries against the `api_keys` table (see `crate::api_keys`). A key is
//! looked up by its hash; the raw key is never bound to a query.

use serde::Deserialize;
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, Result};

use crate::api_keys::{ApiKeyOwner, StoredApiKey};

const INSERT_API_KEY: &str = include_str!("../../queries/insert_api_key.sql");
const LIST_API_KEYS: &str = include_str!("../../queries/list_api_keys.sql");
const DELETE_API_KEY: &str = include_str!("../../queries/delete_api_key.sql");
const GET_API_KEY: &str = include_str!("../../queries/get_api_key.sql");
const TOUCH_API_KEY: &str = include_str!("../../queries/touch_api_key.sql");

#[derive(Deserialize)]
struct ApiKeyRow {
    id: String,
    prefix: String,
    created_at: i64,
    last_used_at: Option<i64>,
}

#[derive(Deserialize)]
struct ApiKeyOwnerRow {
    id: String,
    user_id: String,
    prefix: String,
    created_at: i64,
    last_used_at: Option<i64>,
}

pub(crate) async fn insert_api_key(
    db: &D1Database,
    key: &StoredApiKey,
    user_id: &str,
    token_hash: &str,
) -> Result<()> {
    db.prepare(INSERT_API_KEY)
        .bind(&[
            JsValue::from_str(&key.id),
            JsValue::from_str(user_id),
            JsValue::from_str(token_hash),
            JsValue::from_str(&key.prefix),
            JsValue::from_f64(key.created_at as f64),
        ])?
        .run()
        .await?;
    Ok(())
}

/// The user's keys, newest first.
pub(crate) async fn list_api_keys(db: &D1Database, user_id: &str) -> Result<Vec<StoredApiKey>> {
    let rows: Vec<ApiKeyRow> = db
        .prepare(LIST_API_KEYS)
        .bind(&[JsValue::from_str(user_id)])?
        .all()
        .await?
        .results()?;
    Ok(rows
        .into_iter()
        .map(|row| StoredApiKey {
            id: row.id,
            prefix: row.prefix,
            created_at: row.created_at,
            last_used_at: row.last_used_at,
        })
        .collect())
}

/// Deletes the user's key `id`, returning whether there was one.
pub(crate) async fn delete_api_key(db: &D1Database, id: &str, user_id: &str) -> Result<bool> {
    let result = db
        .prepare(DELETE_API_KEY)
        .bind(&[JsValue::from_str(id), JsValue::from_str(user_id)])?
        .run()
        .await?;
    Ok(result.meta()?.and_then(|meta| meta.changes).unwrap_or(0) > 0)
}

/// The key whose hash is `token_hash`, and whose it is.
pub(crate) async fn get_api_key(db: &D1Database, token_hash: &str) -> Result<Option<ApiKeyOwner>> {
    let row: Option<ApiKeyOwnerRow> = db
        .prepare(GET_API_KEY)
        .bind(&[JsValue::from_str(token_hash)])?
        .first(None)
        .await?;
    Ok(row.map(|row| ApiKeyOwner {
        id: row.id,
        user_id: row.user_id,
        prefix: row.prefix,
        created_at: row.created_at,
        last_used_at: row.last_used_at,
    }))
}

pub(crate) async fn touch_api_key(db: &D1Database, id: &str, now: i64) -> Result<()> {
    db.prepare(TOUCH_API_KEY)
        .bind(&[JsValue::from_str(id), JsValue::from_f64(now as f64)])?
        .run()
        .await?;
    Ok(())
}
