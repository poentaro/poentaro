//! The account routes: `POST /auth/sign-up`, `/auth/sign-in`,
//! `/auth/sign-out` and `/auth/change-password`. Every rule (what an email
//! and a password must look like, lockouts, session lifetime) lives here and
//! in `crate::auth`; the web app only shows what these answer.
//!
//! Passwords and raw session tokens are never logged or stored: a password
//! only ever reaches `crate::crypto::pbkdf2_sha256`, and a session is stored
//! and logged by its hash (`auth::session::hash_token`).

use worker::{D1Database, Env, RouteContext};

use super::routes::{HandlerResult, Json, NoContent, NoPathParams};
use super::{
    check_rate_limit, is_unique_constraint_violation, now_millis, resolve_files_caller, D1_BINDING,
};
use crate::api_error::ApiError;
use crate::app_log::{
    self, AuthEvent, LockedOut, LogEvent, PasswordChanged, RequestMeta, SignInFailed,
    SignInFailureReason, SignedIn, SignedOut, SignedUp,
};
use crate::auth::email::normalize_email;
use crate::auth::lockout::{after_failed_sign_in, retry_after_seconds};
use crate::auth::password::{password_weakness, PasswordHash, PBKDF2_ITERATIONS, SALT_LENGTH};
use crate::auth::session::{hash_token, SESSION_LIFETIME_MILLIS};
use crate::crypto;
use crate::db::{self, PasswordCredentials};
use crate::protocol::{ChangePasswordRequest, CredentialsRequest, SessionResponse, SignOutRequest};

/// Rate limit bindings (`cloudflare.config.ts`), keyed per client IP.
const SIGN_IN_RATE_LIMITER: &str = "SIGN_IN_RATE_LIMITER";
const SIGN_UP_RATE_LIMITER: &str = "SIGN_UP_RATE_LIMITER";

async fn hash_new_password(password: &str) -> HandlerResult<String> {
    let salt = crypto::random_bytes(SALT_LENGTH)?;
    let hash = crypto::pbkdf2_sha256(password, &salt, PBKDF2_ITERATIONS).await?;
    Ok(PasswordHash {
        iterations: PBKDF2_ITERATIONS,
        salt,
        hash,
    }
    .encode())
}

async fn log(db: &D1Database, event: AuthEvent, user_id: Option<&str>, request_meta: &RequestMeta) {
    db::record_app_log(db, &LogEvent::Auth(event), user_id, request_meta).await;
}

/// A new random session token and its hash.
struct NewSession {
    token: String,
    token_hash: String,
}

fn new_session() -> HandlerResult<NewSession> {
    let token = crypto::new_session_token()?;
    let token_hash = hash_token(&token);
    Ok(NewSession { token, token_hash })
}

/// `POST /auth/sign-up`.
pub(super) async fn sign_up(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: CredentialsRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<SessionResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    create_account(&ctx.env, &d1, &body.email, &body.password, &request_meta)
        .await
        .map(Json)
}

/// Creates an account and signs it in.
pub(super) async fn create_account(
    env: &Env,
    d1: &D1Database,
    email: &str,
    password: &str,
    request_meta: &RequestMeta,
) -> HandlerResult<SessionResponse> {
    check_rate_limit(env, SIGN_UP_RATE_LIMITER, request_meta).await?;
    let email = normalize_email(email).ok_or(ApiError::InvalidEmail)?;
    if let Some(message) = password_weakness(password) {
        return Err(ApiError::WeakPassword { message });
    }
    let now = now_millis();
    let credentials = PasswordCredentials {
        user_id: crypto::new_user_id()?,
        email,
        password_hash: hash_new_password(password).await?,
    };
    let session = new_session()?;
    let first_session = db::insert_session_statement(
        d1,
        &session.token_hash,
        &credentials.user_id,
        now,
        now + SESSION_LIFETIME_MILLIS,
    )?;
    match db::insert_account(d1, &credentials, now, vec![first_session]).await {
        Ok(()) => {}
        Err(error) if is_unique_constraint_violation(&error) => return Err(ApiError::EmailTaken),
        Err(error) => return Err(error.into()),
    }
    let event = AuthEvent::SignedUp(SignedUp {
        email: credentials.email.clone(),
        token_tag: app_log::token_tag(&session.token_hash),
    });
    log(d1, event, Some(&credentials.user_id), request_meta).await;
    Ok(SessionResponse {
        session_token: session.token,
        email: credentials.email,
    })
}

