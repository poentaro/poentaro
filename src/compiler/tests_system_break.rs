use crate::compiler::compile;

fn score_from(source: &str) -> crate::ast::grouped::Score {
    let doc = crate::parser::parse(source, "test", &[]).unwrap();
    crate::grouper::group(doc).unwrap()
}

#[test]
fn break_directive_sets_system_break_only_on_its_own_measure() {
    let score = score_from(
        r#"metadata { title "t", author "a" },

parts ["S": notes],

score """
    time=4/4 key=C4 bpm=120
    [S] 1 2 3 4

    break
    [S] 5 6 7 1

    [S] 1 2 3 4
    """,
"#,
    );
    let result = compile(&score);
    let blocks = result.blocks;
    assert!(!blocks[0].system_break, "measure 1: no break directive");
    assert!(blocks[1].system_break, "measure 2: break directive present");
    assert!(
        !blocks[2].system_break,
        "measure 3: break does not persist from measure 2"
    );
}

#[test]
fn break_directive_combines_with_other_directives_on_same_line() {
    let score = score_from(
        r#"metadata { title "t", author "a" },

parts ["S": notes],

score """
    time=4/4 key=C4 bpm=120
    [S] 1 2 3 4

    bpm=100 break
    [S] 5 6 7 1
    """,
"#,
    );
    let result = compile(&score);
    let blocks = result.blocks;
    assert_eq!(blocks[1].decorations.len(), 1);
    let crate::compiler::types::Decoration::DirectiveLine { bpm, .. } = &blocks[1].decorations[0];
    assert_eq!(*bpm, Some(100));
    assert!(blocks[1].system_break);
}

#[test]
fn break_directive_on_a_measure_absorbed_by_a_rest_run_still_starts_a_fresh_run() {
    // Measures 2 and 3 are both all-rest and would normally merge into one
    // `MultiMeasureRest` block; measure 3's `break` must keep it out of the
    // run measure 1-2 would otherwise form, so the forced boundary lands
    // exactly where the user wrote it.
    let score = score_from(
        r#"metadata { title "t", author "a" },

parts ["S": notes],

score """
    time=4/4 key=C4 bpm=120
    [S] 1 2 3 4

    [S] 0 0 0 0

    break
    [S] 0 0 0 0

    [S] 0 0 0 0
    """,
"#,
    );
    let result = compile(&score);
    let blocks = result.blocks;
    // Measures 2-3 don't merge (measure 3 carries `break`), but measures 3-4
    // do (both plain all-rest, no directive).
    assert_eq!(
        blocks.len(),
        3,
        "measure 1, measure 2 alone, measures 3-4 merged"
    );
    assert!(!blocks[0].system_break);
    assert!(!blocks[1].system_break);
    assert!(blocks[2].system_break);
    assert_eq!(blocks[2].represents_measures, 2);
}
