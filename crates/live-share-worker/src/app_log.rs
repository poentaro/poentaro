//! Typed events for the `app_log` table (migrations `0006_app_log.sql` and
//! `0009_app_log_trace_id.sql`): an
//! append-only log in D1 that agents and the owner query with SQL (the saved
//! queries in `queries/observability/`) so production behaviour, e.g. why a
//! user got signed out, can be explained from data instead of guessed.
//!
//! The table is generic (a `flow` column, not a table per flow) and only sees
//! text; the type safety lives here. `LogEvent` is the only way to describe a
//! row: `LogEvent::to_row` splits it into the `flow`/`kind`/`outcome`/
//! `subject`/`detail` columns, deriving `outcome` and `subject` from the
//! variant so they can never contradict it.
//!
//! D1/JsValue-free and `pub` so `tests/app_log.rs` can unit-test it; the D1
//! writes live in the crate-private `db::app_log`.

mod agent;
mod protocol;
mod request;

pub use agent::{AgentEvent, Called, KeyEvent, KeyRejected};
pub use protocol::{ProtocolCalled, ProtocolEvent, ProtocolRefused};
pub use request::{RequestEvent, RequestFailed};

use std::cell::RefCell;
use std::rc::Rc;

use serde::de::Error as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// How long a row is kept before the daily scheduled handler prunes it.
pub const APP_LOG_RETENTION_MILLIS: i64 = 30 * 24 * 60 * 60 * 1000;

/// Rows deleted per `prune_app_log.sql` statement, so one prune never turns
/// into a single huge D1 write.
pub const APP_LOG_PRUNE_BATCH_SIZE: u32 = 5000;

/// Upper bound on prune batches per scheduled run; whatever is left over is
/// picked up by the next day's run.
pub const APP_LOG_PRUNE_MAX_BATCHES: u32 = 10;

/// The git SHA the worker was built from (see `build.rs`), stored in every
/// row's `worker_version` so rows map straight to the deployed commit.
pub const WORKER_VERSION: &str = env!("WORKER_GIT_SHA");

const TOKEN_TAG_LENGTH: usize = 12;

/// Identifies a token without storing it: the first 12 hex chars of its
/// SHA-256 hash (`auth::session::hash_token`'s output).
pub fn token_tag(token_hash: &str) -> String {
    token_hash.chars().take(TOKEN_TAG_LENGTH).collect()
}

/// Rows with `at` before this are pruned.
pub fn prune_cutoff_millis(now_millis: i64) -> i64 {
    now_millis - APP_LOG_RETENTION_MILLIS
}

/// One `app_log` row's worth of event.
///
/// Schema-evolution rule: rows are kept for 30 days and must keep decoding
/// the whole time, so only ever *add* fields to an existing variant's
/// payload, and give every added field `#[serde(default)]`. Never rename or
/// remove a field or variant -- a changed shape is a new variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "flow", content = "event", rename_all = "snake_case")]
pub enum LogEvent {
    Auth(AuthEvent),
    Request(RequestEvent),
    Agent(AgentEvent),
    Protocol(ProtocolEvent),
}

/// An account's life cycle: created, signed in (or refused), locked out,
/// signed out, its password changed, or a session token refused on some
/// other route. Emails are logged; passwords and raw session tokens never
/// are -- a session is named by its `token_tag`.
///
/// The `Token*` and `VerificationUnavailable` variants are from GitHub
/// sign-in, which no longer exists: nothing writes them any more, and they
/// stay only so rows written before the switch keep decoding until pruned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum AuthEvent {
    SignedUp(SignedUp),
    SignedIn(SignedIn),
    SignInFailed(SignInFailed),
    LockedOut(LockedOut),
    SignedOut(SignedOut),
    PasswordChanged(PasswordChanged),
    SessionRejected(SessionRejected),
    TokenIssued(TokenIssued),
    TokenExchangeFailed(TokenExchangeFailed),
    TokenVerified(TokenVerified),
    TokenRejected(TokenRejected),
    VerificationUnavailable(VerificationUnavailable),
    TokenRevoked(TokenRevoked),
    TokenRevokeFailed(TokenRevokeFailed),
}

/// `POST /auth/sign-up` created an account and its first session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedUp {
    pub email: String,
    pub token_tag: String,
}

/// `POST /auth/sign-in` started a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedIn {
    pub email: String,
    pub token_tag: String,
}

