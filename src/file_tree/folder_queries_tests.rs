use super::*;

fn path(raw: &str) -> TreePath {
    TreePath::parse(raw).unwrap()
}

fn context() -> FileOperationContext {
    FileOperationContext {
        demo_files: vec![],
        new_file_id: "new-id".to_string(),
    }
}

fn store(files: &[&str], folders: &[&str], bin: &[&str]) -> FileStoreState {
    FileStoreState {
        active: String::new(),
        user_files: files
            .iter()
            .map(|name| (name.to_string(), String::new()))
            .collect(),
        bin: bin
            .iter()
            .map(|name| (name.to_string(), String::new()))
            .collect(),
        file_ids: Default::default(),
        folders: folders.iter().map(|raw| path(raw)).collect(),
    }
}

#[test]
fn move_error_is_none_when_the_move_would_succeed() {
    let state = store(&["A/x.poentaro"], &[], &[]);
    assert_eq!(folder_move_error(&state, "A", "B", &context()), None);
    assert_eq!(folder_move_error(&state, "A", "A", &context()), None);
}

#[test]
fn move_error_explains_a_clash_with_a_folder_or_a_bin_entry() {
    let state = store(&["A/x.poentaro", "B/y.poentaro"], &[], &["C/z.poentaro"]);
    assert_eq!(
        folder_move_error(&state, "A", "B", &context()).as_deref(),
        Some("`B` already exists")
    );
    assert_eq!(
        folder_move_error(&state, "A", " C/z.poentaro ", &context()).as_deref(),
        Some("`C/z.poentaro` already exists")
    );
}

#[test]
fn move_error_explains_invalid_paths_and_moving_into_itself() {
    let state = store(&["A/x.poentaro"], &[], &[]);
    assert!(folder_move_error(&state, "A", "B//C", &context())
        .unwrap()
        .contains("empty segment"));
    assert!(folder_move_error(&state, "A", "../B", &context())
        .unwrap()
        .contains("not allowed"));
    assert!(folder_move_error(&state, "A", "A/Inner", &context())
        .unwrap()
        .contains("into itself"));
}

#[test]
fn delete_summary_counts_files_at_any_depth() {
    let state = store(
        &["A/x.poentaro", "A/B/y.poentaro", "other.poentaro"],
        &[],
        &[],
    );
    let summary = folder_delete_summary(&state, &path("A"), &ShareOverview::default()).unwrap();
    assert_eq!(summary.file_count, 2);
    assert!(summary.message.starts_with("2 files will move to the bin"));
}

#[test]
fn delete_summary_is_singular_for_one_file() {
    let state = store(&["A/x.poentaro"], &[], &[]);
    let summary = folder_delete_summary(&state, &path("A"), &ShareOverview::default()).unwrap();
    assert!(summary.message.starts_with("1 file will move"));
}

#[test]
fn delete_summary_for_an_empty_folder_and_a_missing_one() {
    let state = store(&[], &["Empty"], &[]);
    let summary = folder_delete_summary(&state, &path("Empty"), &ShareOverview::default()).unwrap();
    assert_eq!(summary.file_count, 0);
    assert!(summary.message.contains("empty folder"));
    assert_eq!(
        folder_delete_summary(&state, &path("Nope"), &ShareOverview::default()),
        None
    );
}

#[test]
fn reveal_lists_the_folder_and_its_ancestors_outermost_first() {
    assert_eq!(folders_to_reveal("A/B/C"), vec!["A", "A/B", "A/B/C"]);
    assert_eq!(folders_to_reveal("A"), vec!["A"]);
    assert!(folders_to_reveal("A//B").is_empty());
}

#[test]
fn delete_summary_carries_the_share_warning() {
    let state = store(&["A/x.poentaro"], &[], &[]);
    let shares = ShareOverview {
        files: vec![],
        folders: vec![FolderShareLink {
            share_id: "S".to_string(),
            folder: "A".to_string(),
        }],
    };
    let summary = folder_delete_summary(&state, &path("A"), &shares).unwrap();
    assert!(summary
        .share_warning
        .unwrap()
        .contains("shared with a link"));
    let plain = folder_delete_summary(&state, &path("A"), &ShareOverview::default()).unwrap();
    assert_eq!(plain.share_warning, None);
}
