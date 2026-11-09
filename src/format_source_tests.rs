//! Tests for [`super::format_measures`], through the document formatter.

use crate::format_anyfix::format_anyfix as format_score;

#[test]
fn drops_a_trailing_all_rest_measure_for_a_plain_notes_part() {
    // Melody's second-measure rest line is entirely redundant with
    // implicit-fill; Bass has real content in the same measure group, so
    // the group isn't left empty and the drop can actually happen.
    let source = r#"parts ["Melody": notes, "Bass": notes],

score """
    [Melody] 1 2 3 4
    [Bass] 5 6 7 1

    [Melody] 0 0 0 0
    [Bass] 3 3 3 3
    """,
"#;
    let formatted = format_score(source);
    assert!(
        !formatted.contains("[Melody] 0 0 0 0"),
        "trailing all-rest measure should be dropped:\n{formatted}"
    );
    assert!(formatted.contains("[Melody] 1 2 3 4"));
    assert!(formatted.contains("[Bass] 3 3 3 3"));
}

#[test]
fn keeps_a_non_trailing_all_rest_verse_when_a_later_verse_has_content() {
    let source = r#"parts ["Melody": notes],

score """
    [Melody] 1 2 3 4
    _ _ _ _
    hel lo world here
    """,
"#;
    let formatted = format_score(source);
    let score_section = formatted.split("score").nth(1).unwrap();
    assert_eq!(
        score_section.matches("[Melody]").count(),
        1,
        "the notes line keeps its [Melody] prefix; verse lines stay positional:\n{formatted}"
    );
    assert!(
        score_section.contains("_ _ _ _"),
        "the earlier all-rest verse must survive since a later verse has real content:\n{formatted}"
    );
    assert!(score_section.contains("hel lo world here"));
}

#[test]
fn never_drops_an_explicit_all_rest_line_on_a_follow_part() {
    let source = r#"parts ["Melody": notes, "Echo": follow [Melody]],

score """
    [Melody] 1 2 3 4
    [Echo] 0 0 0 0
    """,
"#;
    let formatted = format_score(source);
    assert!(
        formatted.contains("[Echo] 0 0 0 0"),
        "an explicit rest line on a follow[X] part is real content, not implicit fill:\n{formatted}"
    );
}

#[test]
fn collapses_irregular_whitespace_on_data_lines() {
    let source = r#"parts ["Melody": notes],

score """
    [Melody]   1    2  3   4
    """,
"#;
    let formatted = format_score(source);
    assert!(
        formatted.contains("[Melody] 1 2 3 4"),
        "irregular internal whitespace should collapse to single spaces:\n{formatted}"
    );
}

#[test]
fn collapses_irregular_whitespace_on_a_directive_line_with_a_quoted_label() {
    let source = r#"parts ["Melody": notes],

score """
    bpm=92    key=C4   label="Two   Words"
    [Melody] 1 2 3 4
    """,
"#;
    let formatted = format_score(source);
    assert!(
        formatted.contains("bpm=92 key=C4 label=\"Two   Words\""),
        "directive tokens collapse to single spaces, but a quoted value's internal spacing is preserved verbatim:\n{formatted}"
    );
}

#[test]
fn keeps_at_least_one_data_line_when_every_line_would_otherwise_be_dropped() {
    let source = r#"parts ["Melody": notes, "Bass": notes],

score """
    [Melody] 0 0 0 0
    [Bass] 0 0 0 0
    """,
"#;
    let formatted = format_score(source);
    let score_section = formatted.split("score").nth(1).unwrap();
    let data_line_count = score_section
        .lines()
        .filter(|l| l.trim_start().starts_with('['))
        .count();
    assert_eq!(
        data_line_count, 1,
        "an empty measure group is a parse error, so exactly one line must survive:\n{formatted}"
    );
}

#[test]
fn formatting_an_already_formatted_document_is_idempotent() {
    let source = r#"parts ["Melody": notes, "Bass": notes],

score """
    [Melody] 1 2 3 4
    [Bass] 5 6 7 1
    """,
"#;
    let once = format_score(source);
    let twice = format_score(&once);
    assert_eq!(once, twice);
}

#[test]
fn formatted_output_still_parses_and_compiles_correctly() {
    let source = r#"parts ["Melody": notes, "Bass": notes],

score """
    [Melody]   1  2   3  4
    [Bass]  5 6 7 1

    [Melody] 0 0 0 0
    [Bass] 3 3 3 3
    """,
"#;
    let formatted = format_score(source);
    assert!(!formatted.contains("[Melody] 0 0 0 0"));

    // Strip the embedded-source `<metadata>` tag before comparing: it
    // legitimately differs (it round-trips the exact `.poentaro` text), but
    // everything else in the SVG (measures, note heads, layout) must match.
    fn strip_embedded_metadata(svg: &str) -> String {
        let start = svg.find("<metadata").unwrap();
        let end = svg.find("</metadata>").unwrap() + "</metadata>".len();
        format!("{}{}", &svg[..start], &svg[end..])
    }
    let original_svgs = crate::render_svgs_from_source(source, "test.poentaro", &[]).unwrap();
    let formatted_svgs = crate::render_svgs_from_source(&formatted, "test.poentaro", &[]).unwrap();
    let strip_all = |svgs: Vec<String>| -> Vec<String> {
        svgs.iter()
            .map(|svg| strip_embedded_metadata(svg))
            .collect()
    };
    assert_eq!(
        strip_all(original_svgs.svgs),
        strip_all(formatted_svgs.svgs),
        "dropping the redundant rest line must not change rendered output"
    );
}
