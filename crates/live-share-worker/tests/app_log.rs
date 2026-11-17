//! Row-shape tests for `app_log`: what each event writes into the `flow` /
//! `kind` / `outcome` / `subject` / `detail` columns, and that rows decode
//! back -- including rows written before a field was added.
#![allow(clippy::disallowed_macros)]

use live_share_worker::app_log::{
    prune_cutoff_millis, token_tag, AuthEvent, LockedOut, LogEvent, LogRow, Outcome,
    PasswordChanged, RequestEvent, RequestFailed, SessionRejected, SignInFailed,
    SignInFailureReason, SignedIn, SignedOut, SignedUp, TokenExchangeFailed, TokenIssued,
    TokenRejected, TokenRevokeFailed, TokenRevoked, TokenVerified, VerificationUnavailable,
    APP_LOG_RETENTION_MILLIS,
};
use serde_json::{json, Value};

const TAG: &str = "3fa9c1d20b7e";

struct ExpectedRow {
    kind: &'static str,
    outcome: Outcome,
    subject: Option<&'static str>,
    detail: Value,
}

struct Case {
    event: AuthEvent,
    expected: ExpectedRow,
}

const EMAIL: &str = "ann@example.com";

fn every_auth_event() -> Vec<Case> {
    session_start_cases()
        .into_iter()
        .chain(session_end_cases())
        .chain(issuance_and_verification_cases())
        .chain(rejection_and_revocation_cases())
        .collect()
}

fn session_start_cases() -> Vec<Case> {
    vec![
        Case {
            event: AuthEvent::SignedUp(SignedUp {
                email: EMAIL.to_string(),
                token_tag: TAG.to_string(),
            }),
            expected: ExpectedRow {
                kind: "signed_up",
                outcome: Outcome::Ok,
                subject: Some("email:ann@example.com"),
                detail: json!({ "email": EMAIL, "token_tag": TAG }),
            },
        },
        Case {
            event: AuthEvent::SignedIn(SignedIn {
                email: EMAIL.to_string(),
                token_tag: TAG.to_string(),
            }),
            expected: ExpectedRow {
                kind: "signed_in",
                outcome: Outcome::Ok,
                subject: Some("email:ann@example.com"),
                detail: json!({ "email": EMAIL, "token_tag": TAG }),
            },
        },
        Case {
            event: AuthEvent::SignInFailed(SignInFailed {
                email: EMAIL.to_string(),
                reason: SignInFailureReason::WrongPassword,
                failed_count: 2,
            }),
            expected: ExpectedRow {
                kind: "sign_in_failed",
                outcome: Outcome::Rejected,
                subject: Some("email:ann@example.com"),
                detail: json!({ "email": EMAIL, "reason": "wrong_password", "failed_count": 2 }),
            },
        },
        Case {
            event: AuthEvent::LockedOut(LockedOut {
                email: EMAIL.to_string(),
                locked_until: 1_760_000_900_000,
                lockout_level: 1,
            }),
            expected: ExpectedRow {
                kind: "locked_out",
                outcome: Outcome::Rejected,
                subject: Some("email:ann@example.com"),
                detail: json!({ "email": EMAIL, "locked_until": 1_760_000_900_000_i64, "lockout_level": 1 }),
            },
        },
    ]
}

fn session_end_cases() -> Vec<Case> {
    vec![
        Case {
            event: AuthEvent::SignedOut(SignedOut {
                token_tag: TAG.to_string(),
            }),
            expected: ExpectedRow {
                kind: "signed_out",
                outcome: Outcome::Ok,
                subject: Some("token:3fa9c1d20b7e"),
                detail: json!({ "token_tag": TAG }),
            },
        },
        Case {
            event: AuthEvent::PasswordChanged(PasswordChanged {
                email: EMAIL.to_string(),
                token_tag: TAG.to_string(),
                ended_sessions: 2,
            }),
            expected: ExpectedRow {
                kind: "password_changed",
                outcome: Outcome::Ok,
                subject: Some("email:ann@example.com"),
                detail: json!({ "email": EMAIL, "token_tag": TAG, "ended_sessions": 2 }),
            },
        },
        Case {
            event: AuthEvent::SessionRejected(SessionRejected {
                token_tag: TAG.to_string(),
            }),
            expected: ExpectedRow {
                kind: "session_rejected",
                outcome: Outcome::Rejected,
                subject: Some("token:3fa9c1d20b7e"),
                detail: json!({ "token_tag": TAG }),
            },
        },
    ]
}