/// Checks `password` against `credentials` (the account of `email`, or
/// `None` when it has none) under `email`'s lockout: refused unchecked while
/// locked, and each wrong password counts toward the next lockout -- the
/// one that locks it answers `SignInLocked` straight away. An unknown email
/// costs the same hashing and counts the same as a wrong password, so
/// neither the answer, its timing nor the lockout says whether an account
/// exists. Success clears the email's failed sign-ins.
async fn check_password(
    d1: &D1Database,
    email: &str,
    credentials: Option<PasswordCredentials>,
    password: &str,
    request_meta: &RequestMeta,
) -> HandlerResult<PasswordCredentials> {
    let now = now_millis();
    let lockout = db::get_sign_in_lockout(d1, email).await?;
    if let Some(retry_after_seconds) = lockout
        .as_ref()
        .and_then(|state| retry_after_seconds(state, now))
    {
        let event = AuthEvent::SignInFailed(SignInFailed {
            email: email.to_string(),
            reason: SignInFailureReason::Locked,
            failed_count: 0,
        });
        log(d1, event, None, request_meta).await;
        return Err(ApiError::SignInLocked {
            retry_after_seconds,
        });
    }

    let stored = credentials
        .as_ref()
        .and_then(|credentials| PasswordHash::parse(&credentials.password_hash));
    let (salt, iterations) = stored.as_ref().map_or_else(
        || (vec![0; SALT_LENGTH], PBKDF2_ITERATIONS),
        |stored| (stored.salt.clone(), stored.iterations),
    );
    let derived = crypto::pbkdf2_sha256(password, &salt, iterations).await?;
    let matched = stored
        .as_ref()
        .is_some_and(|stored| stored.matches(&derived));

    match credentials {
        Some(credentials) if matched => {
            if lockout.is_some() {
                db::delete_sign_in_lockout(d1, email).await?;
            }
            Ok(credentials)
        }
        credentials => {
            let reason = if credentials.is_some() {
                SignInFailureReason::WrongPassword
            } else {
                SignInFailureReason::UnknownEmail
            };
            let next = after_failed_sign_in(lockout.as_ref(), now);
            db::upsert_sign_in_lockout(d1, email, &next).await?;
            let user_id = credentials
                .as_ref()
                .map(|credentials| credentials.user_id.as_str());
            let event = AuthEvent::SignInFailed(SignInFailed {
                email: email.to_string(),
                reason,
                failed_count: next.failed_count,
            });
            log(d1, event, user_id, request_meta).await;
            let newly_locked =
                next.lockout_level > lockout.as_ref().map_or(0, |state| state.lockout_level);
            match (newly_locked, next.locked_until) {
                (true, Some(locked_until)) => {
                    let event = AuthEvent::LockedOut(LockedOut {
                        email: email.to_string(),
                        locked_until,
                        lockout_level: next.lockout_level,
                    });
                    log(d1, event, user_id, request_meta).await;
                    Err(ApiError::SignInLocked {
                        retry_after_seconds: retry_after_seconds(&next, now).unwrap_or(0),
                    })
                }
                _ => Err(ApiError::IncorrectCredentials),
            }
        }
    }
}

/// `POST /auth/sign-in`.
pub(super) async fn sign_in(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: CredentialsRequest,
    request_meta: RequestMeta,
) -> HandlerResult<Json<SessionResponse>> {
    let d1 = ctx.d1(D1_BINDING)?;
    start_session(&ctx.env, &d1, &body.email, &body.password, &request_meta)
        .await
        .map(Json)
}

