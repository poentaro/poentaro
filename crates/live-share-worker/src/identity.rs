//! Resolves the `identity_token` every authenticated route takes -- the
//! file storage routes (`/files/*`) and the Synced Share owner routes alike
//! -- to the internal `users.id` it signs in as. The token is a session
//! token from `POST /auth/sign-up` or `POST /auth/sign-in` (see
//! `crate::handlers::auth`); the worker looks up its hash in `sessions` and
//! never trusts a client-asserted identity.

use worker::{D1Database, Result};

use crate::app_log::{self, AuthEvent, LogEvent, RequestMeta, SessionRejected};
use crate::auth::session::{extended_expiry, hash_token, VerificationFailure};
use crate::db;

/// What the web app is told when a token names no live session.
const SESSION_REJECTED_REASON: &str =
    "Your session has ended (signed out, expired, or the password was changed). Sign in again.";

/// Resolves `identity_token` to a `user_id`:
///
/// 1. Hash the token (SHA-256; the raw token is never stored) and look up
///    an unexpired `sessions` row.
/// 2. If there is one, note its user in `request_meta` (for the request's
///    `app_log` row, should it fail later), slide its expiry forward when
///    it's due (`auth::session::extended_expiry`) and return its user.
/// 3. If not, log `SessionRejected` and return the inner `Err`.
///
/// The split between the two `Err`s matters to the web app
/// (`useCloudStoreLoader.ts`): the inner one means the token is dead and the
/// person must sign in again, so callers turn it into
/// `ApiError::Unauthorized`; the outer one (a D1 failure) says nothing about
/// the token, so it surfaces as a retryable server error and the token is
/// kept.
pub(crate) async fn resolve_verified_user_id(
    db: &D1Database,
    identity_token: &str,
    request_meta: &RequestMeta,
) -> Result<Result<String, VerificationFailure>> {
    let token_hash = hash_token(identity_token);
    let now = worker::Date::now().as_millis() as i64;

    let Some(session) = db::get_live_session(db, &token_hash, now).await? else {
        let event = AuthEvent::SessionRejected(SessionRejected {
            token_tag: app_log::token_tag(&token_hash),
        });
        db::record_app_log(db, &LogEvent::Auth(event), None, request_meta).await;
        return Ok(Err(VerificationFailure {
            reason: SESSION_REJECTED_REASON.to_string(),
            failed_at: now,
            attempts: 1,
        }));
    };

    request_meta
        .resolved_user_id
        .replace(Some(session.user_id.clone()));
    if let Some(expires_at) = extended_expiry(session.expires_at, now) {
        db::extend_session(db, &token_hash, expires_at).await?;
    }
    Ok(Ok(session.user_id))
}
