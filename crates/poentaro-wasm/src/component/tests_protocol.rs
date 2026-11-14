use super::guest_protocol::{protocol_decode_response, protocol_encode_request};
use super::Protocol;

#[test]
fn a_call_is_written_as_the_protocol_text() {
    let request =
        protocol_encode_request(Protocol::App, "files/trash _".into(), r#"["abc"]"#.into());
    assert_eq!(request, Ok("files/trash \"abc\"\n".to_string()));
}

#[test]
fn a_method_that_returns_nothing_reads_as_a_value() {
    let response = protocol_decode_response(
        Protocol::App,
        "files/trash _".into(),
        "files/trash \"abc\" RETURNS NOTHING".into(),
    );
    assert_eq!(response, Ok(r#"{"kind":"value"}"#.to_string()));
}

#[test]
fn an_error_reads_as_its_kind() {
    let response = protocol_decode_response(
        Protocol::Public,
        "migrations/claim _".into(),
        "migrations/claim \"abc\" ERROR missing".into(),
    );
    assert_eq!(
        response,
        Ok(r#"{"error":{"kind":"missing"},"kind":"error"}"#.to_string())
    );
}

#[test]
fn a_method_of_another_protocol_is_refused() {
    let request = protocol_encode_request(Protocol::Public, "files/list".into(), "[]".into());
    assert!(request.is_err());
}
