use super::format_anyx;

const PARTS: &str =
    r#"parts [ "Soprano" [S] : notes, "Alto" [A] : notes, "Words" [Sv1] : lyrics [S] ],"#;

#[test]
fn lays_out_the_document_then_formats_score_strings() {
    let source = [
        r#"metadata { title : "Song" },"#,
        PARTS,
        r#"score """
  label="Verse"
  [A]   5 6   7 1
  [Sv1] _
  [S] 1  2 3 4
  ""","#,
    ]
    .join("\n");
    assert_eq!(
        format_anyx(&source),
        r#"metadata { title "Song" },
parts ["Soprano" [S]: notes, "Alto" [A]: notes, "Words" [Sv1]: lyrics [S]],
score """
    label="Verse"
    [S] 1 2 3 4
    [A] 5 6 7 1
    """,
"#
    );
}

#[test]
fn formatting_twice_changes_nothing() {
    let source = format!("{PARTS}\nscore \"\"\"\n  [A] 5 6 7 1\n  [S] 1 2 3 4\n  \"\"\"");
    let once = format_anyx(&source);
    assert_eq!(format_anyx(&once), once);
}

#[test]
fn a_score_string_with_a_comment_keeps_its_measures() {
    let source =
        format!("{PARTS}\nscore \"\"\"\n  [A] 5   6 7 1 // alto first\n  [S] 1 2 3 4\n  \"\"\"");
    assert!(format_anyx(&source).contains("[A] 5   6 7 1 // alto first\n"));
}

#[test]
fn a_document_that_does_not_parse_comes_back_unchanged() {
    assert_eq!(
        format_anyx("parts [ \"S\" : notes"),
        "parts [ \"S\" : notes"
    );
}
