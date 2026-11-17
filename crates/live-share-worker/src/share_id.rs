//! Random id generation: server-side `share_id`s (per
//! `TODO-synced-share-rust-d1-migration.md` §1 -- no more client-derived
//! id) and the pseudo `users.id` the identity stub mints on
//! create-on-first-sight.
//!
//! Uses `js_sys::Math::random()` (re-exported by `worker`) rather than
//! pulling in a `rand`/`getrandom` dependency -- it's already available in
//! every Worker's JS environment and keeps this crate's dependency surface
//! small.
//!
//! `generate_unique_id` is D1-/worker-free (generic over an injected
//! `exists` check), so `tests/share_id.rs` exercises the collision-handling
//! loop with a fake `exists` closure -- no real D1 needed. Matches this
//! crate's existing split between pure/testable logic and
//! D1-/wasm-runtime-facing glue (see `lib.rs`'s module doc comment).

use std::future::Future;

use serde::Serialize;
use worker::{D1Database, Result};

use crate::db;

/// Every `share_id` this crate mints is exactly this many characters,
/// including on collision (see `generate_unique_share_id`) -- no consumer
/// of a `share_id`, here or client-side, needs to handle more than one
/// length. The single source of truth for the web client too: it reads
/// this via `share_id_format` (see there), never a copy of its own.
/// 11 base64url characters carry 66 bits -- plenty to keep accidental
/// collisions negligible (shares are never deleted) while staying short,
/// since every character lands directly in a copy-pasted URL.
pub(crate) const SHARE_ID_LENGTH: usize = 11;

/// The base64url alphabet, so an id is URL-safe as-is. Exposed to the web
/// client only through `share_id_format`'s `pattern`.
const ID_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";

/// The shape of every `share_id` this crate mints, derived from
/// `SHARE_ID_LENGTH` and `ID_CHARSET`. Published two ways from this one
/// function: as the `share_id` path param's schema in the OpenAPI spec
/// (`share_id_schema`), and as
/// `web/src/generated/live-share-worker/shareIdFormat.json`
/// (`tests/export_web_config.rs`), which `web/src/syncedShareUrl.ts` and
/// `web/functions/index.ts` read to parse a share link.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShareIdFormat {
    /// Exact length of every `share_id`.
    pub length: usize,
    /// Anchored regex (JS `RegExp` and JSON Schema compatible) a whole
    /// `share_id` matches.
    pub pattern: String,
}

pub fn share_id_format() -> ShareIdFormat {
    let character_class: String = ID_CHARSET
        .iter()
        .map(|&byte| escape_in_character_class(byte as char))
        .collect();
    ShareIdFormat {
        length: SHARE_ID_LENGTH,
        pattern: format!("^[{character_class}]{{{SHARE_ID_LENGTH}}}$"),
    }
}

/// Escapes the characters that are special inside a regex `[...]` class.
fn escape_in_character_class(character: char) -> String {
    match character {
        '\\' | ']' | '[' | '^' | '-' => format!("\\{character}"),
        _ => character.to_string(),
    }
}

pub(crate) fn generate_id(length: usize) -> String {
    (0..length).map(|_| random_char()).collect()
}

fn random_char() -> char {
    let index = (worker::js_sys::Math::random() * ID_CHARSET.len() as f64) as usize;
    let index = index.min(ID_CHARSET.len().saturating_sub(1));
    let byte = ID_CHARSET.get(index).copied().unwrap_or(b'0');
    byte as char
}

/// Generates a fresh id, re-rolling a brand new candidate (via
/// `generate_candidate`, same length every time, never extended) on each
/// collision reported by `exists`, per the TODO §8 fix for "shareId
/// collision fallback can mint an id the client can't parse back out of a
/// link": the earlier length-extension fallback minted a 12-char id that
/// the client's fixed-length `ShareIdFormat` pattern couldn't parse.
/// Re-rolling instead keeps every id this crate mints exactly the
/// same length, so no consumer needs to handle more than one length.
///
/// Takes `generate_candidate` as a parameter (rather than calling
/// `generate_id` directly) so this loop is `js_sys`-/worker-free and
/// `tests/share_id.rs` can drive it with a fake, deterministic candidate
/// generator -- `js_sys::Math::random()` (used by the real `generate_id`)
/// panics outside a wasm target.
///
/// This is a true loop rather than a fixed number of attempts, but a
/// second collision is astronomically unlikely at this charset/length
/// (per TODO §1's original collision-handling decision), so it isn't a
/// practical infinite loop.
pub async fn generate_unique_id<GenerateCandidate, Exists, ExistsFut, E>(
    mut generate_candidate: GenerateCandidate,
    mut exists: Exists,
) -> Result<String, E>
where
    GenerateCandidate: FnMut() -> String,
    Exists: FnMut(String) -> ExistsFut,
    ExistsFut: Future<Output = Result<bool, E>>,
{
    loop {
        let candidate = generate_candidate();
        if !exists(candidate.clone()).await? {
            return Ok(candidate);
        }
    }
}

/// Generates a fresh, D1-unique `share_id`.
pub(crate) async fn generate_unique_share_id(db: &D1Database) -> Result<String> {
    generate_unique_id(
        || generate_id(SHARE_ID_LENGTH),
        |candidate| async move { db::share_id_exists(db, &candidate).await },
    )
    .await
}