fn issuance_and_verification_cases() -> Vec<Case> {
    vec![
        Case {
            event: AuthEvent::TokenIssued(TokenIssued {
                token_tag: TAG.to_string(),
                provider_user_id: Some("583231".to_string()),
                login: Some("octocat".to_string()),
            }),
            expected: ExpectedRow {
                kind: "token_issued",
                outcome: Outcome::Ok,
                subject: Some("token:3fa9c1d20b7e"),
                detail: json!({
                    "token_tag": TAG,
                    "provider_user_id": "583231",
                    "login": "octocat",
                }),
            },
        },
        Case {
            event: AuthEvent::TokenExchangeFailed(TokenExchangeFailed {
                reason: "bad_verification_code".to_string(),
            }),
            expected: ExpectedRow {
                kind: "token_exchange_failed",
                outcome: Outcome::Error,
                subject: None,
                detail: json!({ "reason": "bad_verification_code" }),
            },
        },
        Case {
            event: AuthEvent::TokenVerified(TokenVerified {
                token_tag: TAG.to_string(),
                provider_user_id: "583231".to_string(),
            }),
            expected: ExpectedRow {
                kind: "token_verified",
                outcome: Outcome::Ok,
                subject: Some("token:3fa9c1d20b7e"),
                detail: json!({ "token_tag": TAG, "provider_user_id": "583231" }),
            },
        },
    ]
}

fn rejection_and_revocation_cases() -> Vec<Case> {
    vec![
        Case {
            event: AuthEvent::TokenRejected(TokenRejected {
                token_tag: TAG.to_string(),
                reason: "GitHub GET /user failed: status=401, body=Bad credentials".to_string(),
                attempts: 3,
            }),
            expected: ExpectedRow {
                kind: "token_rejected",
                outcome: Outcome::Rejected,
                subject: Some("token:3fa9c1d20b7e"),
                detail: json!({
                    "token_tag": TAG,
                    "reason": "GitHub GET /user failed: status=401, body=Bad credentials",
                    "attempts": 3,
                }),
            },
        },
        Case {
            event: AuthEvent::VerificationUnavailable(VerificationUnavailable {
                token_tag: TAG.to_string(),
                reason: "status=503".to_string(),
                attempts: 3,
            }),
            expected: ExpectedRow {
                kind: "verification_unavailable",
                outcome: Outcome::Unavailable,
                subject: Some("token:3fa9c1d20b7e"),
                detail: json!({ "token_tag": TAG, "reason": "status=503", "attempts": 3 }),
            },
        },
        Case {
            event: AuthEvent::TokenRevoked(TokenRevoked {
                token_tag: TAG.to_string(),
            }),
            expected: ExpectedRow {
                kind: "token_revoked",
                outcome: Outcome::Ok,
                subject: Some("token:3fa9c1d20b7e"),
                detail: json!({ "token_tag": TAG }),
            },
        },
        Case {
            event: AuthEvent::TokenRevokeFailed(TokenRevokeFailed {
                token_tag: TAG.to_string(),
                reason: "status=404".to_string(),
            }),
            expected: ExpectedRow {
                kind: "token_revoke_failed",
                outcome: Outcome::Error,
                subject: Some("token:3fa9c1d20b7e"),
                detail: json!({ "token_tag": TAG, "reason": "status=404" }),
            },
        },
    ]
}

#[test]
fn each_auth_event_splits_into_its_columns() -> Result<(), serde_json::Error> {
    for Case { event, expected } in every_auth_event() {
        let row = LogEvent::Auth(event).to_row()?;

        assert_eq!(row.flow, "auth");
        assert_eq!(row.kind, expected.kind);
        assert_eq!(row.outcome, expected.outcome, "{}", expected.kind);
        assert_eq!(
            row.subject.as_deref(),
            expected.subject,
            "{}",
            expected.kind
        );
        assert_eq!(
            serde_json::from_str::<Value>(&row.detail_json)?,
            expected.detail,
            "{}",
            expected.kind
        );
    }
    Ok(())
}

