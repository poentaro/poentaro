use super::*;
use std::collections::HashSet;

fn path(raw: &str) -> TreePath {
    TreePath::parse(raw).unwrap()
}

fn taken(raw: &[&str]) -> HashSet<TreePath> {
    raw.iter().map(|raw| path(raw)).collect()
}

fn unique(desired: &str, taken_paths: &[&str]) -> String {
    unique_path(&path(desired), &taken(taken_paths))
        .unwrap()
        .as_str()
        .to_string()
}

#[test]
fn free_name_is_kept() {
    assert_eq!(unique("untitled.poentaro", &[]), "untitled.poentaro");
    assert_eq!(unique("a/b.poentaro", &["a/c.poentaro"]), "a/b.poentaro");
}

#[test]
fn collision_gets_numeric_suffix_before_extension() {
    assert_eq!(
        unique("untitled.poentaro", &["untitled.poentaro"]),
        "untitled 2.poentaro"
    );
}

#[test]
fn skips_taken_suffixes() {
    assert_eq!(
        unique(
            "untitled.poentaro",
            &[
                "untitled.poentaro",
                "untitled 2.poentaro",
                "untitled 3.poentaro"
            ]
        ),
        "untitled 4.poentaro"
    );
}

#[test]
fn collisions_only_matter_within_the_same_folder() {
    assert_eq!(
        unique(
            "a/x.poentaro",
            &["x.poentaro", "b/x.poentaro", "a/b/x.poentaro"]
        ),
        "a/x.poentaro"
    );
    assert_eq!(
        unique(
            "a/x.poentaro",
            &["a/x.poentaro", "x 2.poentaro", "b/x 2.poentaro"]
        ),
        "a/x 2.poentaro"
    );
}

#[test]
fn sibling_folder_with_same_name_counts_as_taken() {
    assert_eq!(unique("a/x", &["a/x"]), "a/x 2.poentaro");
}

#[test]
fn name_without_extension_gets_default_extension() {
    assert_eq!(unique("song", &["song"]), "song 2.poentaro");
}

#[test]
fn leading_dot_only_is_not_an_extension() {
    assert_eq!(unique(".hidden", &[".hidden"]), ".hidden 2.poentaro");
}

#[test]
fn other_extension_is_preserved() {
    assert_eq!(unique("a.txt", &["a.txt"]), "a 2.txt");
}

#[test]
fn suffix_pushing_past_the_limit_is_an_error() {
    let long = format!("{}a.poentaro", "x".repeat(1024 - "a.poentaro".len()));
    assert_eq!(long.len(), 1024);
    let result = unique_path(&path(&long), &taken(&[&long]));
    assert!(matches!(result, Err(PathError::TooLong { .. })));
}

fn after_collision(rejected: &str) -> String {
    path_after_collision(&path(rejected))
        .unwrap()
        .as_str()
        .to_string()
}

#[test]
fn after_collision_appends_two_then_increments() {
    assert_eq!(after_collision("a.poentaro"), "a 2.poentaro");
    assert_eq!(after_collision("a 2.poentaro"), "a 3.poentaro");
    assert_eq!(after_collision("a 9.poentaro"), "a 10.poentaro");
}

#[test]
fn after_collision_keeps_the_folder_prefix() {
    assert_eq!(
        after_collision("Hymns/Advent/O Come.poentaro"),
        "Hymns/Advent/O Come 2.poentaro"
    );
    assert_eq!(after_collision("Hymns/X 2.poentaro"), "Hymns/X 3.poentaro");
}

#[test]
fn after_collision_ignores_a_one_or_oversized_suffix() {
    assert_eq!(after_collision("a 1.poentaro"), "a 1 2.poentaro");
    assert_eq!(after_collision("a 99999.poentaro"), "a 99999 2.poentaro");
}

#[test]
fn after_collision_without_extension_gets_poentaro() {
    assert_eq!(after_collision("notes"), "notes 2.poentaro");
}

#[test]
fn after_collision_fails_when_the_suffix_would_exceed_the_limit() {
    let long = format!(
        "{}.poentaro",
        "a".repeat(MAX_PATH_BYTES - ".poentaro".len())
    );
    assert!(path_after_collision(&path(&long)).is_err());
}
