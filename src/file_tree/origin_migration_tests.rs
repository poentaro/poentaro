use super::*;

fn store(files: &[(&str, &str)], folders: &[&str]) -> FileStoreState {
    FileStoreState {
        active: files
            .first()
            .map(|(path, _)| path.to_string())
            .unwrap_or_default(),
        user_files: files
            .iter()
            .map(|(path, _)| (path.to_string(), format!("content of {path}")))
            .collect(),
        file_ids: files
            .iter()
            .map(|(path, id)| (path.to_string(), id.to_string()))
            .collect(),
        folders: folders
            .iter()
            .map(|raw| TreePath::parse(raw).unwrap())
            .collect(),
        ..FileStoreState::default()
    }
}

fn context() -> FileOperationContext {
    FileOperationContext {
        new_file_id: "fresh".to_string(),
        ..FileOperationContext::default()
    }
}

fn keys(map: &std::collections::BTreeMap<String, String>) -> Vec<&str> {
    map.keys().map(String::as_str).collect()
}

#[test]
fn an_empty_new_origin_takes_the_carried_store_whole() {
    let carried = store(&[("Hymns/a.poentaro", "id-a")], &["Empty"]);

    let merged = merge_migrated_file_store(&FileStoreState::default(), &carried, &context());

    assert_eq!(merged, Ok(carried));
}

#[test]
fn nothing_carried_keeps_the_new_origin_as_is() {
    let current = store(&[("mine.poentaro", "id-m")], &[]);

    let merged = merge_migrated_file_store(&current, &FileStoreState::default(), &context());

    assert_eq!(merged, Ok(current));
}

#[test]
fn carried_files_go_under_a_new_folder_next_to_existing_ones() {
    let current = store(&[("a.poentaro", "id-1")], &[]);
    let carried = store(
        &[("a.poentaro", "id-2"), ("Hymns/b.poentaro", "id-3")],
        &["Empty"],
    );

    let merged = merge_migrated_file_store(&current, &carried, &context()).unwrap();

    assert_eq!(
        keys(&merged.user_files),
        [
            "From jianpu-generator.pages.dev/Hymns/b.poentaro",
            "From jianpu-generator.pages.dev/a.poentaro",
            "a.poentaro",
        ]
    );
    assert_eq!(
        merged
            .file_ids
            .get("From jianpu-generator.pages.dev/a.poentaro"),
        Some(&"id-2".to_string())
    );
    assert_eq!(
        merged.folders,
        [TreePath::parse("From jianpu-generator.pages.dev/Empty").unwrap()]
    );
    assert_eq!(merged.active, "a.poentaro");
}

#[test]
fn the_folder_gets_a_number_when_its_name_is_taken() {
    let current = store(
        &[("From jianpu-generator.pages.dev/x.poentaro", "id-1")],
        &[],
    );
    let carried = store(&[("y.poentaro", "id-2")], &[]);

    let merged = merge_migrated_file_store(&current, &carried, &context()).unwrap();

    assert!(merged
        .user_files
        .contains_key("From jianpu-generator.pages.dev 2/y.poentaro"));
}

#[test]
fn a_carried_id_already_in_use_is_replaced() {
    let current = store(&[("a.poentaro", "same")], &[]);
    let carried = store(&[("b.poentaro", "same")], &[]);

    let merged = merge_migrated_file_store(&current, &carried, &context()).unwrap();

    assert_eq!(
        merged
            .file_ids
            .get("From jianpu-generator.pages.dev/b.poentaro"),
        Some(&"fresh-0".to_string())
    );
}

#[test]
fn carried_bin_entries_move_under_the_folder_too() {
    let current = store(&[("a.poentaro", "id-1")], &[]);
    let carried = FileStoreState {
        bin: [("old.poentaro".to_string(), "gone".to_string())].into(),
        ..FileStoreState::default()
    };

    let merged = merge_migrated_file_store(&current, &carried, &context()).unwrap();

    assert_eq!(
        keys(&merged.bin),
        ["From jianpu-generator.pages.dev/old.poentaro"]
    );
}
