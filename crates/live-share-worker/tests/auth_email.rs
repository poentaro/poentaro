//! Unit tests for `auth::email::normalize_email`: what counts as an email,
//! and the one form it's stored and looked up in.
#![allow(clippy::disallowed_macros)]

use live_share_worker::auth::email::{normalize_email, MAX_EMAIL_LENGTH};

#[test]
fn trims_and_lowercases_so_case_never_splits_an_account() {
    assert_eq!(
        normalize_email("  Ann.Lee@Example.COM "),
        Some("ann.lee@example.com".to_string())
    );
}

#[test]
fn accepts_plus_addresses_and_subdomains() {
    assert_eq!(
        normalize_email("ann+poentaro@mail.example.co.uk"),
        Some("ann+poentaro@mail.example.co.uk".to_string())
    );
}

#[test]
fn rejects_what_cannot_be_an_address() {
    let malformed = [
        "",
        "ann",
        "ann@",
        "@example.com",
        "ann@example",
        "ann@@example.com",
        "ann@exa@mple.com",
        "ann@example..com",
        "ann@.example.com",
        "ann lee@example.com",
    ];

    for email in malformed {
        assert_eq!(normalize_email(email), None, "{email:?}");
    }
}

#[test]
fn rejects_an_address_longer_than_smtp_allows() {
    let domain = "@example.com";
    let fits = format!("{}{domain}", "a".repeat(MAX_EMAIL_LENGTH - domain.len()));
    let too_long = format!("a{fits}");

    assert_eq!(normalize_email(&fits), Some(fits.clone()));
    assert_eq!(normalize_email(&too_long), None);
}
