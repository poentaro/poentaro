//! Wire-shape tests for `protocol`, ported from the old
//! `live-share-worker/test/protocol.test.ts`.
#![allow(clippy::disallowed_macros)]

use live_share_worker::protocol::{
    ChangePasswordRequest, FileShareRequest, FileShareResponse, SessionResponse, ShareStatus,
    ShareStatusResponse, SyncedDoc,
};

#[test]
fn serializes_synced_doc_with_camel_case_fields() -> Result<(), serde_json::Error> {
    let doc = SyncedDoc {
        ended: false,
        filename: "song.poentaro".to_string(),
        content: "[M] 1".to_string(),
        revision: 1,
        folder: None,
    };

    let json = serde_json::to_string(&doc)?;

    assert!(json.contains("\"filename\":\"song.poentaro\""));
    assert!(json.contains("\"revision\":1"));
    assert!(!json.contains("owner"));
    Ok(())
}

#[test]
fn reads_a_change_password_request_in_camel_case() -> Result<(), serde_json::Error> {
    let request: ChangePasswordRequest = serde_json::from_str(
        r#"{"identityToken":"t","currentPassword":"old password","newPassword":"new password"}"#,
    )?;

    assert_eq!(request.identity_token, "t");
    assert_eq!(request.current_password, "old password");
    assert_eq!(request.new_password, "new password");
    Ok(())
}

#[test]
fn writes_a_session_response_in_camel_case() -> Result<(), serde_json::Error> {
    let response = SessionResponse {
        session_token: "t".to_string(),
        email: "a@example.com".to_string(),
    };

    assert_eq!(
        serde_json::to_string(&response)?,
        r#"{"sessionToken":"t","email":"a@example.com"}"#
    );
    Ok(())
}

#[test]
fn deserializes_a_file_share_request_from_camel_case_wire_fields() -> Result<(), serde_json::Error>
{
    let json = r#"{"identityToken":"tok-a"}"#;

    let request: FileShareRequest = serde_json::from_str(json)?;

    assert_eq!(request.identity_token, "tok-a");
    Ok(())
}

#[test]
fn serializes_a_file_share_response_with_camel_case_fields() -> Result<(), serde_json::Error> {
    let response = FileShareResponse {
        share_id: "abcdefghijk".to_string(),
    };

    assert_eq!(
        serde_json::to_string(&response)?,
        r#"{"shareId":"abcdefghijk"}"#
    );
    Ok(())
}

#[test]
fn serializes_a_share_status_response_with_a_share() -> Result<(), serde_json::Error> {
    let response = ShareStatusResponse {
        share: Some(ShareStatus {
            share_id: "abcdefghijk".to_string(),
            ended: true,
        }),
    };

    assert_eq!(
        serde_json::to_string(&response)?,
        r#"{"share":{"shareId":"abcdefghijk","ended":true}}"#
    );
    Ok(())
}

#[test]
fn serializes_a_share_status_response_without_a_share_as_null() -> Result<(), serde_json::Error> {
    let response = ShareStatusResponse { share: None };

    assert_eq!(serde_json::to_string(&response)?, r#"{"share":null}"#);
    Ok(())
}
