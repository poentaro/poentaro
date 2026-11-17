use super::*;

fn path(raw: &str) -> TreePath {
    TreePath::parse(raw).unwrap()
}

fn context() -> FileOperationContext {
    FileOperationContext {
        demo_files: vec![DemoFile {
            path: path("Demo.poentaro"),
            content: String::new(),
        }],
        new_file_id: "new-id".to_string(),
    }
}

fn store(files: &[&str], folders: &[&str]) -> FileStoreState {
    FileStoreState {
        active: files.first().copied().unwrap_or_default().to_string(),
        user_files: files
            .iter()
            .map(|name| (name.to_string(), String::new()))
            .collect(),
        bin: Default::default(),
        file_ids: Default::default(),
        folders: folders.iter().map(|raw| path(raw)).collect(),
    }
}

fn labels(targets: &[MoveTarget]) -> Vec<String> {
    targets
        .iter()
        .map(|target| format!("{}{}", "-".repeat(target.depth), target.name))
        .collect()
}

#[test]
fn targets_list_top_level_then_every_folder_depth_first() {
    let state = store(
        &["Hymns/Advent/O Come.poentaro", "Scratch.poentaro"],
        &["Empty"],
    );
    assert_eq!(
        labels(&move_targets(&state, None)),
        vec!["Top level", "-Empty", "-Hymns", "--Advent"]
    );
}

#[test]
fn targets_carry_full_paths_with_empty_path_for_top_level() {
    let state = store(&["Hymns/Advent/x.poentaro"], &[]);
    let paths: Vec<String> = move_targets(&state, None)
        .into_iter()
        .map(|target| target.path)
        .collect();
    assert_eq!(paths, vec!["", "Hymns", "Hymns/Advent"]);
}

#[test]
fn excluded_folder_and_its_descendants_are_not_offered() {
    let state = store(
        &["A/B/x.poentaro", "A/y.poentaro", "Other/z.poentaro"],
        &["A/Empty"],
    );
    assert_eq!(
        labels(&move_targets(&state, Some(&path("A")))),
        vec!["Top level", "-Other"]
    );
    assert_eq!(
        labels(&move_targets(&state, Some(&path("A/B")))),
        vec!["Top level", "-A", "--Empty", "-Other"]
    );
}

#[test]
fn excluding_a_folder_keeps_siblings_sharing_its_name_prefix() {
    let state = store(&["A/x.poentaro", "AB/y.poentaro"], &[]);
    assert_eq!(
        labels(&move_targets(&state, Some(&path("A")))),
        vec!["Top level", "-AB"]
    );
}

#[test]
fn destination_keeps_the_name_and_swaps_the_folder() {
    assert_eq!(
        move_destination("Hymns/O Come.poentaro", Some(&path("Archive/Old")))
            .unwrap()
            .as_str(),
        "Archive/Old/O Come.poentaro"
    );
    assert_eq!(
        move_destination("Hymns/O Come.poentaro", None)
            .unwrap()
            .as_str(),
        "O Come.poentaro"
    );
}

#[test]
fn rename_error_reports_clash_and_invalid_paths_but_not_success() {
    let state = store(&["a.poentaro", "Docs/a.poentaro"], &[]);
    let error = |to: &str| rename_error(&state, "a.poentaro", to, &context());
    assert_eq!(
        error("Docs/a"),
        Some("`Docs/a.poentaro` already exists".into())
    );
    assert_eq!(
        error("Docs//a"),
        Some("invalid path: path has an empty segment".into())
    );
    assert!(error("../a").unwrap().contains("not allowed"));
    assert!(error(&"x".repeat(2000)).unwrap().contains("limit is 1024"));
    assert_eq!(error("Docs/b"), None);
    assert_eq!(error("a"), None);
}
