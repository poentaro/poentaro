use super::*;

/// Every numbered `.poentaro` fragment under `demo/` (sorted by filename) —
/// each one is a complete, standalone-renderable document (its own
/// metadata/parts/score), shown to users as a folder of individually
/// selectable demo files in the web editor.
fn demo_file_paths() -> Vec<std::path::PathBuf> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../demo");
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

fn read_demo_file(path: &std::path::Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {path:?}: {e}"))
}

#[path = "tests_parts_and_measures.rs"]
mod tests_parts_and_measures;

#[cfg(feature = "pdf")]
#[path = "tests_pdf.rs"]
mod tests_pdf;

#[path = "tests_render.rs"]
mod tests_render;
#[path = "tests_render_invalid_part.rs"]
mod tests_render_invalid_part;

#[path = "tests_folder_share.rs"]
mod tests_folder_share;

#[path = "tests_share.rs"]
mod tests_share;

#[cfg(feature = "wav")]
#[path = "tests_wav.rs"]
mod tests_wav;

#[path = "group_diagnostics_tests.rs"]
mod group_diagnostics_tests;
