use super::*;

fn store(bin: &[&str]) -> FileStoreState {
    FileStoreState {
        bin: bin
            .iter()
            .map(|name| (name.to_string(), String::new()))
            .collect(),
        ..Default::default()
    }
}

fn entry(key: &str, name: &str, location: Option<&str>) -> BinEntry {
    BinEntry {
        key: key.to_string(),
        name: name.to_string(),
        location: location.map(str::to_string),
    }
}

#[test]
fn entries_split_the_leaf_name_from_the_original_folder() {
    let state = store(&["Old.poentaro", "Hymns/Advent/Veni.poentaro"]);
    assert_eq!(
        bin_entries(&state),
        vec![
            entry("Old.poentaro", "Old.poentaro", None),
            entry(
                "Hymns/Advent/Veni.poentaro",
                "Veni.poentaro",
                Some("Hymns/Advent")
            ),
        ]
    );
}

#[test]
fn entries_sort_by_name_then_location_ignoring_case() {
    let state = store(&[
        "b/Veni.poentaro",
        "A/veni.poentaro",
        "Zed.poentaro",
        "Veni.poentaro",
    ]);
    let keys: Vec<String> = bin_entries(&state).into_iter().map(|e| e.key).collect();
    assert_eq!(
        keys,
        vec![
            "Veni.poentaro",
            "A/veni.poentaro",
            "b/Veni.poentaro",
            "Zed.poentaro"
        ]
    );
}

#[test]
fn an_invalid_key_is_listed_whole_at_the_top_level() {
    let state = store(&["a//b.poentaro"]);
    assert_eq!(
        bin_entries(&state),
        vec![entry("a//b.poentaro", "a//b.poentaro", None)]
    );
}
