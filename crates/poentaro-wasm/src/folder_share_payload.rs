//! The static folder share link: a `poentaro::file_tree::FolderSnapshot`
//! as JSON, brotli-compressed (the same compression as the single-file
//! `#share=` link) and base64url-encoded. The part after `#share-folder=`.

use poentaro::file_tree::{
    base64url_decode, base64url_encode, FolderSnapshot, MAX_STATIC_FOLDER_LINK_CHARS,
    MAX_UNCOMPRESSED_SNAPSHOT_BYTES,
};

use crate::{compress_share_payload_bytes, decompress_share_payload_bytes};

/// The link suffix for `snapshot`, or `None` when it would exceed
/// [`MAX_STATIC_FOLDER_LINK_CHARS`] (checked without compressing when the
/// JSON alone is beyond [`MAX_UNCOMPRESSED_SNAPSHOT_BYTES`]).
pub(crate) fn encode_folder_snapshot(snapshot: &FolderSnapshot) -> Option<String> {
    let json = snapshot.to_json();
    if json.len() > MAX_UNCOMPRESSED_SNAPSHOT_BYTES {
        return None;
    }
    let suffix = base64url_encode(&compress_share_payload_bytes(&json));
    (suffix.len() <= MAX_STATIC_FOLDER_LINK_CHARS).then_some(suffix)
}

/// Inverse of [`encode_folder_snapshot`]; `None` for anything that is not a
/// valid folder snapshot (including a single-file share payload).
pub(crate) fn decode_folder_snapshot(suffix: &str) -> Option<FolderSnapshot> {
    let json = decompress_share_payload_bytes(&base64url_decode(suffix)?)?;
    FolderSnapshot::from_json(&json).ok()
}