/// Why a sign-in (or a change password's current password) was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignInFailureReason {
    /// The email has an account, but the password is not its password.
    WrongPassword,
    /// No account has this email. Answered exactly like a wrong password.
    UnknownEmail,
    /// The email is locked out; the password wasn't checked.
    Locked,
}

/// A sign-in or a change password was refused. `failed_count` is the
/// email's wrong passwords in a row after this one (0 when this one locked
/// it, or when `reason` is `locked`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignInFailed {
    pub email: String,
    pub reason: SignInFailureReason,
    pub failed_count: i64,
}

/// A wrong password locked the email until `locked_until` (milliseconds
/// since the epoch). `lockout_level` counts lockouts since its last
/// successful sign-in, this one included.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockedOut {
    pub email: String,
    pub locked_until: i64,
    pub lockout_level: i64,
}

/// `POST /auth/sign-out` ended a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedOut {
    pub token_tag: String,
}

/// `POST /auth/change-password` changed an account's password and ended
/// its `ended_sessions` other sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasswordChanged {
    pub email: String,
    pub token_tag: String,
    pub ended_sessions: usize,
}

/// A route was called with a session token that names no live session
/// (never existed, expired, signed out, or ended by a password change); the
/// worker answered `unauthorized` and the web app discards the token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRejected {
    pub token_tag: String,
}

/// Legacy (GitHub sign-in). `POST /auth/github/callback` exchanged a code for a new token.
/// `provider_user_id`/`login` come from a best-effort `GET /user` with the
/// new token, so they are `None` when that call failed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenIssued {
    pub token_tag: String,
    #[serde(default)]
    pub provider_user_id: Option<String>,
    #[serde(default)]
    pub login: Option<String>,
}

/// Legacy (GitHub sign-in). GitHub's token endpoint did not hand out a token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenExchangeFailed {
    pub reason: String,
}

/// Legacy (GitHub sign-in). A token not in the verification cache was
/// verified against GitHub.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenVerified {
    pub token_tag: String,
    pub provider_user_id: String,
}

/// Legacy (GitHub sign-in). GitHub's `GET /user` answered 401 for a token -- the worker then answers
/// `unauthorized` and the web app discards the token. `reason` is GitHub's
/// status and body, verbatim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenRejected {
    pub token_tag: String,
    pub reason: String,
    pub attempts: u32,
}

/// Legacy (GitHub sign-in). A token could not be verified for a reason that says nothing about the
/// token (GitHub 5xx or rate limit, network, D1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationUnavailable {
    pub token_tag: String,
    pub reason: String,
    pub attempts: u32,
}

/// Legacy (GitHub sign-in). `POST /auth/github/revoke` revoked a token on
/// sign-out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenRevoked {
    pub token_tag: String,
}

/// Legacy (GitHub sign-in). `POST /auth/github/revoke` failed to revoke a
/// token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenRevokeFailed {
    pub token_tag: String,
    pub reason: String,
}

/// The `outcome` column, derived from the event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Ok,
    Rejected,
    Unavailable,
    Error,
}

impl Outcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Ok => "ok",
            Outcome::Rejected => "rejected",
            Outcome::Unavailable => "unavailable",
            Outcome::Error => "error",
        }
    }
}

/// Where a row's request came from: captured on every request-driven row
/// (sign-in, sign-out, a rejected session, a failed request); `None` on
/// rows with no request, such as the scheduled prune.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RequestMeta {
    pub origin: Option<String>,
    pub user_agent: Option<String>,
    /// `CF-Connecting-IP`, the key of the sign-in rate limits. Never
    /// written to `app_log`.
    pub client_ip: Option<String>,
    /// The request's trace ID: answered in every response's `X-Trace-Id`
    /// header (and a failure's body), and stored on every row the request
    /// writes.
    pub trace_id: Option<String>,
    /// This worker's own origin, as the request reached it, such as
    /// `https://….workers.dev`: where an agent sends `POST /agent`. Never
    /// written to `app_log`.
    pub worker_origin: Option<String>,
    /// The caller's `users.id` once its session token has resolved, so the
    /// row of a request that fails afterwards names who it was. Shared by
    /// every clone of this `RequestMeta`.
    pub resolved_user_id: Rc<RefCell<Option<String>>>,
}

/// The event-derived columns of an `app_log` row (the rest -- `at`,
/// `user_id`, `worker_version`, `origin`, `user_agent`, `trace_id` -- are
/// filled in when the row is written).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRow {
    pub flow: String,
    pub kind: String,
    pub outcome: Outcome,
    pub subject: Option<String>,
    pub detail_json: String,
}

