//! Unit tests for `files::validate_file_name` -- the path rules every
//! create, rename and restore of a cloud file name must pass.
#![allow(clippy::disallowed_macros)]

use live_share_worker::api_error::ApiError;
use live_share_worker::files::validate_file_name;

fn is_invalid(name: &str) -> bool {
    matches!(validate_file_name(name), Err(ApiError::InvalidName { .. }))
}

#[test]
fn accepts_a_flat_name() {
    assert!(validate_file_name("song.poentaro").is_ok());
}

#[test]
fn accepts_a_nested_path_with_spaces() {
    assert!(validate_file_name("Hymns/Advent/O Come.poentaro").is_ok());
}

#[test]
fn rejects_an_empty_name() {
    assert!(is_invalid(""));
}

#[test]
fn rejects_empty_segments() {
    assert!(is_invalid("/song.poentaro"));
    assert!(is_invalid("Hymns/"));
    assert!(is_invalid("Hymns//song.poentaro"));
    assert!(is_invalid("Hymns/ /song.poentaro"));
}

#[test]
fn rejects_relative_segments() {
    assert!(is_invalid("./song.poentaro"));
    assert!(is_invalid("Hymns/../song.poentaro"));
}

#[test]
fn accepts_exactly_the_byte_limit() {
    let name = "a".repeat(1024);

    assert!(validate_file_name(&name).is_ok());
}

#[test]
fn rejects_one_byte_over_the_limit() {
    let name = "a".repeat(1025);

    assert!(is_invalid(&name));
}

#[test]
fn counts_bytes_not_characters() {
    // 342 CJK characters are 1026 bytes but only 342 characters.
    let name = "和".repeat(342);

    assert!(is_invalid(&name));
    assert!(validate_file_name(&"和".repeat(341)).is_ok());
}

#[test]
fn invalid_name_is_a_400_with_the_reason() -> Result<(), serde_json::Error> {
    let error = validate_file_name("a//b").unwrap_err();

    assert_eq!(error.status_code(), 400);
    assert_eq!(
        serde_json::to_string(&error)?,
        r#"{"code":"invalid_name","message":"path has an empty segment"}"#
    );
    Ok(())
}
