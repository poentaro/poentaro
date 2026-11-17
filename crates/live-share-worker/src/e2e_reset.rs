//! The test-only `POST /e2e/reset` route's configuration (the route itself
//! is `crate::handlers::e2e_reset`). Local e2e runs reuse one Miniflare D1
//! state dir across invocations, so rows the synthetic e2e accounts create
//! pile up between runs -- e.g. every run reserves one more "source N"
//! default name -- and this route deletes those accounts outright before
//! each run (`web/e2e/global-setup.ts`), and before scenarios that need an
//! account in a known state (a fresh sign-up, a lockout).
//!
//! Gated on `EMAILS_VAR`: the route is only registered into the router when
//! that var is set, so without it the worker answers `/e2e/reset` exactly as
//! it does any unknown path. The real `cloudflare.config.ts` never sets it;
//! only e2e's local `cf dev` passes it (see `web/playwright.config.ts`). Its
//! value is also the allowlist: a JSON array of the emails whose accounts
//! the route may delete, so the route can never touch any other account.
//!
//! D1-free, so `tests/e2e_reset.rs` can test the parsing directly.

use crate::auth::email::normalize_email;

/// `[vars]` name that both enables the route and lists the emails it may
/// reset. Published to the web through
/// `e2e_worker_config::e2e_worker_config`, never copied.
pub const EMAILS_VAR: &str = "SYNCED_SHARE_E2E_RESET_EMAILS";

/// Parses `EMAILS_VAR`'s value (a JSON array of emails) into the stored,
/// normalized form. A value that isn't an email fails the whole parse.
pub fn parse_reset_emails(value: &str) -> Result<Vec<String>, String> {
    let emails: Vec<String> = serde_json::from_str(value).map_err(|error| error.to_string())?;
    emails
        .iter()
        .map(|email| normalize_email(email).ok_or_else(|| format!("{email:?} is not an email")))
        .collect()
}

/// Which emails one reset deletes: `requested` (normalized) when given --
/// refused, naming the first email not in `allowed`, unless every one is --
/// otherwise all of `allowed`.
pub fn emails_to_reset(
    allowed: &[String],
    requested: Option<&[String]>,
) -> Result<Vec<String>, String> {
    let Some(requested) = requested else {
        return Ok(allowed.to_vec());
    };
    requested
        .iter()
        .map(|email| {
            normalize_email(email)
                .filter(|email| allowed.contains(email))
                .ok_or_else(|| email.clone())
        })
        .collect()
}