/// Starts a session for an email and its password.
pub(super) async fn start_session(
    env: &Env,
    d1: &D1Database,
    email: &str,
    password: &str,
    request_meta: &RequestMeta,
) -> HandlerResult<SessionResponse> {
    check_rate_limit(env, SIGN_IN_RATE_LIMITER, request_meta).await?;
    let email = normalize_email(email).ok_or(ApiError::InvalidEmail)?;
    let credentials = db::get_password_credentials_by_email(d1, &email).await?;
    let credentials = check_password(d1, &email, credentials, password, request_meta).await?;
    let now = now_millis();
    let session = new_session()?;
    db::insert_session(
        d1,
        &session.token_hash,
        &credentials.user_id,
        now,
        now + SESSION_LIFETIME_MILLIS,
    )
    .await?;
    let event = AuthEvent::SignedIn(SignedIn {
        email: credentials.email.clone(),
        token_tag: app_log::token_tag(&session.token_hash),
    });
    log(d1, event, Some(&credentials.user_id), request_meta).await;
    Ok(SessionResponse {
        session_token: session.token,
        email: credentials.email,
    })
}

/// `POST /auth/sign-out`.
pub(super) async fn sign_out(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: SignOutRequest,
    request_meta: RequestMeta,
) -> HandlerResult<NoContent> {
    let d1 = ctx.d1(D1_BINDING)?;
    end_session(&d1, &body.identity_token, &request_meta).await?;
    Ok(NoContent)
}

/// Ends one session (this device's). Answers the same whether or not the
/// session still existed.
pub(super) async fn end_session(
    d1: &D1Database,
    token: &str,
    request_meta: &RequestMeta,
) -> HandlerResult<()> {
    let token_hash = hash_token(token);
    db::delete_session(d1, &token_hash).await?;
    let event = AuthEvent::SignedOut(SignedOut {
        token_tag: app_log::token_tag(&token_hash),
    });
    log(d1, event, None, request_meta).await;
    Ok(())
}

/// `POST /auth/change-password`.
pub(super) async fn change_password(
    ctx: RouteContext<()>,
    _path: NoPathParams,
    body: ChangePasswordRequest,
    request_meta: RequestMeta,
) -> HandlerResult<NoContent> {
    let d1 = ctx.d1(D1_BINDING)?;
    let user_id = resolve_files_caller(&d1, &body.identity_token, &request_meta).await?;
    let session = SignedInSession {
        token: &body.identity_token,
        user_id: &user_id,
    };
    let passwords = PasswordChange {
        current: &body.current_password,
        new: &body.new_password,
    };
    replace_password(&d1, session, passwords, &request_meta).await?;
    Ok(NoContent)
}

/// A session that resolved to its account.
#[derive(Clone, Copy)]
pub(super) struct SignedInSession<'a> {
    pub(super) token: &'a str,
    pub(super) user_id: &'a str,
}

pub(super) struct PasswordChange<'a> {
    pub(super) current: &'a str,
    pub(super) new: &'a str,
}

/// Replaces the signed-in account's password and ends every other session
/// it has; `session` survives. A wrong current password counts toward the
/// email's lockout, like a wrong password at sign-in.
pub(super) async fn replace_password(
    d1: &D1Database,
    session: SignedInSession<'_>,
    passwords: PasswordChange<'_>,
    request_meta: &RequestMeta,
) -> HandlerResult<()> {
    let user_id = session.user_id;
    if let Some(message) = password_weakness(passwords.new) {
        return Err(ApiError::WeakPassword { message });
    }
    let credentials = db::get_password_credentials_by_user(d1, user_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let email = credentials.email.clone();
    let credentials = check_password(
        d1,
        &email,
        Some(credentials),
        passwords.current,
        request_meta,
    )
    .await?;
    let password_hash = hash_new_password(passwords.new).await?;
    let token_hash = hash_token(session.token);
    let results = d1
        .batch(vec![
            db::update_password_hash_statement(d1, user_id, &password_hash, now_millis())?,
            db::delete_other_sessions_statement(d1, user_id, &token_hash)?,
        ])
        .await?;
    let ended_sessions = results
        .get(1)
        .map(|result| result.meta())
        .transpose()?
        .flatten()
        .and_then(|meta| meta.changes)
        .unwrap_or(0);
    let event = AuthEvent::PasswordChanged(PasswordChanged {
        email: credentials.email,
        token_tag: app_log::token_tag(&token_hash),
        ended_sessions,
    });
    log(d1, event, Some(user_id), request_meta).await;
    Ok(())
}
