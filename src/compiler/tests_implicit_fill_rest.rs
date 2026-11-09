use crate::compiler::{compile, types::*};
use crate::grouper::group;
use crate::parser::parse;

fn score_from(source: &str) -> crate::ast::grouped::Score {
    let doc = parse(source, "test", &[]).unwrap();
    group(doc).unwrap()
}

#[test]
fn not_mentioned_part_s_filled_rest_is_marked_implicit_fill() {
    let score = score_from(
        r#"metadata { title "t", author "a", hide resting parts: no },

parts ["A": notes, "B": notes],

score """
    time=4/4 key=C4 bpm=120
    [A] 1 2 3 4
    """,
"#,
    );
    let result = compile(&score);
    let blocks = result.blocks;
    let b_row = &blocks[0].rows[1];
    assert_eq!(b_row.label, "B");
    let rests: Vec<_> = b_row
        .elements
        .iter()
        .filter_map(|e| match &e.content {
            ElementContent::Rest { implicit_fill, .. } => Some(*implicit_fill),
            _ => None,
        })
        .collect();
    assert_eq!(rests.len(), 4, "expected one rest per beat");
    assert!(
        rests.iter().all(|&implicit_fill| implicit_fill),
        "B was never mentioned in this measure, so every filled rest should be implicit_fill"
    );
}

#[test]
fn explicitly_written_rest_is_not_marked_implicit_fill() {
    let score = score_from(
        r#"metadata { title "t", author "a", hide resting parts: no },

parts ["A": notes, "B": notes],

score """
    time=4/4 key=C4 bpm=120
    [A] 1 2 3 4
    [B] 0 0 0 0
    """,
"#,
    );
    let result = compile(&score);
    let blocks = result.blocks;
    let b_row = &blocks[0].rows[1];
    assert_eq!(b_row.label, "B");
    let rests: Vec<_> = b_row
        .elements
        .iter()
        .filter_map(|e| match &e.content {
            ElementContent::Rest { implicit_fill, .. } => Some(*implicit_fill),
            _ => None,
        })
        .collect();
    assert_eq!(rests.len(), 4, "expected one rest per beat");
    assert!(
        rests.iter().all(|&implicit_fill| !implicit_fill),
        "B wrote its own rests explicitly, so none should be implicit_fill"
    );
}
