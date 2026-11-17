#![allow(clippy::disallowed_macros)]

use live_share_worker::file_history::apply_patch;
use migrate_comments::{migrate, FileRow, Outcome};

fn row(content: &str) -> FileRow {
    FileRow {
        id: "id".to_string(),
        name: "a.poentaro".to_string(),
        content: content.to_string(),
        revision: 3,
    }
}

#[test]
fn turns_comments_into_double_hash_comments_and_leaves_the_score_alone() {
    let old = "# about the song\nmetadata { title \"Song\" },\n\nscore \"\"\"\n    [M] #1 2\n    \"\"\",\n";
    let Ok(Outcome::Converted(migration)) = migrate(&row(old)) else {
        panic!("expected {old:?} to be converted");
    };
    assert_eq!(
        migration.converted,
        "## about the song\nmetadata { title \"Song\" },\n\nscore \"\"\"\n    [M] #1 2\n    \"\"\",\n"
    );
    assert_eq!(migration.revision, 3);
    assert_eq!(
        apply_patch(&migration.reverse_patch, &migration.converted).as_deref(),
        Ok(old)
    );
}

#[test]
fn skips_a_file_with_no_single_hash_comments() {
    assert_eq!(
        migrate(&row("## done\nmetadata { title \"Song\" },\n")),
        Ok(Outcome::Unchanged)
    );
}

#[test]
fn holds_text_in_the_removed_section_syntax() {
    let old = "# metadata\ntitle = \"Song\"\n\n# score\n1 2 3 4\n";
    assert_eq!(migrate(&row(old)), Ok(Outcome::Held));
}