#[test]
fn each_auth_event_round_trips_through_its_row() -> Result<(), serde_json::Error> {
    for Case { event, .. } in every_auth_event() {
        let event = LogEvent::Auth(event);
        let row = event.to_row()?;

        assert_eq!(LogEvent::try_from(&row)?, event);
    }
    Ok(())
}

#[test]
fn outcome_column_values_are_snake_case() {
    let values = [
        Outcome::Ok,
        Outcome::Rejected,
        Outcome::Unavailable,
        Outcome::Error,
    ]
    .map(Outcome::as_str);

    assert_eq!(values, ["ok", "rejected", "unavailable", "error"]);
}

#[test]
fn a_row_missing_a_defaulted_field_still_decodes() -> Result<(), serde_json::Error> {
    let row = LogRow {
        flow: "auth".to_string(),
        kind: "token_issued".to_string(),
        outcome: Outcome::Ok,
        subject: Some("token:3fa9c1d20b7e".to_string()),
        detail_json: json!({ "token_tag": TAG }).to_string(),
    };

    assert_eq!(
        LogEvent::try_from(&row)?,
        LogEvent::Auth(AuthEvent::TokenIssued(TokenIssued {
            token_tag: TAG.to_string(),
            provider_user_id: None,
            login: None,
        }))
    );
    Ok(())
}

#[test]
fn a_row_of_an_unknown_kind_fails_to_decode() {
    let row = LogRow {
        flow: "auth".to_string(),
        kind: "token_teleported".to_string(),
        outcome: Outcome::Ok,
        subject: None,
        detail_json: "{}".to_string(),
    };

    assert!(LogEvent::try_from(&row).is_err());
}

#[test]
fn token_tag_is_the_first_twelve_chars_of_the_hash() {
    let hash = "3fa9c1d20b7e4d5f8a6b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f";

    assert_eq!(token_tag(hash), "3fa9c1d20b7e");
}

#[test]
fn token_tag_of_a_short_hash_is_the_whole_hash() {
    assert_eq!(token_tag("3fa9"), "3fa9");
}

#[test]
fn retention_is_thirty_days() {
    assert_eq!(APP_LOG_RETENTION_MILLIS, 30 * 24 * 60 * 60 * 1000);
}

#[test]
fn prune_cutoff_is_retention_before_now() {
    let now = 1_760_000_000_000;

    assert_eq!(prune_cutoff_millis(now), now - 2_592_000_000);
}

fn request_failed(status: u16, code: &str, message: Option<&str>) -> LogEvent {
    LogEvent::Request(RequestEvent::RequestFailed(RequestFailed {
        route: "POST /files/{id}/content".to_string(),
        status,
        code: code.to_string(),
        message: message.map(str::to_owned),
    }))
}

#[test]
fn a_failed_request_is_a_request_failed_row() -> Result<(), serde_json::Error> {
    let row = request_failed(500, "internal", Some("D1_ERROR: no such table")).to_row()?;

    assert_eq!(
        row,
        LogRow {
            flow: "request".to_string(),
            kind: "request_failed".to_string(),
            outcome: Outcome::Error,
            subject: None,
            detail_json: json!({
                "route": "POST /files/{id}/content",
                "status": 500,
                "code": "internal",
                "message": "D1_ERROR: no such table",
            })
            .to_string(),
        }
    );
    Ok(())
}

#[test]
fn a_refused_request_is_rejected_not_an_error() -> Result<(), serde_json::Error> {
    let row = request_failed(409, "revision_conflict", None).to_row()?;

    assert_eq!(row.outcome, Outcome::Rejected);
    Ok(())
}

#[test]
fn a_request_failed_row_decodes_back() -> Result<(), serde_json::Error> {
    let event = request_failed(404, "not_found", None);

    assert_eq!(LogEvent::try_from(&event.to_row()?)?, event);
    Ok(())
}
