//! One-time handoffs of a browser's saved data from an old web origin to
//! the app's current one (the app moved from `jianpu-generator.pages.dev` to
//! `poentaro.pages.dev`). Browsers keep `localStorage` per origin, so a plain
//! redirect would strand every local file and sign the user out. Instead
//! the old origin's bridge page uploads what it kept
//! (`POST /origin-migrations`), redirects with the returned id, and the new
//! origin claims it (`POST /origin-migrations/claim`), which deletes it.
//!
//! The worker treats the payload as opaque text: it only bounds its size
//! and lifetime. The id is the only credential, so it is long and drawn
//! from the runtime's CSPRNG (`crate::crypto`), like a session token.

/// How long an unclaimed handoff stays claimable. The bridge redirects
/// straight after uploading, so this only needs to cover a slow page load.
pub const ORIGIN_MIGRATION_TTL_MILLIS: i64 = 60 * 60 * 1000;

/// The largest payload accepted, in bytes: just under D1's 2,000,000-byte
/// limit on a single row.
pub const MAX_ORIGIN_MIGRATION_PAYLOAD_BYTES: usize = 1_900_000;

/// Random bytes in a handoff id (base64url-encoded on the wire).
pub const ORIGIN_MIGRATION_ID_BYTES: usize = 24;

/// Whether a handoff created at `created_at_millis` can still be claimed at
/// `now_millis`.
pub fn is_claimable(created_at_millis: i64, now_millis: i64) -> bool {
    now_millis.saturating_sub(created_at_millis) <= ORIGIN_MIGRATION_TTL_MILLIS
}

/// Why a payload was refused, if it was.
pub fn payload_rejection(payload: &str) -> Option<String> {
    (payload.len() > MAX_ORIGIN_MIGRATION_PAYLOAD_BYTES).then(|| {
        format!(
            "saved data is {} bytes, more than the {MAX_ORIGIN_MIGRATION_PAYLOAD_BYTES}-byte limit",
            payload.len()
        )
    })
}

/// Handoffs created before this cutoff are pruned.
pub fn prune_cutoff_millis(now_millis: i64) -> i64 {
    now_millis.saturating_sub(ORIGIN_MIGRATION_TTL_MILLIS)
}
