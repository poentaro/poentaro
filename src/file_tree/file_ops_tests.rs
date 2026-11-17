use super::*;

fn path(raw: &str) -> TreePath {
    TreePath::parse(raw).unwrap()
}

fn context() -> FileOperationContext {
    FileOperationContext {
        demo_files: vec![DemoFile {
            path: path("Demo.poentaro"),
            content: "demo content".to_string(),
        }],
        new_file_id: "new-id".to_string(),
    }
}

fn store(files: &[&str], bin: &[&str]) -> FileStoreState {
    FileStoreState {
        active: files.first().copied().unwrap_or_default().to_string(),
        user_files: files
            .iter()
            .map(|name| (name.to_string(), format!("content of {name}")))
            .collect(),
        bin: bin
            .iter()
            .map(|name| (name.to_string(), format!("binned {name}")))
            .collect(),
        file_ids: files
            .iter()
            .chain(bin)
            .map(|name| (name.to_string(), format!("id:{name}")))
            .collect(),
        folders: Vec::new(),
    }
}

fn file_names(state: &FileStoreState) -> Vec<&str> {
    state.user_files.keys().map(String::as_str).collect()
}

#[test]
fn create_file_at_top_level_is_named_and_active() {
    let state = store(&["a.poentaro"], &[]);
    let next = create_file(&state, None, "  Song ", "template", &context()).unwrap();
    assert_eq!(next.active, "Song.poentaro");
    assert_eq!(next.user_files["Song.poentaro"], "template");
    assert_eq!(next.file_ids["Song.poentaro"], "new-id");
}

#[test]
fn create_file_keeps_an_existing_extension() {
    let state = store(&[], &[]);
    let next = create_file(&state, None, "Song.poentaro", "t", &context()).unwrap();
    assert_eq!(next.active, "Song.poentaro");
}

#[test]
fn create_file_in_folder_is_scoped_to_that_folder() {
    let state = store(&["Song.poentaro", "Hymns/Other.poentaro"], &[]);
    let next = create_file(&state, Some(&path("Hymns")), "Song", "t", &context()).unwrap();
    assert_eq!(next.active, "Hymns/Song.poentaro");
}

#[test]
fn create_file_rejects_a_taken_name() {
    let state = store(&["Song.poentaro"], &[]);
    for taken in ["Song", "Song.poentaro"] {
        assert!(
            matches!(
                create_file(&state, None, taken, "t", &context()),
                Err(FileStoreError::Collision { .. })
            ),
            "{taken}"
        );
    }
}

#[test]
fn create_file_rejects_an_empty_or_nested_name() {
    let state = store(&[], &[]);
    assert!(create_file(&state, None, "  ", "t", &context()).is_err());
    assert!(create_file(&state, None, "A/B", "t", &context()).is_err());
}

#[test]
fn create_file_error_explains_a_clash_but_not_success() {
    let state = store(&["Song.poentaro"], &[]);
    assert!(create_file_error(&state, None, "Song", &context()).is_some());
    assert_eq!(create_file_error(&state, None, "New", &context()), None);
}

#[test]
fn create_file_inside_a_file_is_rejected() {
    let state = store(&["a.poentaro"], &[]);
    assert_eq!(
        create_file(&state, Some(&path("a.poentaro")), "t", "t", &context()),
        Err(FileStoreError::AncestorIsFile {
            path: path("a.poentaro")
        })
    );
}

#[test]
fn duplicate_stays_in_the_same_folder() {
    let state = store(&["Hymns/Advent/song.poentaro"], &[]);
    let next = duplicate_file(&state, &path("Hymns/Advent/song.poentaro"), &context()).unwrap();
    assert_eq!(next.active, "Hymns/Advent/song 2.poentaro");
    assert_eq!(
        next.user_files["Hymns/Advent/song 2.poentaro"],
        "content of Hymns/Advent/song.poentaro"
    );
    assert_ne!(
        next.file_ids["Hymns/Advent/song 2.poentaro"],
        "id:Hymns/Advent/song.poentaro"
    );
}

#[test]
fn duplicate_skips_binned_names_and_copies_demo_files_to_top_level() {
    let state = store(&["a.poentaro"], &["a 2.poentaro"]);
    let next = duplicate_file(&state, &path("a.poentaro"), &context()).unwrap();
    assert_eq!(next.active, "a 3.poentaro");
    let next = duplicate_file(&state, &path("Demo.poentaro"), &context()).unwrap();
    assert_eq!(next.active, "Demo 2.poentaro");
    assert_eq!(next.user_files["Demo 2.poentaro"], "demo content");
}

#[test]
fn rename_keeps_content_and_file_id() {
    let state = store(&["a.poentaro"], &[]);
    let next = rename_file(&state, "a.poentaro", "b", &context()).unwrap();
    assert_eq!(file_names(&next), vec!["b.poentaro"]);
    assert_eq!(next.user_files["b.poentaro"], "content of a.poentaro");
    assert_eq!(next.file_ids["b.poentaro"], "id:a.poentaro");
    assert!(!next.file_ids.contains_key("a.poentaro"));
    assert_eq!(next.active, "b.poentaro");
}

#[test]
fn move_file_into_new_folder_keeps_file_id() {
    let state = store(&["a.poentaro"], &[]);
    let next = rename_file(&state, "a.poentaro", "Hymns/Advent/a.poentaro", &context()).unwrap();
    assert_eq!(file_names(&next), vec!["Hymns/Advent/a.poentaro"]);
    assert_eq!(next.file_ids["Hymns/Advent/a.poentaro"], "id:a.poentaro");
}

