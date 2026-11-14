//! The codec the web app's generated anyx clients call: it writes a call as
//! the protocol's text and reads the response back as JSON, against the
//! schemas in `crates/poentaro-protocol`. Each function takes the owned
//! strings wit-bindgen hands `Guest`, so `needless_pass_by_value` is
//! relaxed here.
#![allow(clippy::needless_pass_by_value)]

use super::Protocol;

fn schema(protocol: Protocol) -> &'static anyx_protocol::Schema {
    match protocol {
        Protocol::App => poentaro_protocol::app::schema(),
        Protocol::Public => poentaro_protocol::public::schema(),
        Protocol::Test => poentaro_protocol::test::schema(),
    }
}

pub(super) fn protocol_encode_request(
    protocol: Protocol,
    method: String,
    arguments: String,
) -> Result<String, String> {
    anyx_protocol::json::encode_request_json(schema(protocol), &method, &arguments)
}

pub(super) fn protocol_decode_response(
    protocol: Protocol,
    method: String,
    response: String,
) -> Result<String, String> {
    anyx_protocol::json::decode_response_json(schema(protocol), &method, &response)
}
