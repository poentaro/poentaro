use super::guest_protocol::{protocol_decode_response, protocol_encode_request};
use super::Protocol;

#[test]
fn a_call_is_written_as_the_protocol_text() {
    let request = protocol_encode_request(
        Protocol::App,
        r#"[{"method":"trash file _","arguments":["abc"]}]"#.into(),
    );
    assert_eq!(request, Ok("trash file \"abc\"\n".to_string()));
}

#[test]
fn several_calls_are_written_as_one_request() {
    let request = protocol_encode_request(
        Protocol::App,
        r#"[{"method":"trash file _","arguments":["a"]},{"method":"trash file _","arguments":["b"]}]"#
            .into(),
    );
    assert_eq!(
        request,
        Ok("trash file \"a\",\ntrash file \"b\",\n".to_string())
    );
}

#[test]
fn a_method_that_returns_nothing_reads_as_a_value() {
    let response = protocol_decode_response(
        Protocol::App,
        r#"["trash file _"]"#.into(),
        "RETURNS NOTHING".into(),
    );
    assert_eq!(response, Ok(r#"[{"kind":"value"}]"#.to_string()));
}

#[test]
fn an_error_reads_as_its_kind() {
    let response = protocol_decode_response(
        Protocol::Public,
        r#"["claim origin handoff _"]"#.into(),
        "ERROR missing".into(),
    );
    assert_eq!(
        response,
        Ok(r#"[{"error":{"kind":"missing"},"kind":"error"}]"#.to_string())
    );
}

#[test]
fn a_method_of_another_protocol_is_refused() {
    let request = protocol_encode_request(
        Protocol::Public,
        r#"[{"method":"list files","arguments":[]}]"#.into(),
    );
    assert!(request.is_err());
}

#[test]
fn named_ids_are_annotated_by_the_codec() {
    let request = protocol_encode_request(
        Protocol::App,
        r#"[{"method":"revoke agent key _","arguments":["key-a"]}]"#.into(),
    )
    .unwrap();
    assert!(request.contains(r#""key-a" AS"#));
    assert!(request.contains("Key Id"));
    let decoded = super::guest_protocol::protocol_decode_request(Protocol::App, request).unwrap();
    assert_eq!(
        decoded,
        r#"[{"arguments":["key-a"],"method":"revoke agent key _"}]"#
    );
}

#[test]
fn lockout_lengths_are_read_as_milliseconds() {
    let response = protocol_decode_response(
        Protocol::Public,
        r#"["sign in _ password _"]"#.into(),
        "ERROR locked for (1 minute 30 seconds)".into(),
    );
    assert_eq!(
        response,
        Ok(r#"[{"error":{"duration":90000,"kind":"lockedFor"},"kind":"error"}]"#.into())
    );
}

#[test]
fn file_timestamps_are_instants_in_json() {
    let response = protocol_decode_response(
        Protocol::App,
        r#"["list files"]"#.into(),
        r#"RETURNS [{ id "a", path "a.poentaro", revision 3, trashed at "2026-11-14T06:00:08Z" }]"#
            .into(),
    )
    .unwrap();
    assert!(response.contains(r#""trashedAt":"2026-11-14T06:00:08.000Z""#));
}

#[test]
fn batch_results_keep_their_order_without_echoing_passwords() {
    let request = protocol_encode_request(
        Protocol::Public,
        r#"[{"method":"sign in _ password _","arguments":["owner@example.com","secret password"]},{"method":"open shared link _","arguments":["share-a"]}]"#.into(),
    ).unwrap();
    let response = super::guest_protocol::protocol_encode_response(
        Protocol::Public,
        request,
        r#"[{"kind":"error","error":{"kind":"incorrect"}},{"kind":"value","value":{"kind":"ended"}}]"#.into(),
    ).unwrap();
    assert!(!response.contains("secret password"));
    assert!(!response.contains("owner@example.com"));
    let outcomes = protocol_decode_response(
        Protocol::Public,
        r#"["sign in _ password _","open shared link _"]"#.into(),
        response,
    );
    assert_eq!(outcomes, Ok(r#"[{"error":{"kind":"incorrect"},"kind":"error"},{"kind":"value","value":{"kind":"ended"}}]"#.into()));
}
