use crate::folder_share_payload::{decode_folder_snapshot, encode_folder_snapshot};
use poentaro::file_tree::{FolderSnapshot, SnapshotFile};

fn snapshot(file_count: usize, content: &str) -> FolderSnapshot {
    FolderSnapshot {
        folder: "Hymns".to_string(),
        files: (0..file_count)
            .map(|n| SnapshotFile {
                path: format!("Advent/{n}.poentaro"),
                content: content.to_string(),
            })
            .collect(),
        empty_folders: vec!["Empty".to_string()],
    }
}

#[test]
fn a_folder_snapshot_round_trips_through_the_link_suffix() {
    let original = snapshot(
        3,
        r#"metadata { title "O Come" },

score """
    1 2 3 4
    """,
"#,
    );
    let suffix = encode_folder_snapshot(&original).unwrap();
    assert!(suffix
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    assert_eq!(decode_folder_snapshot(&suffix), Some(original));
}

#[test]
fn a_snapshot_that_compresses_past_the_limit_has_no_link() {
    // Distinct numbers in every file, so the snapshot does not compress away.
    let mut big = snapshot(0, "");
    big.files = (0..40)
        .map(|file| SnapshotFile {
            path: format!("{file}.poentaro"),
            content: (0..120)
                .map(|n| format!("{} ", (n * 7919 + file * 104_729 + 13) % 1_299_709))
                .collect(),
        })
        .collect();
    assert_eq!(encode_folder_snapshot(&big), None);
}

#[test]
fn an_enormous_snapshot_is_refused_without_compressing() {
    assert_eq!(
        encode_folder_snapshot(&snapshot(1, &"x".repeat(300_000))),
        None
    );
}

#[test]
fn garbage_and_single_file_payloads_do_not_decode_as_folders() {
    assert_eq!(decode_folder_snapshot("not base64!"), None);
    assert_eq!(decode_folder_snapshot("AAAA"), None);
    let single = crate::compress_share_payload_bytes(r#"{"filename":"a.poentaro","content":"x"}"#);
    assert_eq!(
        decode_folder_snapshot(&poentaro::file_tree::base64url_encode(&single)),
        None
    );
}