impl AuthEvent {
    fn outcome(&self) -> Outcome {
        match self {
            AuthEvent::SignedUp(_)
            | AuthEvent::SignedIn(_)
            | AuthEvent::SignedOut(_)
            | AuthEvent::PasswordChanged(_) => Outcome::Ok,
            AuthEvent::SignInFailed(_)
            | AuthEvent::LockedOut(_)
            | AuthEvent::SessionRejected(_) => Outcome::Rejected,
            AuthEvent::TokenIssued(_)
            | AuthEvent::TokenVerified(_)
            | AuthEvent::TokenRevoked(_) => Outcome::Ok,
            AuthEvent::TokenRejected(_) => Outcome::Rejected,
            AuthEvent::VerificationUnavailable(_) => Outcome::Unavailable,
            AuthEvent::TokenExchangeFailed(_) | AuthEvent::TokenRevokeFailed(_) => Outcome::Error,
        }
    }

    /// The `subject` column: the account's email when the event is about
    /// one, otherwise the session's token tag.
    fn subject(&self) -> Option<String> {
        let email = |email: &str| Some(format!("email:{email}"));
        let token = |tag: &str| Some(format!("token:{tag}"));
        match self {
            AuthEvent::SignedUp(event) => email(&event.email),
            AuthEvent::SignedIn(event) => email(&event.email),
            AuthEvent::SignInFailed(event) => email(&event.email),
            AuthEvent::LockedOut(event) => email(&event.email),
            AuthEvent::PasswordChanged(event) => email(&event.email),
            AuthEvent::SignedOut(event) => token(&event.token_tag),
            AuthEvent::SessionRejected(event) => token(&event.token_tag),
            AuthEvent::TokenIssued(event) => token(&event.token_tag),
            AuthEvent::TokenExchangeFailed(_) => None,
            AuthEvent::TokenVerified(event) => token(&event.token_tag),
            AuthEvent::TokenRejected(event) => token(&event.token_tag),
            AuthEvent::VerificationUnavailable(event) => token(&event.token_tag),
            AuthEvent::TokenRevoked(event) => token(&event.token_tag),
            AuthEvent::TokenRevokeFailed(event) => token(&event.token_tag),
        }
    }
}

impl LogEvent {
    pub fn outcome(&self) -> Outcome {
        match self {
            LogEvent::Auth(event) => event.outcome(),
            LogEvent::Request(event) => event.outcome(),
            LogEvent::Agent(event) => event.outcome(),
            LogEvent::Protocol(event) => event.outcome(),
        }
    }

    /// What the row is about, as a typed tag (`email:<email>` or
    /// `token:<token tag>`), so rows of one account or one session can be
    /// joined across kinds. A failed request has none: its `user_id` and
    /// `trace_id` columns say who and which request.
    pub fn subject(&self) -> Option<String> {
        match self {
            LogEvent::Auth(event) => event.subject(),
            LogEvent::Request(_) => None,
            LogEvent::Agent(event) => event.subject(),
            LogEvent::Protocol(_) => None,
        }
    }

    /// Splits the serialized `{"flow", "event": {"kind", "detail"}}` into
    /// columns.
    pub fn to_row(&self) -> serde_json::Result<LogRow> {
        let value = serde_json::to_value(self)?;
        let flow = string_field(&value, "flow")?;
        let event = value
            .get("event")
            .ok_or_else(|| serde_json::Error::missing_field("event"))?;
        let kind = string_field(event, "kind")?;
        let detail = event
            .get("detail")
            .ok_or_else(|| serde_json::Error::missing_field("detail"))?;
        Ok(LogRow {
            flow,
            kind,
            outcome: self.outcome(),
            subject: self.subject(),
            detail_json: detail.to_string(),
        })
    }
}

fn string_field(value: &Value, field: &'static str) -> serde_json::Result<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| serde_json::Error::missing_field(field))
}

impl TryFrom<&LogRow> for LogEvent {
    type Error = serde_json::Error;

    /// Reassembles a row read back from `app_log` into its event.
    fn try_from(row: &LogRow) -> serde_json::Result<LogEvent> {
        let detail: Value = serde_json::from_str(&row.detail_json)?;
        serde_json::from_value(serde_json::json!({
            "flow": row.flow,
            "event": { "kind": row.kind, "detail": detail },
        }))
    }
}
