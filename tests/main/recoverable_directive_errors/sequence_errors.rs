use super::*;

// Group 1e — `sequence` section errors

fn fixture_with_sequence(score_section: &str, sequence: &str) -> String {
    format!(
        r#"metadata {{ title "t", author "a" }},

parts ["Melody": notes],

sequence ["{sequence}"],

score """
    {score_section}
    """,
"#
    )
}

#[test]
fn sequence_referencing_undefined_label_is_recoverable() {
    let source = fixture_with_sequence(
        "time=4/4 key=C4 bpm=120 label=\"A\"\n[Melody] 1 2 3 4\n",
        "A, Z",
    );
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("sequence referencing undefined label must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "undefined label"),
        "expected error about undefined label, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn sequence_with_duplicate_label_definitions_is_recoverable() {
    let source = fixture_with_sequence(
        concat!(
            "time=4/4 key=C4 bpm=120 label=\"A\"\n[Melody] 1 2 3 4\n\n",
            "label=\"A\"\n[Melody] 5 6 7 1\n",
        ),
        "A",
    );
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("duplicate label definitions must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "defined more than once"),
        "expected error about duplicate label definition, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}
