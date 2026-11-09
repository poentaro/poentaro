#![allow(clippy::disallowed_macros)]
use poentaro::cli::check;
use std::fs;
use std::path::Path;

#[test]
fn check_succeeds_on_valid_file() {
    let input_path = "/tmp/test_check_valid.poentaro";
    fs::write(
        input_path,
        r#"parts ["Melody": notes],

score """
    [Melody] 1 2 3 4
    """,
"#,
    )
    .unwrap();

    let outcome = check(Path::new(input_path)).unwrap();

    fs::remove_file(input_path).ok();
    assert!(
        outcome.ok,
        "expected success, got: {:?}",
        outcome.diagnostics
    );
}

#[test]
fn check_fails_on_recoverable_error() {
    let input_path = "/tmp/test_check_recoverable_error.poentaro";
    fs::write(
        input_path,
        r#"parts ["Melody": notes],

score """
    [Unknown] 1 2 3 4
    """,
"#,
    )
    .unwrap();

    let outcome = check(Path::new(input_path)).unwrap();

    fs::remove_file(input_path).ok();
    assert!(
        !outcome.ok,
        "expected failure for unrecognised abbreviation"
    );
    assert!(outcome
        .diagnostics
        .iter()
        .any(|d| d.message().contains("Unknown")));
}

#[test]
fn check_fails_on_irrecoverable_error() {
    let input_path = "/tmp/test_check_irrecoverable_error.poentaro";
    fs::write(
        input_path,
        r#"score """
    [Melody] 1 2 3 4
    """
"#,
    )
    .unwrap();

    let outcome = check(Path::new(input_path)).unwrap();

    fs::remove_file(input_path).ok();
    assert!(!outcome.ok, "expected failure for missing parts section");
}
