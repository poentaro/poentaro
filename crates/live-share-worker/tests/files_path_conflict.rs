//! `files::find_file_folder_conflict`: a path can't be both a file and a
//! folder, judged on the final state of a write (live files only).
#![allow(clippy::disallowed_macros)]

use live_share_worker::files::{find_file_folder_conflict, FileNameRow};

fn rows(pairs: &[(&str, &str)]) -> Vec<FileNameRow> {
    pairs
        .iter()
        .map(|(id, name)| FileNameRow {
            id: (*id).to_string(),
            name: (*name).to_string(),
        })
        .collect()
}

#[test]
fn a_file_named_like_an_existing_folder_conflicts() {
    let live = rows(&[("a", "Hymns/Advent.poentaro")]);

    let conflict = find_file_folder_conflict(&live, &rows(&[("new", "Hymns")]));

    assert_eq!(conflict, Some("Hymns".to_string()));
}

#[test]
fn a_file_inside_an_existing_file_conflicts() {
    let live = rows(&[("a", "Hymns")]);

    let conflict = find_file_folder_conflict(&live, &rows(&[("new", "Hymns/Advent.poentaro")]));

    assert_eq!(conflict, Some("Hymns/Advent.poentaro".to_string()));
}

#[test]
fn a_deep_descendant_of_a_file_conflicts() {
    let live = rows(&[("a", "Hymns")]);

    let conflict = find_file_folder_conflict(&live, &rows(&[("new", "Hymns/Sub/Deep.poentaro")]));

    assert_eq!(conflict, Some("Hymns/Sub/Deep.poentaro".to_string()));
}

#[test]
fn siblings_and_similar_prefixes_do_not_conflict() {
    let live = rows(&[("a", "Hymns/Advent.poentaro"), ("b", "Hymns2")]);

    let conflict = find_file_folder_conflict(&live, &rows(&[("new", "Hymns/Easter.poentaro")]));

    assert_eq!(conflict, None);
}

#[test]
fn a_placeholder_inside_its_folder_does_not_conflict_with_it() {
    let live = rows(&[("a", "Hymns/Advent.poentaro")]);

    let conflict = find_file_folder_conflict(&live, &rows(&[("new", "Hymns/.keep")]));

    assert_eq!(conflict, None);
}

#[test]
fn a_file_named_like_a_folder_that_only_a_placeholder_holds_conflicts() {
    let live = rows(&[("a", "Hymns/.keep")]);

    let conflict = find_file_folder_conflict(&live, &rows(&[("new", "Hymns")]));

    assert_eq!(conflict, Some("Hymns".to_string()));
}

#[test]
fn a_renamed_file_is_judged_without_its_old_name() {
    let live = rows(&[("a", "Hymns")]);

    let conflict = find_file_folder_conflict(&live, &rows(&[("a", "Hymns/Advent.poentaro")]));

    assert_eq!(conflict, None);
}

#[test]
fn a_batch_is_judged_on_its_final_state() {
    // Moving the file `Hymns` away and a file into `Hymns/` in one batch is
    // fine; neither move alone would be.
    let live = rows(&[("a", "Hymns"), ("b", "Other.poentaro")]);

    let fine = find_file_folder_conflict(
        &live,
        &rows(&[("a", "Archive/Hymns"), ("b", "Hymns/Other.poentaro")]),
    );
    let clash = find_file_folder_conflict(
        &live,
        &rows(&[("a", "Hymns"), ("b", "Hymns/Other.poentaro")]),
    );

    assert_eq!(fine, None);
    assert_eq!(clash, Some("Hymns".to_string()));
}

#[test]
fn two_moves_of_one_batch_can_conflict_with_each_other() {
    let live = rows(&[("a", "One"), ("b", "Two")]);

    let conflict = find_file_folder_conflict(&live, &rows(&[("a", "X"), ("b", "X/Two")]));

    assert!(conflict.is_some());
}
