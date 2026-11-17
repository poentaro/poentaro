#![allow(clippy::panic, clippy::disallowed_macros)]

#[test]
fn new_file_template_parses_and_renders() {
    let source = include_str!("../../new_file_template.poentaro");
    let svgs = poentaro::render_svgs_from_source(source, "new_file_template.poentaro", &[])
        .unwrap_or_else(|e| {
            panic!("new_file_template.poentaro failed to parse/render: {e}");
        })
        .svgs;
    assert!(
        !svgs.is_empty(),
        "new_file_template.poentaro should produce at least one SVG page"
    );
    assert!(
        svgs.iter()
            .all(|svg| svg.starts_with("<svg") && svg.ends_with("</svg>")),
        "new_file_template.poentaro SVG output should be well-formed"
    );
}

#[test]
fn new_file_template_has_no_recoverable_errors() {
    use poentaro::error::Diagnostic;
    let source = include_str!("../../new_file_template.poentaro");
    let output = poentaro::render_svgs_from_source(source, "new_file_template.poentaro", &[])
        .unwrap_or_else(|e| panic!("new_file_template.poentaro failed to parse/render: {e}"));
    let errors: Vec<_> = output
        .diagnostics
        .iter()
        .filter(|d| matches!(d, Diagnostic::Error(_)))
        .map(|d| d.message())
        .collect();
    assert!(
        errors.is_empty(),
        "new_file_template.poentaro should have no recoverable errors, got: {errors:?}"
    );
}
