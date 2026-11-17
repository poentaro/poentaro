//! Unit tests for `files::idempotent_create_match` -- the decision behind
//! `POST /files` being idempotent for a repeated id.
#![allow(clippy::disallowed_macros)]

use live_share_worker::files::{idempotent_create_match, to_public_file, StoredFile};

fn existing_file() -> StoredFile {
    StoredFile {
        id: "file-1".to_string(),
        owner_user_id: "owner-1".to_string(),
        name: "song.poentaro".to_string(),
        content: "1 2 3".to_string(),
        revision: 0,
        trashed_at: None,
        created_at: 1_000,
        updated_at: 1_000,
    }
}

#[test]
fn same_id_owner_and_name_returns_the_existing_file() {
    let existing = existing_file();

    let result = idempotent_create_match(Some(&existing), "song.poentaro");

    assert_eq!(result, Some(to_public_file(&existing)));
}

#[test]
fn same_name_under_a_different_id_is_name_taken() {
    // The lookup by the requested id finds nothing, because the colliding
    // row has another id.
    let result = idempotent_create_match(None, "song.poentaro");

    assert_eq!(result, None);
}

#[test]
fn same_id_owned_by_someone_else_is_name_taken() {
    // The owner-scoped lookup finds nothing for a row owned by another user.
    let result = idempotent_create_match(None, "song.poentaro");

    assert_eq!(result, None);
}

#[test]
fn same_id_with_a_different_name_is_name_taken() {
    let existing = existing_file();

    let result = idempotent_create_match(Some(&existing), "other.poentaro");

    assert_eq!(result, None);
}