#[test]
fn rename_to_same_path_is_a_no_op() {
    let state = store(&["a.poentaro"], &[]);
    assert_eq!(
        rename_file(&state, "a.poentaro", " a ", &context()),
        Ok(state)
    );
}

#[test]
fn rename_collisions_are_rejected() {
    let state = store(
        &["a.poentaro", "b.poentaro", "F.poentaro/x.poentaro"],
        &["gone.poentaro"],
    );
    for target in ["b.poentaro", "gone.poentaro", "F.poentaro", "Demo.poentaro"] {
        let result = rename_file(&state, "a.poentaro", target, &context());
        assert!(
            matches!(result, Err(FileStoreError::Collision { .. })),
            "{target}: {result:?}"
        );
    }
}

#[test]
fn rename_under_a_file_or_with_bad_path_is_rejected() {
    let state = store(&["a.poentaro", "b.poentaro"], &[]);
    assert!(matches!(
        rename_file(&state, "a.poentaro", "b.poentaro/c", &context()),
        Err(FileStoreError::AncestorIsFile { .. })
    ));
    assert!(matches!(
        rename_file(&state, "a.poentaro", "x//y", &context()),
        Err(FileStoreError::InvalidPath { .. })
    ));
    assert!(matches!(
        rename_file(&state, "a.poentaro", "folder/", &context()),
        Err(FileStoreError::InvalidPath { .. })
    ));
    assert!(matches!(
        rename_file(&state, "a.poentaro", &"x".repeat(1020), &context()),
        Err(FileStoreError::InvalidPath { .. })
    ));
}

#[test]
fn demo_and_missing_files_cannot_be_renamed_or_deleted() {
    let state = store(&["a.poentaro"], &[]);
    assert!(matches!(
        rename_file(&state, "Demo.poentaro", "x", &context()),
        Err(FileStoreError::DemoFileReadOnly { .. })
    ));
    assert!(matches!(
        delete_file(&state, "Demo.poentaro", &context()),
        Err(FileStoreError::DemoFileReadOnly { .. })
    ));
    assert!(matches!(
        delete_file(&state, "nope.poentaro", &context()),
        Err(FileStoreError::NotFound { .. })
    ));
}

#[test]
fn delete_bins_under_full_path_and_picks_next_active() {
    let state = store(&["A/a.poentaro", "b.poentaro"], &[]);
    let next = delete_file(&state, "A/a.poentaro", &context()).unwrap();
    assert_eq!(next.bin["A/a.poentaro"], "content of A/a.poentaro");
    assert_eq!(next.active, "b.poentaro");
    assert_eq!(next.file_ids["A/a.poentaro"], "id:A/a.poentaro");
    let next = delete_file(&next, "b.poentaro", &context()).unwrap();
    assert_eq!(next.active, "Demo.poentaro");
}

#[test]
fn restore_goes_back_to_original_path_recreating_the_folder() {
    let state = store(&["b.poentaro"], &["Hymns/Advent/a.poentaro"]);
    let next = restore_file(&state, "Hymns/Advent/a.poentaro", &context()).unwrap();
    assert_eq!(next.active, "Hymns/Advent/a.poentaro");
    assert_eq!(
        next.user_files["Hymns/Advent/a.poentaro"],
        "binned Hymns/Advent/a.poentaro"
    );
    assert!(next.bin.is_empty());
    assert_eq!(
        next.file_ids["Hymns/Advent/a.poentaro"],
        "id:Hymns/Advent/a.poentaro"
    );
}

#[test]
fn restore_picks_unique_name_when_path_is_taken() {
    let state = store(&["F/a.poentaro"], &["F/a.poentaro_"]);
    let mut state = state;
    state.bin.insert("F/a.poentaro".into(), "old".into());
    state.bin.remove("F/a.poentaro_");
    // A live file now shares the bin key (e.g. from an older flat store).
    let next = restore_file(&state, "F/a.poentaro", &context()).unwrap();
    assert_eq!(next.active, "F/a 2.poentaro");
    assert_eq!(next.user_files["F/a 2.poentaro"], "old");
    assert_eq!(next.user_files["F/a.poentaro"], "content of F/a.poentaro");
    assert_eq!(next.file_ids["F/a 2.poentaro"], "id:F/a.poentaro");
}

#[test]
fn restore_under_a_file_fails_and_keeps_the_bin() {
    let state = store(&["F"], &["F/a.poentaro"]);
    assert!(matches!(
        restore_file(&state, "F/a.poentaro", &context()),
        Err(FileStoreError::AncestorIsFile { .. })
    ));
    assert!(matches!(
        restore_file(&state, "nope", &context()),
        Err(FileStoreError::NotFound { .. })
    ));
}

#[test]
fn rename_to_the_reserved_placeholder_leaf_is_rejected() {
    let state = store(&["a.poentaro"], &[]);
    let error = rename_file(&state, "a.poentaro", "Songs/.keep", &context()).unwrap_err();
    assert_eq!(
        error,
        FileStoreError::ReservedName {
            path: path("Songs/.keep")
        }
    );
    assert_eq!(
        error.to_string(),
        "`.keep` is a reserved name and cannot be used"
    );
}

#[test]
fn rename_error_reports_the_reserved_leaf() {
    let state = store(&["a.poentaro"], &[]);
    assert_eq!(
        rename_error(&state, "a.poentaro", "Songs/.keep", &context()).as_deref(),
        Some("`.keep` is a reserved name and cannot be used")
    );
}

#[test]
fn a_name_that_merely_starts_with_keep_is_fine() {
    let state = store(&["a.poentaro"], &[]);
    let next = rename_file(&state, "a.poentaro", "Songs/.keep.poentaro", &context()).unwrap();
    assert!(next.user_files.contains_key("Songs/.keep.poentaro"));
}
