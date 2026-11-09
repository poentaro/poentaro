use crate::compiler::compile;

fn score_from(source: &str) -> crate::ast::grouped::Score {
    let doc = crate::parser::parse(source, "test", &[]).unwrap();
    crate::grouper::group(doc).unwrap()
}

#[test]
fn hide_resting_parts_directive_toggles_mid_score() {
    let score = score_from(
        r#"metadata { title "t", author "a" },

parts ["A": notes, "B": chords],

score """
    time=4/4 key=C4 bpm=120
    [A] 1 2 3 4
    la la la la

    hide_resting_parts=no
    [A] 5 6 7 1
    lo lo lo lo
    """,
"#,
    );
    let result = compile(&score);
    let blocks = result.blocks;
    assert_eq!(
        blocks[0].rows.len(),
        1,
        "measure 1: default hide_resting_parts=yes omits rest-filled B"
    );
    assert_eq!(
        blocks[1].rows.len(),
        2,
        "measure 2: hide_resting_parts=no keeps rest-filled B"
    );
}
