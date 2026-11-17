//! The pure side of folder shares: path validation, the size limits, the
//! viewer's tree and which shares a batch rename moves.
#![allow(clippy::disallowed_macros)]

use live_share_worker::api_error::ApiError;
use live_share_worker::folder_share::{
    check_size, plan_prefix_rewrites, to_public_folder_doc, validate_folder_path, ActiveFileName,
    FolderFileRow, FolderShareHeader, FolderStats, PrefixRewrite, MAX_SHARED_FOLDER_BYTES,
    MAX_SHARED_FOLDER_FILES,
};
use live_share_worker::protocol::{RenameBatchMove, SharedFolder, SharedFolderFile, SyncedDoc};
use live_share_worker::share::empty_doc;

fn header(ended_at: Option<i64>) -> FolderShareHeader {
    FolderShareHeader {
        ended_at,
        owner_user_id: "owner".to_string(),
        folder_prefix: "Hymns/Advent".to_string(),
    }
}

fn row(name: &str, content: &str, revision: i64) -> FolderFileRow {
    FolderFileRow {
        name: name.to_string(),
        content: content.to_string(),
        revision,
    }
}

fn shared_file(path: &str, content: &str, revision: i64) -> SharedFolderFile {
    SharedFolderFile {
        path: path.to_string(),
        content: content.to_string(),
        revision,
    }
}

#[test]
fn a_folder_path_follows_the_shared_tree_path_rules() {
    assert!(validate_folder_path("Hymns/Advent").is_ok());
    for bad in ["", "/Hymns", "Hymns/", "Hymns//Advent", "a/../b", "."] {
        assert!(
            matches!(validate_folder_path(bad), Err(ApiError::InvalidName { .. })),
            "{bad:?} should be refused"
        );
    }
    let too_long = "あ".repeat(342);
    assert!(matches!(
        validate_folder_path(&too_long),
        Err(ApiError::InvalidName { .. })
    ));
}

#[test]
fn the_placeholder_name_is_not_a_folder_name() {
    assert!(matches!(
        validate_folder_path("Hymns/.keep"),
        Err(ApiError::InvalidName { .. })
    ));
}

#[test]
fn a_folder_over_either_limit_is_refused() {
    let within = FolderStats {
        file_count: MAX_SHARED_FOLDER_FILES,
        content_bytes: MAX_SHARED_FOLDER_BYTES,
    };
    assert_eq!(check_size(within), Ok(()));
    for over in [
        FolderStats {
            file_count: MAX_SHARED_FOLDER_FILES + 1,
            content_bytes: 0,
        },
        FolderStats {
            file_count: 1,
            content_bytes: MAX_SHARED_FOLDER_BYTES + 1,
        },
    ] {
        assert!(matches!(
            check_size(over),
            Err(ApiError::SharedFolderTooLarge { .. })
        ));
    }
    assert_eq!(
        ApiError::SharedFolderTooLarge {
            message: String::new()
        }
        .status_code(),
        413
    );
}

#[test]
fn the_viewer_tree_uses_paths_relative_to_the_shared_folder() {
    let rows = [
        row("Hymns/Advent/O Come.poentaro", "[M] 1", 2),
        row("Hymns/Advent/Sub/Deep.poentaro", "[M] 2", 0),
    ];

    let doc = to_public_folder_doc(&header(None), &rows);

    assert_eq!(
        doc,
        SyncedDoc {
            ended: false,
            filename: "Advent".to_string(),
            content: String::new(),
            revision: 0,
            folder: Some(SharedFolder {
                name: "Advent".to_string(),
                files: vec![
                    shared_file("O Come.poentaro", "[M] 1", 2),
                    shared_file("Sub/Deep.poentaro", "[M] 2", 0),
                ],
                empty_folders: vec![],
            }),
        }
    );
}

#[test]
fn placeholders_are_not_files_and_mark_only_folders_without_files() {
    let rows = [
        row("Hymns/Advent/.keep", "", 0),
        row("Hymns/Advent/Empty/.keep", "", 0),
        row("Hymns/Advent/Full/.keep", "", 0),
        row("Hymns/Advent/Full/a.poentaro", "x", 1),
        row("Hymns/Advent/Nested/Inner/.keep", "", 0),
    ];

    let folder = to_public_folder_doc(&header(None), &rows).folder.unwrap();

    assert_eq!(folder.files, vec![shared_file("Full/a.poentaro", "x", 1)]);
    assert_eq!(folder.empty_folders, vec!["Empty", "Nested/Inner"]);
}

#[test]
fn a_folder_without_files_is_a_live_empty_tree() {
    let doc = to_public_folder_doc(&header(None), &[]);

    assert!(!doc.ended);
    assert_eq!(
        doc.folder,
        Some(SharedFolder {
            name: "Advent".to_string(),
            files: vec![],
            empty_folders: vec![],
        })
    );
}

#[test]
fn a_stopped_folder_share_hides_everything() {
    let doc = to_public_folder_doc(&header(Some(5)), &[row("Hymns/Advent/a.poentaro", "x", 1)]);

    assert_eq!(doc, empty_doc());
}

#[test]
fn a_file_share_and_an_ended_share_have_no_folder_key_on_the_wire() -> Result<(), serde_json::Error>
{
    let json = serde_json::to_value(empty_doc())?;
    assert!(json.get("folder").is_none());
    Ok(())
}

