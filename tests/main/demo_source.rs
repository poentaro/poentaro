#![allow(clippy::panic, clippy::disallowed_macros)]

/// Every numbered `.poentaro` fragment under `demo/`, sorted by filename —
/// each one is a complete, standalone-renderable document (its own
/// metadata/parts/score), shown to users as a folder of individually
/// selectable demo files in the web editor.
fn demo_file_paths() -> Vec<std::path::PathBuf> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("demo");
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("failed to read demo dir {dir:?}: {e}"))
        .map(|entry| {
            entry
                .unwrap_or_else(|e| panic!("failed to read demo dir entry: {e}"))
                .path()
        })
        .collect();
    paths.sort();
    paths
}

fn render_demo_file(path: &std::path::Path) -> poentaro::RenderOutput {
    let source =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {path:?}: {e}"));
    let errors = poentaro::document::anyx_errors(&source, &[]);
    assert!(errors.is_empty(), "{path:?}: {errors:?}");
    poentaro::render_svgs_from_source(&source, "demo.poentaro", &[])
        .unwrap_or_else(|e| panic!("{path:?} failed to parse/render: {e}"))
}

#[test]
fn demo_files_parse_and_render() {
    for path in demo_file_paths() {
        let svgs = render_demo_file(&path).svgs;
        assert!(
            !svgs.is_empty(),
            "{path:?} should produce at least one SVG page"
        );
        assert!(
            svgs.iter()
                .all(|svg| svg.starts_with("<svg") && svg.ends_with("</svg>")),
            "{path:?} SVG output should be well-formed"
        );
    }
}

#[test]
fn demo_files_have_no_diagnostics() {
    for path in demo_file_paths() {
        let output = render_demo_file(&path);
        assert!(
            output.diagnostics.is_empty(),
            "{path:?} should have no errors or warnings, got: {:?}",
            output
                .diagnostics
                .iter()
                .map(|d| d.message())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn demo_files_render_expected_content() {
    let combined: String = demo_file_paths()
        .iter()
        .map(|path| render_demo_file(path).svgs.join(""))
        .collect();
    assert!(
        combined.contains('春'),
        "demo files should render CJK lyrics"
    );
    assert!(
        combined.contains("1m"),
        "demo files should render minor chord symbols"
    );
}
