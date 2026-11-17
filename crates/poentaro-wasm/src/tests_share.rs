use super::*;
use crate::{compress_share_payload_bytes, decompress_share_payload_bytes};

#[test]
fn share_payload_round_trips_through_brotli() {
    let demo_sources: Vec<String> = demo_file_paths()
        .iter()
        .map(|p| read_demo_file(p))
        .collect();
    let fixtures: Vec<&str> = demo_sources
        .iter()
        .map(String::as_str)
        .chain([
            include_str!("../../../simple.poentaro"),
            include_str!("../../../fixtures/follow_and_key.poentaro"),
        ])
        .collect();
    for fixture in fixtures {
        let compressed = compress_share_payload_bytes(fixture);
        let decompressed = decompress_share_payload_bytes(&compressed);
        assert_eq!(decompressed.as_deref(), Some(fixture));
    }
}

#[test]
fn decompress_share_payload_rejects_garbage() {
    assert_eq!(decompress_share_payload_bytes(b"not brotli"), None);
}