#[test]
fn the_folder_tree_serializes_in_camel_case_without_internal_ids() -> Result<(), serde_json::Error>
{
    let doc = to_public_folder_doc(&header(None), &[row("Hymns/Advent/a.poentaro", "x", 1)]);

    let json = serde_json::to_value(doc)?;

    assert_eq!(
        json["folder"],
        serde_json::json!({
            "name": "Advent",
            "files": [{"path": "a.poentaro", "content": "x", "revision": 1}],
            "emptyFolders": [],
        })
    );
    assert!(!json.to_string().contains("owner_user_id"));
    assert!(!json.to_string().contains("Hymns/Advent/a"));
    Ok(())
}

fn moves(pairs: &[(&str, &str)]) -> Vec<RenameBatchMove> {
    pairs
        .iter()
        .map(|(id, name)| RenameBatchMove {
            id: (*id).to_string(),
            name: (*name).to_string(),
        })
        .collect()
}

fn current(pairs: &[(&str, &str)]) -> Vec<ActiveFileName> {
    pairs
        .iter()
        .map(|(id, name)| ActiveFileName {
            id: (*id).to_string(),
            name: (*name).to_string(),
        })
        .collect()
}

fn rewrite(from: &str, to: &str) -> PrefixRewrite {
    PrefixRewrite {
        from: from.to_string(),
        to: to.to_string(),
    }
}

fn prefixes(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[test]
fn a_folder_rename_rewrites_the_shared_prefix() {
    let plan = plan_prefix_rewrites(
        &prefixes(&["Hymns"]),
        &moves(&[("a", "Songs/a.poentaro"), ("b", "Songs/Sub/b.poentaro")]),
        &current(&[("a", "Hymns/a.poentaro"), ("b", "Hymns/Sub/b.poentaro")]),
    );

    assert_eq!(plan, vec![rewrite("Hymns", "Songs")]);
}

#[test]
fn a_folder_moved_into_another_folder_rewrites_to_the_new_path() {
    let plan = plan_prefix_rewrites(
        &prefixes(&["Hymns"]),
        &moves(&[("a", "Archive/Old/Hymns/a.poentaro")]),
        &current(&[("a", "Hymns/a.poentaro")]),
    );

    assert_eq!(plan, vec![rewrite("Hymns", "Archive/Old/Hymns")]);
}

#[test]
fn a_nested_shared_folder_moves_with_its_parent_and_has_its_own_rewrite() {
    let plan = plan_prefix_rewrites(
        &prefixes(&["Hymns", "Hymns/Advent"]),
        &moves(&[("a", "Songs/a.poentaro"), ("b", "Songs/Advent/b.poentaro")]),
        &current(&[("a", "Hymns/a.poentaro"), ("b", "Hymns/Advent/b.poentaro")]),
    );

    assert_eq!(
        plan,
        vec![
            rewrite("Hymns", "Songs"),
            rewrite("Hymns/Advent", "Songs/Advent")
        ]
    );
}

#[test]
fn renaming_only_a_shared_subfolder_leaves_the_parent_share_alone() {
    let plan = plan_prefix_rewrites(
        &prefixes(&["Hymns", "Hymns/Advent"]),
        &moves(&[("b", "Hymns/Lent/b.poentaro")]),
        &current(&[("a", "Hymns/a.poentaro"), ("b", "Hymns/Advent/b.poentaro")]),
    );

    assert_eq!(plan, vec![rewrite("Hymns/Advent", "Hymns/Lent")]);
}

#[test]
fn a_folder_sharing_only_a_string_prefix_is_not_the_shared_folder() {
    let plan = plan_prefix_rewrites(
        &prefixes(&["Hymns"]),
        &moves(&[("x", "Songs/x.poentaro")]),
        // `current` is whatever the files query returned; only whole path
        // segments count as inside `Hymns`.
        &current(&[("a", "Hymns/a.poentaro"), ("x", "Hymns 2/x.poentaro")]),
    );

    assert_eq!(plan, vec![]);
}

#[test]
fn moving_some_files_out_does_not_move_the_share() {
    let plan = plan_prefix_rewrites(
        &prefixes(&["Hymns"]),
        &moves(&[("a", "Elsewhere/a.poentaro")]),
        &current(&[("a", "Hymns/a.poentaro"), ("b", "Hymns/b.poentaro")]),
    );

    assert_eq!(plan, vec![]);
}

#[test]
fn moving_every_file_to_different_places_does_not_move_the_share() {
    let plan = plan_prefix_rewrites(
        &prefixes(&["Hymns"]),
        &moves(&[("a", "One/a.poentaro"), ("b", "Two/b.poentaro")]),
        &current(&[("a", "Hymns/a.poentaro"), ("b", "Hymns/b.poentaro")]),
    );

    assert_eq!(plan, vec![]);
}

#[test]
fn renaming_a_file_inside_the_folder_does_not_move_the_share() {
    let plan = plan_prefix_rewrites(
        &prefixes(&["Hymns"]),
        &moves(&[("a", "Hymns/renamed.poentaro")]),
        &current(&[("a", "Hymns/a.poentaro")]),
    );

    assert_eq!(plan, vec![]);
}

#[test]
fn moving_everything_to_the_top_level_does_not_move_the_share() {
    let plan = plan_prefix_rewrites(
        &prefixes(&["Hymns"]),
        &moves(&[("a", "a.poentaro")]),
        &current(&[("a", "Hymns/a.poentaro")]),
    );

    assert_eq!(plan, vec![]);
}

#[test]
fn a_shared_folder_without_active_files_is_left_alone() {
    let plan = plan_prefix_rewrites(&prefixes(&["Hymns"]), &moves(&[("z", "Q/z")]), &[]);

    assert_eq!(plan, vec![]);
}
