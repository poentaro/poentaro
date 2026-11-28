#![allow(clippy::disallowed_macros)]
use super::Protocol;

// `app` and `public` became `api`, an anyx core schema whose generated
// clients speak JSON themselves; only `test` still goes through this codec.

#[test]
fn the_codec_refuses_the_protocols_api_replaced() {
    for protocol in [Protocol::App, Protocol::Public] {
        let refused = super::guest_protocol::protocol_encode_request(protocol, "[]".to_string());
        assert!(refused.is_err(), "{protocol:?} should be refused");
    }
}
