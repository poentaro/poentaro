//! What `POST /agent` decides without D1: the errors that stop a save, the
//! warnings that come back with one, the paths a new file may have, and new
//! files' ids.
#![allow(clippy::disallowed_macros)]

use live_share_worker::agent::{
    file_id, line_column, new_file_path_problem, score_diagnostics, LineColumn,
};

const PARTS: &str = "parts [\"Melody\" [M]: notes],\n";
const SCORE: &str = "score \"\"\"\n    [M] 1 2 3 4\n    \"\"\",\n";

#[test]
fn a_valid_score_has_no_errors_or_warnings() {
    assert_eq!(
        score_diagnostics(&format!("{PARTS}\n{SCORE}")),
        Default::default()
    );
}

#[test]
fn a_warning_comes_back_without_stopping_the_save() {
    let score = "score \"\"\"\n    [M] 1 2 (3) 4\n    \"\"\",\n";
    let diagnostics = score_diagnostics(&format!("{PARTS}\n{score}"));

    assert!(diagnostics.errors.is_empty(), "{:?}", diagnostics.errors);
    assert_eq!(diagnostics.warnings.len(), 1, "{:?}", diagnostics.warnings);
    assert_eq!(diagnostics.warnings[0].line, 4);
    assert!(diagnostics.warnings[0].message.contains("at least 2 notes"));
}

#[test]
fn a_score_with_a_shape_error_is_rejected_with_where_it_is() {
    let errors = score_diagnostics("tittle \"Hymn\",\n").errors;

    assert!(!errors.is_empty(), "expected an error");
    assert_eq!(errors[0].line, 1);
}

#[test]
fn a_part_with_a_general_midi_sound_needs_no_soundfont() {
    let errors = score_diagnostics(&format!(
        "parts [\"Melody\" [M]: notes {{ sound 52 }}],\n\n{SCORE}"
    ))
    .errors;

    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn line_and_column_count_from_one_in_characters() {
    assert_eq!(
        line_column("ab\n你好x", 0),
        LineColumn { line: 1, column: 1 }
    );
    assert_eq!(
        line_column("ab\n你好x", 3),
        LineColumn { line: 2, column: 1 }
    );
    assert_eq!(
        line_column("ab\n你好x", 9),
        LineColumn { line: 2, column: 3 }
    );
}

#[test]
fn an_offset_inside_a_character_or_past_the_end_counts_as_that_character() {
    assert_eq!(line_column("你好", 1), LineColumn { line: 1, column: 1 });
    assert_eq!(line_column("ab", 99), LineColumn { line: 1, column: 3 });
}

#[test]
fn a_new_file_needs_a_valid_path_ending_in_the_poentaro_extension() {
    assert_eq!(new_file_path_problem("Hymns/Advent.poentaro"), None);
    assert!(new_file_path_problem("Hymns/Advent.txt").is_some());
    assert!(new_file_path_problem("Hymns//Advent.poentaro").is_some());
    assert!(new_file_path_problem("../Advent.poentaro").is_some());
}

#[test]
fn a_file_id_is_a_version_4_uuid() {
    let id = file_id([0xff; 16]);

    assert_eq!(id, "ffffffff-ffff-4fff-bfff-ffffffffffff");
}
