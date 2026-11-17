//! Unit tests for the test-only `POST /e2e/reset` route's allowlist: the
//! var's parser and which emails one reset deletes.
#![allow(clippy::disallowed_macros)]

use live_share_worker::e2e_reset::{emails_to_reset, parse_reset_emails};

fn allowed() -> Vec<String> {
    vec!["a@poentaro.test".to_string(), "b@poentaro.test".to_string()]
}

#[test]
fn parses_a_json_array_of_emails_into_their_normalized_form() {
    assert_eq!(
        parse_reset_emails(r#"["A@Poentaro.test"," b@poentaro.test"]"#),
        Ok(allowed()),
    );
}

#[test]
fn rejects_a_value_that_is_not_a_json_array_of_emails() {
    assert!(parse_reset_emails("a@poentaro.test").is_err());
    assert!(parse_reset_emails(r#"["e2e-test-user"]"#).is_err());
    assert!(parse_reset_emails("[987654321]").is_err());
}

#[test]
fn resets_every_allowed_email_when_none_are_requested() {
    assert_eq!(emails_to_reset(&allowed(), None), Ok(allowed()));
}

#[test]
fn resets_only_the_requested_emails() {
    let requested = ["B@poentaro.test".to_string()];

    assert_eq!(
        emails_to_reset(&allowed(), Some(&requested)),
        Ok(vec!["b@poentaro.test".to_string()]),
    );
}

#[test]
fn refuses_a_requested_email_outside_the_allowlist() {
    let requested = [
        "a@poentaro.test".to_string(),
        "real@example.com".to_string(),
    ];

    assert!(emails_to_reset(&allowed(), Some(&requested)).is_err());
}
