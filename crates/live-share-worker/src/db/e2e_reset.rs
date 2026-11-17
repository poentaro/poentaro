//! Test-only hard deletes behind `POST /e2e/reset` -- see
//! `crate::e2e_reset`.

use serde::Deserialize;
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, D1Result, Result};

const LIST_USER_IDS_BY_EMAILS: &str = include_str!("../../queries/list_user_ids_by_emails.sql");
const DELETE_SHARES_OF_USERS: &str = include_str!("../../queries/delete_shares_of_users.sql");
const DELETE_FILES_OF_USERS: &str = include_str!("../../queries/delete_files_of_users.sql");
const DELETE_SESSIONS_OF_USERS: &str = include_str!("../../queries/delete_sessions_of_users.sql");
const DELETE_API_KEYS_OF_USERS: &str = include_str!("../../queries/delete_api_keys_of_users.sql");
const DELETE_PASSWORD_CREDENTIALS_OF_USERS: &str =
    include_str!("../../queries/delete_password_credentials_of_users.sql");
const DELETE_USERS: &str = include_str!("../../queries/delete_users.sql");
const DELETE_SIGN_IN_LOCKOUTS_OF_EMAILS: &str =
    include_str!("../../queries/delete_sign_in_lockouts_of_emails.sql");

/// How many rows `delete_accounts` removed from each table worth reporting.
pub(crate) struct DeletedRows {
    pub(crate) accounts: usize,
    pub(crate) shares: usize,
    pub(crate) files: usize,
}

#[derive(Deserialize)]
struct UserIdRow {
    user_id: String,
}

fn to_json(values: &[String]) -> Result<String> {
    serde_json::to_string(values).map_err(|error| worker::Error::RustError(error.to_string()))
}

/// Deletes the accounts of `emails` outright -- their shares, files,
/// sessions, API keys, credentials and `users` rows -- and the emails' failed
/// sign-ins. One D1 batch, so it's a single transaction, ordered so nothing
/// is deleted while still referenced.
pub(crate) async fn delete_accounts(db: &D1Database, emails: &[String]) -> Result<DeletedRows> {
    let emails_json = JsValue::from_str(&to_json(emails)?);
    let user_ids: Vec<String> = db
        .prepare(LIST_USER_IDS_BY_EMAILS)
        .bind(std::slice::from_ref(&emails_json))?
        .all()
        .await?
        .results::<UserIdRow>()?
        .into_iter()
        .map(|row| row.user_id)
        .collect();
    let user_ids_json = [JsValue::from_str(&to_json(&user_ids)?)];
    let results = db
        .batch(vec![
            db.prepare(DELETE_SHARES_OF_USERS).bind(&user_ids_json)?,
            db.prepare(DELETE_FILES_OF_USERS).bind(&user_ids_json)?,
            db.prepare(DELETE_SESSIONS_OF_USERS).bind(&user_ids_json)?,
            db.prepare(DELETE_API_KEYS_OF_USERS).bind(&user_ids_json)?,
            db.prepare(DELETE_PASSWORD_CREDENTIALS_OF_USERS)
                .bind(&user_ids_json)?,
            db.prepare(DELETE_USERS).bind(&user_ids_json)?,
            db.prepare(DELETE_SIGN_IN_LOCKOUTS_OF_EMAILS)
                .bind(&[emails_json])?,
        ])
        .await?;
    let [shares, files, _sessions, _api_keys, _credentials, users, _lockouts] = results.as_slice()
    else {
        return Err(worker::Error::RustError(format!(
            "expected 7 batch results, got {}",
            results.len()
        )));
    };
    Ok(DeletedRows {
        accounts: changes(users)?,
        shares: changes(shares)?,
        files: changes(files)?,
    })
}

fn changes(result: &D1Result) -> Result<usize> {
    Ok(result.meta()?.and_then(|meta| meta.changes).unwrap_or(0))
}
