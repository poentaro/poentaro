use super::*;

fn path(raw: &str) -> TreePath {
    TreePath::parse(raw).unwrap()
}

fn store(files: &[(&str, &str)], folders: &[&str]) -> FileStoreState {
    FileStoreState {
        active: String::new(),
        user_files: files
            .iter()
            .map(|(name, content)| (name.to_string(), content.to_string()))
            .collect(),
        bin: Default::default(),
        file_ids: Default::default(),
        folders: folders.iter().map(|raw| path(raw)).collect(),
    }
}

#[test]
fn a_snapshot_has_relative_paths_contents_and_explicit_empty_folders() {
    let state = store(
        &[
            ("Hymns/Advent/b.poentaro", "B"),
            ("Hymns/a.poentaro", "A"),
            ("Other/c.poentaro", "C"),
        ],
        &["Hymns/Empty", "Hymns/Advent", "Hymns"],
    );
    let snapshot = folder_snapshot(&state, &path("Hymns")).unwrap();
    assert_eq!(snapshot.folder, "Hymns");
    assert_eq!(
        snapshot.files,
        vec![
            SnapshotFile {
                path: "Advent/b.poentaro".into(),
                content: "B".into()
            },
            SnapshotFile {
                path: "a.poentaro".into(),
                content: "A".into()
            },
        ]
    );
    // `Advent` is explicit but holds a file, so it is not an empty folder.
    assert_eq!(snapshot.empty_folders, vec!["Empty".to_string()]);
}

#[test]
fn a_nested_folder_is_named_by_its_own_name_only() {
    let state = store(&[("A/B/x.poentaro", "X")], &[]);
    let snapshot = folder_snapshot(&state, &path("A/B")).unwrap();
    assert_eq!(snapshot.folder, "B");
    assert_eq!(snapshot.files[0].path, "x.poentaro");
}

#[test]
fn a_missing_folder_has_no_snapshot() {
    assert_eq!(
        folder_snapshot(&store(&[("a.poentaro", "")], &[]), &path("Nope")),
        None
    );
}

#[test]
fn json_round_trips_including_unicode() {
    let state = store(&[("詩歌/一.poentaro", "## 你好\n\"q\"")], &["詩歌/空"]);
    let snapshot = folder_snapshot(&state, &path("詩歌")).unwrap();
    assert_eq!(FolderSnapshot::from_json(&snapshot.to_json()), Ok(snapshot));
}

#[test]
fn json_uses_the_documented_field_names() {
    let snapshot = FolderSnapshot {
        folder: "F".into(),
        files: vec![SnapshotFile {
            path: "a.poentaro".into(),
            content: "x".into(),
        }],
        empty_folders: vec!["E".into()],
    };
    assert_eq!(
        snapshot.to_json(),
        r#"{"folder":"F","files":[{"path":"a.poentaro","content":"x"}],"emptyFolders":["E"]}"#
    );
}

#[test]
fn a_single_file_share_payload_is_not_a_folder_snapshot() {
    assert_eq!(
        FolderSnapshot::from_json(r#"{"filename":"a.poentaro","content":"x"}"#),
        Err(SnapshotError::NotJson)
    );
}

#[test]
fn decoding_rejects_bad_folder_names_and_paths() {
    let decode = |json: &str| FolderSnapshot::from_json(json);
    assert_eq!(
        decode(r#"{"folder":"A/B","files":[]}"#),
        Err(SnapshotError::InvalidFolderName)
    );
    assert_eq!(
        decode(r#"{"folder":"..","files":[]}"#),
        Err(SnapshotError::InvalidFolderName)
    );
    assert_eq!(
        decode(r#"{"folder":"A","files":[{"path":"../x","content":""}]}"#),
        Err(SnapshotError::InvalidPath {
            path: "../x".into()
        })
    );
    assert_eq!(
        decode(r#"{"folder":"A","files":[{"path":"","content":""}]}"#),
        Err(SnapshotError::InvalidPath {
            path: String::new()
        })
    );
}

#[test]
fn decoding_enforces_the_path_byte_limit_on_the_whole_path() {
    let long = "界".repeat(340); // 1020 bytes; with `A/` it is 1022
    let ok = format!(r#"{{"folder":"A","files":[{{"path":"{long}","content":""}}]}}"#);
    assert!(FolderSnapshot::from_json(&ok).is_ok());
    let too_long = format!(r#"{{"folder":"A","files":[{{"path":"{long}界","content":""}}]}}"#);
    assert!(matches!(
        FolderSnapshot::from_json(&too_long),
        Err(SnapshotError::InvalidPath { .. })
    ));
}

#[test]
fn decoding_rejects_a_path_that_is_both_file_and_folder() {
    let json = r#"{"folder":"A","files":[{"path":"x","content":""},{"path":"x/y","content":""}]}"#;
    assert!(matches!(
        FolderSnapshot::from_json(json),
        Err(SnapshotError::Conflict { .. })
    ));
    let duplicate =
        r#"{"folder":"A","files":[{"path":"x","content":""},{"path":"x","content":""}]}"#;
    assert!(matches!(
        FolderSnapshot::from_json(duplicate),
        Err(SnapshotError::Conflict { .. })
    ));
}

#[test]
fn base64url_round_trips_and_rejects_other_alphabets() {
    let bytes = [0u8, 255, 62, 63, 1];
    let text = base64url_encode(&bytes);
    assert!(!text.contains(['+', '/', '=']));
    assert_eq!(base64url_decode(&text), Some(bytes.to_vec()));
    assert_eq!(base64url_decode("a+b/"), None);
}
