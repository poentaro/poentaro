//! Generates each audience's server traits, codec and client from its
//! schema folder in `schemas/`, as the module `<audience>` in
//! `OUT_DIR/<audience>.rs`, which `lib.rs` includes. The module wraps the
//! generated code here because `include!` refuses its `#![allow(...)]`.
//! The TypeScript client goes to `OUT_DIR/<audience>.ts`, for the
//! `poentaro-protocol-typescript` binary to hand to the web app.
//!
//! Both carry `SCHEMA_HASH`, a hash of the schema's files: a client sends it
//! as `Poentaro-Schema`, so the worker can tell a tab built from another
//! schema to reload.

use std::path::{Path, PathBuf};

use anyx_protocol::{Schema, SchemaFile};

const AUDIENCES: [&str; 4] = ["agent", "app", "public", "test"];

fn main() {
    let schemas = Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas");
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    println!("cargo::rerun-if-changed={}", schemas.display());
    for audience in AUDIENCES {
        let generated = generate(&schemas.join(audience), audience);
        let hash = &generated.hash;
        let module = format!(
            "#[rustfmt::skip]\npub mod {audience} {{\n{}\n\n/// A hash of the schema's files, sent as `Poentaro-Schema`.\npub const SCHEMA_HASH: &str = \"{hash}\";\n}}\n",
            generated.rust
        );
        std::fs::write(out_dir.join(format!("{audience}.rs")), module)
            .expect("OUT_DIR is writable");
        let typescript = format!(
            "{}\n/** A hash of the schema's files, sent as `Poentaro-Schema`. */\nexport const SCHEMA_HASH = \"{hash}\";\n",
            generated.typescript
        );
        std::fs::write(out_dir.join(format!("{audience}.ts")), typescript)
            .expect("OUT_DIR is writable");
    }
}

struct Generated {
    rust: String,
    typescript: String,
    hash: String,
}

/// The code generated from every `.anyx` file under `folder`.
fn generate(folder: &Path, audience: &str) -> Generated {
    let mut paths = Vec::new();
    collect(folder, folder, &mut paths);
    paths.sort();
    let sources: Vec<String> = paths
        .iter()
        .map(|path| {
            println!("cargo::rerun-if-changed={}", folder.join(path).display());
            std::fs::read_to_string(folder.join(path))
                .unwrap_or_else(|error| panic!("schemas/{audience}/{path}: {error}"))
        })
        .collect();
    let files: Vec<SchemaFile> = paths
        .iter()
        .zip(&sources)
        .map(|(path, source)| SchemaFile { path, source })
        .collect();
    let schema = Schema::parse_files(&files).unwrap_or_else(|diagnostics| {
        let rendered = diagnostics
            .iter()
            .map(|diagnostic| {
                let index = paths
                    .iter()
                    .position(|path| *path == diagnostic.path)
                    .unwrap_or(0);
                diagnostic.render(
                    &sources[index],
                    &format!("schemas/{audience}/{}", diagnostic.path),
                )
            })
            .collect::<Vec<_>>();
        panic!("{}", rendered.join("\n"))
    });
    let fail = |error: anyx_protocol_generate::Error| panic!("schemas/{audience}: {error}");
    Generated {
        rust: anyx_protocol_generate::rust(&schema).unwrap_or_else(fail),
        typescript: anyx_protocol_generate::typescript(&schema).unwrap_or_else(fail),
        hash: hash(&files),
    }
}

/// FNV-1a over each file's path and source, in hex. Written out here rather
/// than taken from `std`, whose hashers may change between Rust releases.
fn hash(files: &[SchemaFile]) -> String {
    let hash = files
        .iter()
        .flat_map(|file| [file.path.as_bytes(), &[0], file.source.as_bytes(), &[0]])
        .flatten()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
        });
    format!("{hash:016x}")
}

/// Every `.anyx` file under `folder`, as a slash path relative to `root`.
fn collect(root: &Path, folder: &Path, found: &mut Vec<String>) {
    for entry in std::fs::read_dir(folder).expect("the schema folder is readable") {
        let path = entry.expect("the schema folder is readable").path();
        if path.is_dir() {
            collect(root, &path, found);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "anyx")
        {
            let relative = path.strip_prefix(root).expect("under the schema folder");
            found.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
}
