//! Unit tests for `files::classify_content_write` -- the one piece of the
//! cloud storage backend's content-save conflict logic worth testing in
//! isolation from real D1 (per this crate's hard convention: no test here
//! touches D1, only `cf dev` + Playwright e2e do -- see `files.rs`'s
//! module doc comment).
#![allow(clippy::disallowed_macros)]

use live_share_worker::files::{
    classify_content_write, ContentWriteAttempt, ContentWriteOutcome, StoredFile,
};

fn sample_file() -> StoredFile {
    StoredFile {
        id: "file-1".to_string(),
        owner_user_id: "owner-1".to_string(),
        name: "song.poentaro".to_string(),
        content: "1 2 3".to_string(),
        revision: 4,
        trashed_at: None,
        created_at: 1_000,
        updated_at: 1_000,
    }
}

#[test]
fn applied_reports_the_expected_revision_plus_one() {
    let result = classify_content_write(ContentWriteAttempt::Applied, 4, None);

    assert_eq!(result, ContentWriteOutcome::Applied { new_revision: 5 });
}

#[test]
fn zero_rows_affected_with_a_present_row_at_a_different_revision_is_a_conflict() {
    let current = StoredFile {
        revision: 7,
        ..sample_file()
    };

    let result = classify_content_write(ContentWriteAttempt::RowsAffectedZero, 4, Some(&current));

    assert_eq!(
        result,
        ContentWriteOutcome::Conflict {
            current_revision: 7
        }
    );
}

#[test]
fn zero_rows_affected_with_a_trashed_row_is_a_conflict_not_a_not_found() {
    // The write's own `WHERE ... trashed_at IS NULL` guard is what makes a
    // trashed row report zero rows affected; the row still exists (and is
    // still owned by the caller), so this is "moved on", not "gone".
    let current = StoredFile {
        trashed_at: Some(2_000),
        ..sample_file()
    };

    let result = classify_content_write(ContentWriteAttempt::RowsAffectedZero, 4, Some(&current));

    assert_eq!(
        result,
        ContentWriteOutcome::Conflict {
            current_revision: 4
        }
    );
}

#[test]
fn zero_rows_affected_with_no_row_found_is_not_found() {
    let result = classify_content_write(ContentWriteAttempt::RowsAffectedZero, 4, None);

    assert_eq!(result, ContentWriteOutcome::NotFound);
}
