use super::*;

fn store(files: &[&str], bin: &[&str]) -> FileStoreState {
    let entries = |paths: &[&str]| {
        paths
            .iter()
            .map(|path| (path.to_string(), format!("content of {path}")))
            .collect()
    };
    FileStoreState {
        active: files
            .first()
            .map(|path| path.to_string())
            .unwrap_or_default(),
        user_files: entries(files),
        bin: entries(bin),
        file_ids: files
            .iter()
            .chain(bin)
            .map(|path| (path.to_string(), format!("id of {path}")))
            .collect(),
        folders: Vec::new(),
    }
}

fn keys(map: &std::collections::BTreeMap<String, String>) -> Vec<&str> {
    map.keys().map(String::as_str).collect()
}

#[test]
fn the_extension_comes_from_the_shared_file() {
    assert_eq!(FILE_EXTENSION, ".poentaro");
    assert_eq!(LEGACY_FILE_EXTENSION, ".jianpu");
}

#[test]
fn both_extensions_are_hidden_from_the_shown_name() {
    assert_eq!(strip_file_extension("Hymns/a.poentaro"), "Hymns/a");
    assert_eq!(strip_file_extension("a.jianpu"), "a");
    assert_eq!(strip_file_extension("notes.txt"), "notes.txt");
}

#[test]
fn legacy_files_are_renamed_keeping_content_ids_and_the_active_file() {
    let state = store(&["Hymns/a.jianpu", "b.poentaro"], &["old.jianpu"]);

    let renamed = rename_legacy_extensions(&state, &FileOperationContext::default()).unwrap();

    assert_eq!(renamed.active, "Hymns/a.poentaro");
    assert_eq!(
        keys(&renamed.user_files),
        ["Hymns/a.poentaro", "b.poentaro"]
    );
    assert_eq!(
        renamed.user_files["Hymns/a.poentaro"],
        "content of Hymns/a.jianpu"
    );
    assert_eq!(keys(&renamed.bin), ["old.poentaro"]);
    assert_eq!(renamed.file_ids["old.poentaro"], "id of old.jianpu");
    assert_eq!(renamed.file_ids["Hymns/a.poentaro"], "id of Hymns/a.jianpu");
}

#[test]
fn a_legacy_file_whose_new_name_is_taken_keeps_its_name() {
    let state = store(&["a.jianpu", "a.poentaro"], &[]);

    assert_eq!(
        rename_legacy_extensions(&state, &FileOperationContext::default()),
        None
    );
}

#[test]
fn a_store_without_legacy_files_needs_no_rename() {
    let state = store(&["a.poentaro"], &["b.txt"]);

    assert_eq!(
        rename_legacy_extensions(&state, &FileOperationContext::default()),
        None
    );
}
