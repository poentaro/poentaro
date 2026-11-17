use super::path::TreePath;
use super::placeholder::{
    classify_cloud_listing, is_placeholder_path, placeholder_path, FolderPlaceholder, ListedEntry,
};

fn entry(id: &str, name: &str, trashed: bool) -> ListedEntry {
    ListedEntry {
        id: id.to_string(),
        name: name.to_string(),
        trashed,
    }
}

fn folder(raw: &str) -> TreePath {
    TreePath::parse(raw).unwrap()
}

#[test]
fn a_keep_leaf_inside_a_folder_is_a_placeholder() {
    assert!(is_placeholder_path("Hymns/.keep"));
    assert!(is_placeholder_path("Hymns/Advent/.keep"));
}

#[test]
fn other_names_are_not_placeholders() {
    assert!(!is_placeholder_path(".keep"));
    assert!(!is_placeholder_path("Hymns/.keep.poentaro"));
    assert!(!is_placeholder_path("Hymns/.keep/song.poentaro"));
    assert!(!is_placeholder_path("Hymns/song.poentaro"));
}

#[test]
fn placeholder_path_appends_the_reserved_leaf() {
    assert_eq!(
        placeholder_path(&folder("A/B")).unwrap().as_str(),
        "A/B/.keep"
    );
}

#[test]
fn active_placeholders_become_folders_and_all_are_hidden() {
    let result = classify_cloud_listing(&[
        entry("1", "Empty/.keep", false),
        entry("2", "Gone/.keep", true),
        entry("3", "Empty/song.poentaro", false),
        entry("4", "Top.poentaro", false),
    ]);
    assert_eq!(
        result.folders,
        vec![FolderPlaceholder {
            folder: folder("Empty"),
            file_id: "1".to_string()
        }]
    );
    assert_eq!(
        result.trashed_folders,
        vec![FolderPlaceholder {
            folder: folder("Gone"),
            file_id: "2".to_string()
        }]
    );
    assert_eq!(result.hidden_file_ids, vec!["1", "2"]);
}

#[test]
fn a_listing_without_placeholders_is_unchanged() {
    let result = classify_cloud_listing(&[entry("1", "a.poentaro", false)]);
    assert!(result.folders.is_empty());
    assert!(result.hidden_file_ids.is_empty());
}
