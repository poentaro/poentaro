//! Wire-shape tests for `protocol`, ported from the old
//! `live-share-worker/test/protocol.test.ts`.
#![allow(clippy::disallowed_macros)]

use live_share_worker::model::{SessionResponse, SyncedDoc};

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
