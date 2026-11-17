#![allow(clippy::disallowed_macros)]
use poentaro::error::Diagnostic;
use poentaro::render_svgs_from_source;

#[path = "recoverable_directive_errors/duration_and_group_errors.rs"]
mod duration_and_group_errors;

#[path = "recoverable_directive_errors/sequence_errors.rs"]
mod sequence_errors;

pub(crate) fn minimal_fixture(score_section: &str) -> String {
    format!(
        r#"metadata {{ title "t", author "a" }},

parts ["Melody": notes],

score """
    {score_section}
    """,
"#
    )
}

pub(crate) fn has_error_containing(output: &poentaro::RenderOutput, keyword: &str) -> bool {
    output
        .diagnostics
        .iter()
        .any(|d| matches!(d, Diagnostic::Error(_)) && d.message().contains(keyword))
}

// Tie/slur group at measure start (motivating case for removing directive parens)

#[test]
fn measure_starting_with_paren_group_is_not_a_directive() {
    let source = minimal_fixture("[Melody] (1 2) 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("measure starting with tie group must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        output
            .diagnostics
            .iter()
            .all(|d| !matches!(d, Diagnostic::Error(_))),
        "expected no errors, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// Group 1a — Directive row errors (whole-row skip)

#[test]
fn directive_unclosed_quote_is_recoverable() {
    let source = minimal_fixture("label=\"unterminated\n[Melody] 1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("unclosed quote must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "unclosed quote"),
        "expected error about unclosed quote, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// Group 1b — Per-token directive errors

#[test]
fn directive_invalid_bpm_is_recoverable() {
    let source = minimal_fixture("time=4/4 key=C4 bpm=abc\n[Melody] 1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("invalid bpm must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "invalid bpm"),
        "expected error about invalid bpm, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn directive_label_not_quoted_is_recoverable() {
    let source = minimal_fixture("time=4/4 key=C4 bpm=120 label=unquoted\n[Melody] 1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("unquoted label must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "quoted string"),
        "expected error about quoted string, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn directive_empty_label_is_recoverable() {
    let source = minimal_fixture("time=4/4 key=C4 bpm=120 label=\"\"\n[Melody] 1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("empty label must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "must not be empty"),
        "expected error about empty label, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn directive_unknown_token_is_recoverable() {
    let source = minimal_fixture("time=4/4 key=C4 bpm=120 unknown_token\n[Melody] 1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("unknown directive token must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "unknown directive"),
        "expected error about unknown directive, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn directive_key_missing_note_name_is_recoverable() {
    let source = minimal_fixture("time=4/4 key=4 bpm=120\n[Melody] 1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("key= with digit only must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "invalid note name")
            || has_error_containing(&output, "expected note name"),
        "expected error about missing note name, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn directive_key_invalid_note_letter_is_recoverable() {
    let source = minimal_fixture("time=4/4 key=Z4 bpm=120\n[Melody] 1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("key= with invalid note letter must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "invalid note name"),
        "expected error about invalid note name, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn directive_time_not_fraction_is_recoverable() {
    let source = minimal_fixture("time=abc key=C4 bpm=120\n[Melody] 1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("time= not in N/D form must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "invalid time"),
        "expected error about invalid time signature, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn directive_time_numerator_too_large_is_recoverable() {
    let source = minimal_fixture("time=999/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("time numerator too large must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "invalid time numerator"),
        "expected error about invalid time numerator, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn directive_time_zero_denominator_is_recoverable() {
    let source = minimal_fixture("time=4/0 key=C4 bpm=120\n[Melody] 1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("time denominator zero must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "zero"),
        "expected error about zero denominator, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// Group 2 — Inline timed-lexer errors (in notes line)

#[test]
fn inline_bpm_invalid_is_recoverable() {
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\n[Melody] bpm=abc 1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("inline invalid bpm must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "bpm="),
        "expected error about bpm=, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn inline_time_zero_denominator_is_recoverable() {
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\n[Melody] 4/0 1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("inline time 4/0 must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "zero"),
        "expected error about zero denominator, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn debug_label_only_measure() {
    let source = minimal_fixture("label=\"hello\"\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[]);
    match &output {
        Ok(o) => {
            eprintln!(
                "DIAGS: {:?}",
                o.diagnostics
                    .iter()
                    .map(|d| d.message())
                    .collect::<Vec<_>>()
            );
        }
        Err(e) => eprintln!("ERR: {e:?}"),
    }
}
