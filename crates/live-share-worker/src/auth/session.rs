//! Sessions: a random token the browser holds (sent as `identity_token`),
//! of which the worker stores only the SHA-256 hash (`sessions.token_hash`).
//! A session lasts `SESSION_LIFETIME_MILLIS` from its last use.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// How long a session lasts after it was last used.
pub const SESSION_LIFETIME_MILLIS: i64 = 60 * 24 * 60 * 60 * 1000;

/// A use only writes a new `expires_at` once less than this is left, so
/// most requests don't write to D1 at all (at most one write a day per
/// session).
pub const SESSION_EXTEND_BELOW_MILLIS: i64 = 59 * 24 * 60 * 60 * 1000;

/// Random bytes in a session token.
pub const SESSION_TOKEN_BYTES: usize = 32;

/// The new `expires_at` for a session used at `now`, or `None` when it has
/// enough time left not to bother.
pub fn extended_expiry(expires_at: i64, now: i64) -> Option<i64> {
    (expires_at - now < SESSION_EXTEND_BELOW_MILLIS).then_some(now + SESSION_LIFETIME_MILLIS)
}

/// A session token's `sessions.token_hash` (lowercase hex SHA-256). Cut down
/// to an `app_log::token_tag`, it also names the session in `app_log`. The
/// token itself is never stored or logged.
pub fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Why a request's `identity_token` was refused (`ApiError::Unauthorized`):
/// it names no session, or one that has expired or been signed out. The web
/// app then discards the token and asks the person to sign in again. Never
/// carries the token or its hash.
///
/// The shape (and the `attempts` field, always 1 now) predates sessions,
/// when this reported a failed GitHub verification after retries; it is
/// kept so the web app's error dialog reads it unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationFailure {
    pub reason: String,
    pub failed_at: i64,
    pub attempts: u32,
}
