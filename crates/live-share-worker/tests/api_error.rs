//! Wire-shape and status tests for `api_error::ApiError`, the one failure
//! body every route answers with.
#![allow(clippy::disallowed_macros)]

use live_share_worker::api_error::{ApiError, ApiErrorBody};
use live_share_worker::auth::session::VerificationFailure;

#[test]
fn serializes_a_unit_variant_as_just_its_code() -> Result<(), serde_json::Error> {
    assert_eq!(
        serde_json::to_string(&ApiError::NameTaken)?,
        r#"{"code":"name_taken"}"#
    );
    Ok(())
}

#[test]
fn serializes_a_revision_conflict_with_a_camel_case_revision() -> Result<(), serde_json::Error> {
    let error = ApiError::RevisionConflict {
        current_revision: 7,
    };

    assert_eq!(
        serde_json::to_string(&error)?,
        r#"{"code":"revision_conflict","currentRevision":7}"#
    );
    Ok(())
}

#[test]
fn flattens_a_verification_failure_next_to_its_code() -> Result<(), serde_json::Error> {
    let error = ApiError::Unauthorized(VerificationFailure {
        reason: "GitHub said no".to_string(),
        failed_at: 1,
        attempts: 3,
    });

    assert_eq!(
        serde_json::to_string(&error)?,
        r#"{"code":"unauthorized","reason":"GitHub said no","failedAt":1,"attempts":3}"#
    );
    Ok(())
}

struct StatusCase {
    error: ApiError,
    status: u16,
}

#[test]
fn derives_each_status_code_from_the_variant() {
    let failure = VerificationFailure {
        reason: "boom".to_string(),
        failed_at: 0,
        attempts: 1,
    };
    let cases = [
        StatusCase {
            error: ApiError::BadRequest,
            status: 400,
        },
        StatusCase {
            error: ApiError::Unauthorized(failure),
            status: 401,
        },
        StatusCase {
            error: ApiError::NotFound,
            status: 404,
        },
        StatusCase {
            error: ApiError::NameTaken,
            status: 409,
        },
        StatusCase {
            error: ApiError::RevisionConflict {
                current_revision: 1,
            },
            status: 409,
        },
        StatusCase {
            error: ApiError::Internal {
                message: "boom".to_string(),
            },
            status: 500,
        },
        StatusCase {
            error: ApiError::EmailTaken,
            status: 409,
        },
        StatusCase {
            error: ApiError::InvalidEmail,
            status: 400,
        },
        StatusCase {
            error: ApiError::WeakPassword {
                message: "Use at least 12 characters.".to_string(),
            },
            status: 400,
        },
        StatusCase {
            error: ApiError::IncorrectCredentials,
            status: 401,
        },
        StatusCase {
            error: ApiError::SignInLocked {
                retry_after_seconds: 60,
            },
            status: 429,
        },
        StatusCase {
            error: ApiError::RateLimited,
            status: 429,
        },
    ];

    for case in cases {
        assert_eq!(case.error.status_code(), case.status, "{:?}", case.error);
    }
}

#[test]
fn serializes_names_taken_with_the_rejected_destinations() -> Result<(), serde_json::Error> {
    let error = ApiError::NamesTaken {
        names: vec!["A/x".to_string()],
    };

    assert_eq!(
        serde_json::to_string(&error)?,
        r#"{"code":"names_taken","names":["A/x"]}"#
    );
    assert_eq!(error.status_code(), 409);
    Ok(())
}

#[test]
fn serializes_a_sign_in_lockout_with_a_camel_case_retry_after() -> Result<(), serde_json::Error> {
    let error = ApiError::SignInLocked {
        retry_after_seconds: 900,
    };

    assert_eq!(
        serde_json::to_string(&error)?,
        r#"{"code":"sign_in_locked","retryAfterSeconds":900}"#
    );
    Ok(())
}

#[test]
fn a_failure_body_carries_its_trace_id_next_to_the_code() -> Result<(), serde_json::Error> {
    let body = ApiErrorBody {
        error: ApiError::RevisionConflict {
            current_revision: 7,
        },
        trace_id: "0123456789abcdef".to_string(),
    };

    assert_eq!(
        serde_json::to_string(&body)?,
        r#"{"code":"revision_conflict","currentRevision":7,"traceId":"0123456789abcdef"}"#
    );
    Ok(())
}

#[test]
fn code_is_the_serialized_tag() {
    assert_eq!(ApiError::NamesTaken { names: vec![] }.code(), "names_taken");
}

#[test]
fn message_is_the_free_text_of_the_variants_that_have_one() {
    let internal = ApiError::Internal {
        message: "D1_ERROR".to_string(),
    };

    assert_eq!(internal.message(), Some("D1_ERROR"));
    assert_eq!(ApiError::NotFound.message(), None);
}
