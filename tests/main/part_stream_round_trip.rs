#![allow(clippy::panic, clippy::disallowed_macros)]

use std::path::{Path, PathBuf};

use poentaro::part_stream::{
    apply_part_stream, extract_part_stream, list_streamable_parts, StreamDiagnosticSeverity,
    StreamablePartKind,
};

/// Fewer than this many round-tripped parts (notes, chords, percussion,
/// follow and lyric parts) means file discovery or eligibility silently broke.
const MINIMUM_CHECKED_PART_COUNT: usize = 35;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn poentaro_files_in(directory: &Path, recursive: bool) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .flat_map(|path| {
            if path.is_dir() && recursive {
                poentaro_files_in(&path, true)
            } else if path
                .extension()
                .is_some_and(|extension| extension == "poentaro")
            {
                vec![path]
            } else {
                Vec::new()
            }
        })
        .collect();
    files.sort();
    files
}

fn corpus() -> Vec<PathBuf> {
    let root = repository_root();
    let mut files = poentaro_files_in(&root, false);
    for directory in ["fixtures", "scores", "demo"] {
        files.extend(poentaro_files_in(&root.join(directory), true));
    }
    files
}

/// Round-trips one part, returning a failure description if it changes anything.
fn round_trip_failure(source: &str, abbreviation: &str) -> Option<String> {
    let stream = match extract_part_stream(source, abbreviation) {
        Ok(stream) => stream,
        Err(error) => return Some(format!("extraction failed: {error:?}")),
    };
    let result = apply_part_stream(source, abbreviation, &stream.text);
    let errors: Vec<&str> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == StreamDiagnosticSeverity::Error)
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    if !errors.is_empty() {
        return Some(format!("errors {errors:?} for stream:\n{}", stream.text));
    }
    if result.changed_measure_count != 0 {
        return Some(format!("{} measures changed", result.changed_measure_count));
    }
    if result.measure_starts != stream.measure_starts {
        return Some("measure starts moved".to_string());
    }
    (result.source.as_deref() != Some(source)).then(|| "source changed".to_string())
}

#[test]
fn every_eligible_part_in_the_repository_round_trips_unchanged() {
    let files = corpus();
    assert!(!files.is_empty(), "no .poentaro files found");
    let mut checked_part_count = 0;
    let mut checked_kinds = Vec::new();
    let mut failures = Vec::new();
    for path in &files {
        let source =
            std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
        for part in list_streamable_parts(&source) {
            if part.ineligible_reason.is_some() {
                continue;
            }
            checked_part_count += 1;
            checked_kinds.push(part.kind);
            if let Some(failure) = round_trip_failure(&source, &part.abbreviation) {
                failures.push(format!(
                    "{} [{}]: {failure}",
                    path.display(),
                    part.abbreviation
                ));
            }
        }
    }
    println!(
        "round-tripped {checked_part_count} parts across {} files",
        files.len()
    );
    assert!(
        failures.is_empty(),
        "round trip failures:\n{}",
        failures.join("\n")
    );
    assert!(
        checked_part_count >= MINIMUM_CHECKED_PART_COUNT,
        "only {checked_part_count} parts were eligible"
    );
    for kind in [StreamablePartKind::Follow, StreamablePartKind::Lyrics] {
        assert!(
            checked_kinds.contains(&kind),
            "no {kind:?} part was round-tripped"
        );
    }
}
