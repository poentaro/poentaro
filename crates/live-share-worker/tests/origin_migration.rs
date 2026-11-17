//! Tests for `origin_migration`'s expiry and size rules.
#![allow(clippy::disallowed_macros)]

use live_share_worker::origin_migration::{
    is_claimable, payload_rejection, prune_cutoff_millis, MAX_ORIGIN_MIGRATION_PAYLOAD_BYTES,
    ORIGIN_MIGRATION_TTL_MILLIS,
};

#[test]
fn a_handoff_is_claimable_until_its_ttl_runs_out() {
    let created_at = 1_000_000;

    assert!(is_claimable(created_at, created_at));
    assert!(is_claimable(
        created_at,
        created_at + ORIGIN_MIGRATION_TTL_MILLIS
    ));
    assert!(!is_claimable(
        created_at,
        created_at + ORIGIN_MIGRATION_TTL_MILLIS + 1
    ));
}

#[test]
fn prunes_handoffs_older_than_the_ttl() {
    assert_eq!(prune_cutoff_millis(ORIGIN_MIGRATION_TTL_MILLIS + 5), 5);
}

#[test]
fn accepts_a_payload_at_the_limit_and_refuses_one_byte_more() {
    let at_limit = "x".repeat(MAX_ORIGIN_MIGRATION_PAYLOAD_BYTES);
    let over_limit = "x".repeat(MAX_ORIGIN_MIGRATION_PAYLOAD_BYTES + 1);

    assert_eq!(payload_rejection(&at_limit), None);
    assert!(payload_rejection(&over_limit).is_some());
}
