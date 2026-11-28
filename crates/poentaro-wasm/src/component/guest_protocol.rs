//! The codec the web app's generated anyx clients call: it writes calls as
//! the protocol's text and reads the response back as JSON, against the
//! schemas in `crates/poentaro-protocol`. Each function takes the owned
//! strings wit-bindgen hands `Guest`, so `needless_pass_by_value` is
//! relaxed here.
#![allow(clippy::needless_pass_by_value)]

use super::Protocol;

/// `app` and `public` became `api`, whose generated client speaks JSON
/// itself, so only `test` still goes through this codec.
fn schema(protocol: Protocol) -> Result<&'static anyx_protocol::Schema, String> {
    match protocol {
        Protocol::App | Protocol::Public => {
            Err("`app` and `public` are served by `api` now".to_string())
        }
        Protocol::Test => Ok(poentaro_protocol::test::schema()),
    }
}

pub(super) fn protocol_encode_request(protocol: Protocol, calls: String) -> Result<String, String> {
    anyx_protocol::json::encode_requests_json(schema(protocol)?, &calls)
}

pub(super) fn protocol_decode_request(
    protocol: Protocol,
    request: String,
) -> Result<String, String> {
    anyx_protocol::json::decode_request_json(schema(protocol)?, &request)
}

pub(super) fn protocol_encode_response(
    protocol: Protocol,
    request: String,
    outcomes: String,
) -> Result<String, String> {
    anyx_protocol::json::encode_response_json(schema(protocol)?, &request, &outcomes)
}

pub(super) fn protocol_decode_response(
    protocol: Protocol,
    methods: String,
    response: String,
) -> Result<String, String> {
    anyx_protocol::json::decode_responses_json(schema(protocol)?, &methods, &response)
}
