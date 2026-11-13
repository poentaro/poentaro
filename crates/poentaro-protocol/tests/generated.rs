//! `src/generated.rs` is written from the schema in `schema/`, a file per
//! scope. Run with `UPDATE_GENERATED=1` (`just generate-protocol`) to write
//! it again.

use anyx_protocol::{Schema, SchemaFile};

const SCHEMA_FOLDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/schema");
const GENERATED_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/generated.rs");

/// Each scope's file, by its path in `schema/`.
const SCHEMA_PATHS: [&str; 4] = [
    "protocol.anyx",
    "files.anyx",
    "files/export.anyx",
    "score.anyx",
];

#[test]
fn the_server_and_client_are_generated_from_the_schema() {
    let sources: Vec<String> = SCHEMA_PATHS
        .iter()
        .map(|path| {
            std::fs::read_to_string(format!("{SCHEMA_FOLDER}/{path}"))
                .unwrap_or_else(|error| panic!("schema/{path}: {error}"))
        })
        .collect();
    let files: Vec<SchemaFile> = SCHEMA_PATHS
        .iter()
        .zip(&sources)
        .map(|(path, source)| SchemaFile { path, source })
        .collect();
    let schema = Schema::parse_files(&files).unwrap_or_else(|diagnostics| {
        let rendered = diagnostics
            .iter()
            .map(|diagnostic| {
                let index = SCHEMA_PATHS
                    .iter()
                    .position(|path| *path == diagnostic.path)
                    .unwrap_or(0);
                diagnostic.render(&sources[index], &format!("schema/{}", diagnostic.path))
            })
            .collect::<Vec<_>>();
        panic!("{}", rendered.join("\n"))
    });
    let code = anyx_protocol_generate::rust(&schema).unwrap_or_else(|error| panic!("{error}"));
    let expected = anyx_protocol_generate::format_rust(&code).expect("rustfmt is installed");
    if std::env::var_os("UPDATE_GENERATED").is_some() {
        std::fs::write(GENERATED_PATH, &expected).expect("generated.rs is writable");
    }
    let committed = std::fs::read_to_string(GENERATED_PATH).unwrap_or_default();
    assert!(
        committed == expected,
        "src/generated.rs is not what schema/ generates; run `just generate-protocol`"
    );
}

#[test]
fn every_schema_file_is_read() {
    let mut found = Vec::new();
    collect(std::path::Path::new(SCHEMA_FOLDER), &mut found);
    found.sort();
    let mut expected: Vec<String> = SCHEMA_PATHS.iter().map(|path| path.to_string()).collect();
    expected.sort();
    assert_eq!(found, expected, "add the new file to SCHEMA_PATHS");
}

fn collect(folder: &std::path::Path, found: &mut Vec<String>) {
    for entry in std::fs::read_dir(folder).expect("schema/ is readable") {
        let path = entry.expect("schema/ is readable").path();
        if path.is_dir() {
            collect(&path, found);
        } else {
            let relative = path.strip_prefix(SCHEMA_FOLDER).expect("under schema/");
            found.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
}
