//! Where the server puts a restored file, and the moves a folder move makes.
//! A folder move is checked against `poentaro::file_tree::move_folder`, the
//! rules the web app applied before the server took over.
#![allow(clippy::disallowed_macros, clippy::unwrap_used)]

use std::collections::BTreeSet;

use live_share_worker::files::FileNameRow;
use live_share_worker::naming::{plan_folder_move, restored_path, FileSummaryRow, FolderMovePlan};
use poentaro::file_tree::{move_folder, FileOperationContext, FileStoreState, TreePath};
use proptest::prelude::*;

fn file(id: &str, name: &str, trashed: bool) -> FileSummaryRow {
    FileSummaryRow {
        id: id.to_string(),
        name: name.to_string(),
        revision: 0,
        trashed_at: trashed.then_some(1),
    }
}

fn live(names: &[&str]) -> Vec<FileNameRow> {
    names
        .iter()
        .enumerate()
        .map(|(index, name)| FileNameRow {
            id: format!("id-{index}"),
            name: (*name).to_string(),
        })
        .collect()
}

fn path(raw: &str) -> TreePath {
    TreePath::parse(raw).unwrap()
}

#[test]
fn a_restored_file_keeps_its_path_when_it_is_free() {
    let files = [file("a", "Hymns/hymn.poentaro", true)];
    let restored = restored_path("a", "Hymns/hymn.poentaro", &files).unwrap();
    assert_eq!(restored.as_str(), "Hymns/hymn.poentaro");
}

#[test]
fn a_restored_file_takes_the_next_free_number_when_its_path_was_taken() {
    let files = [
        file("a", "Hymns/hymn.poentaro", true),
        file("b", "Hymns/hymn.poentaro", false),
        file("c", "Hymns/hymn 2.poentaro", true),
    ];
    let restored = restored_path("a", "Hymns/hymn.poentaro", &files).unwrap();
    assert_eq!(restored.as_str(), "Hymns/hymn 3.poentaro");
}

#[test]
fn a_restored_file_steps_around_a_folder_of_the_same_name() {
    let files = [
        file("a", "Hymns.poentaro", true),
        file("b", "Hymns.poentaro/inner.poentaro", false),
    ];
    let restored = restored_path("a", "Hymns.poentaro", &files).unwrap();
    assert_eq!(restored.as_str(), "Hymns 2.poentaro");
}

#[test]
fn a_folder_of_trashed_files_does_not_block_a_restore() {
    let files = [
        file("a", "Hymns.poentaro", true),
        file("b", "Hymns.poentaro/inner.poentaro", true),
    ];
    let restored = restored_path("a", "Hymns.poentaro", &files).unwrap();
    assert_eq!(restored.as_str(), "Hymns.poentaro");
}

#[test]
fn a_folder_move_renames_every_file_inside_it() {
    let files = live(&["A/x.poentaro", "A/B/y.poentaro", "Ab/z.poentaro"]);
    let FolderMovePlan::Move(moves) = plan_folder_move(&path("A"), &path("C/A"), &files) else {
        panic!("expected moves");
    };
    let renamed: Vec<(&str, &str)> = moves
        .iter()
        .map(|rename| (rename.id.as_str(), rename.name.as_str()))
        .collect();
    assert_eq!(
        renamed,
        [("id-0", "C/A/x.poentaro"), ("id-1", "C/A/B/y.poentaro")]
    );
}

#[test]
fn a_folder_already_at_its_destination_is_already_moved() {
    let files = live(&["C/A/x.poentaro"]);
    assert_eq!(
        plan_folder_move(&path("A"), &path("C/A"), &files),
        FolderMovePlan::AlreadyMoved
    );
}

#[test]
fn a_folder_with_no_files_is_no_such_folder() {
    let files = live(&["B/x.poentaro"]);
    assert_eq!(
        plan_folder_move(&path("A"), &path("C"), &files),
        FolderMovePlan::NoSuchFolder
    );
}

#[test]
fn a_folder_cannot_move_into_itself() {
    let files = live(&["A/x.poentaro"]);
    assert_eq!(
        plan_folder_move(&path("A"), &path("A/B"), &files),
        FolderMovePlan::IntoItself
    );
}

#[test]
fn folders_never_merge() {
    let files = live(&["A/x.poentaro", "C/y.poentaro"]);
    assert_eq!(
        plan_folder_move(&path("A"), &path("C"), &files),
        FolderMovePlan::DestinationExists
    );
}

fn file_paths() -> impl Strategy<Value = BTreeSet<String>> {
    let folder = prop::sample::select(vec!["A", "B", "A/B", "B/A", "A/A", "C"]);
    let leaf = prop::sample::select(vec!["x.poentaro", "y.poentaro"]);
    prop::collection::btree_set(
        (folder, leaf).prop_map(|(folder, leaf)| format!("{folder}/{leaf}")),
        0..8,
    )
}

fn folder() -> impl Strategy<Value = TreePath> {
    prop::sample::select(vec!["A", "B", "C", "A/B", "B/A", "D", "D/A"]).prop_map(path)
}

proptest! {
    /// Whenever the web app's rules allowed a folder move, the server moves
    /// the same files to the same paths.
    #[test]
    fn a_folder_move_matches_the_web_apps_rules(
        names in file_paths(),
        from in folder(),
        to in folder(),
    ) {
        let rows: Vec<FileNameRow> = names
            .iter()
            .map(|name| FileNameRow { id: format!("id:{name}"), name: name.clone() })
            .collect();
        let state = FileStoreState {
            user_files: names.iter().map(|name| (name.clone(), String::new())).collect(),
            file_ids: rows.iter().map(|row| (row.name.clone(), row.id.clone())).collect(),
            ..FileStoreState::default()
        };
        let Ok(outcome) = move_folder(&state, &from, &to, &FileOperationContext::default()) else {
            return Ok(());
        };
        let expected: BTreeSet<(String, String)> = outcome
            .changes
            .iter()
            .map(|change| (format!("id:{}", change.old_path.as_str()), change.new_path.as_str().to_string()))
            .collect();
        let planned: BTreeSet<(String, String)> = match plan_folder_move(&from, &to, &rows) {
            FolderMovePlan::Move(moves) => moves.into_iter().map(|rename| (rename.id, rename.name)).collect(),
            FolderMovePlan::AlreadyMoved => BTreeSet::new(),
            other => panic!("the web app moved it, the server planned {other:?}"),
        };
        prop_assert_eq!(planned, expected);
    }
}
