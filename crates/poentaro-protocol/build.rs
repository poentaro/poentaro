//! Generates `api`, the web app's API, from the anyx core schema
//! `schemas/api/api.anyx` (`anyx-core-generate`), as the module `api` in
//! `OUT_DIR/api.rs` and its client in `OUT_DIR/api.ts`.
//!
//! Generates each other audience's server traits, codec and client from its
//! entry file in `schemas/`, as the module `<audience>` in
//! `OUT_DIR/<audience>.rs`, which `lib.rs` includes. The module wraps the
//! generated code here because `include!` refuses its `#![allow(...)]`.
//! The TypeScript client goes to `OUT_DIR/<audience>.ts`, for the
//! `poentaro-protocol-typescript` binary to hand to the web app.
//!
//! Both carry `SCHEMA_HASH`, a hash of the schema's files: a client sends it
//! as `Poentaro-Schema`, so the worker can tell a tab built from another
//! schema to reload.

use std::path::{Path, PathBuf};

use anyx_protocol::Schema;

const AUDIENCES: [&str; 2] = ["agent", "test"];

fn main() {
    let schemas = Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas");
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    println!("cargo::rerun-if-changed={}", schemas.display());
    generate_api(&schemas.join("api/api.anyx"), &out_dir);
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

/// The web app's API, from its core schema.
fn generate_api(path: &Path, out_dir: &Path) {
    let source = std::fs::read_to_string(path).expect("schemas/api/api.anyx is readable");
    let schema = anyx_core_generate::load(&source, "schemas/api/api.anyx")
        .unwrap_or_else(|error| panic!("{error}"));
    let fail = |error: anyx_core_generate::Error| panic!("schemas/api/api.anyx: {error}");
    let hash = hash_bytes(source.as_bytes());
    let rust = anyx_core_generate::rust(&schema).unwrap_or_else(fail);
    let module = format!(
        "#[rustfmt::skip]\n#[allow(dead_code, unused_imports, clippy::all, clippy::pedantic)]\npub mod api {{\n{rust}\n\n/// A hash of the schema, sent as `Poentaro-Schema`.\npub const SCHEMA_HASH: &str = \"{hash}\";\n}}\n"
    );
    std::fs::write(out_dir.join("api.rs"), module).expect("OUT_DIR is writable");
    let typescript = anyx_core_generate::typescript(&schema).unwrap_or_else(fail);
    std::fs::write(
        out_dir.join("api.ts"),
        format!(
            "{typescript}\n/** A hash of the schema, sent as `Poentaro-Schema`. */\nexport const SCHEMA_HASH = \"{hash}\";\n"
        ),
    )
    .expect("OUT_DIR is writable");
}

fn hash_bytes(bytes: &[u8]) -> String {
    let hash = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
    });
    format!("{hash:016x}")
}

struct Generated {
    rust: String,
    typescript: String,
    hash: String,
}

/// Generate from an explicit entry, embedding only its imported files.
fn generate(folder: &Path, audience: &str) -> Generated {
    let mut sources = std::collections::BTreeMap::new();
    let schema = Schema::load("protocol.protocol.anyx", |path| {
        let source = std::fs::read_to_string(folder.join(path))?;
        println!("cargo::rerun-if-changed={}", folder.join(path).display());
        sources.insert(path.to_string(), source.clone());
        Ok::<_, std::io::Error>(source)
    })
    .unwrap_or_else(|diagnostics| {
        let rendered: Vec<String> = diagnostics
            .iter()
            .map(|diagnostic| {
                let source = sources.get(&diagnostic.path).map_or("", String::as_str);
                diagnostic.render(source, &format!("schemas/{audience}/{}", diagnostic.path))
            })
            .collect();
        panic!("{}", rendered.join("\n"))
    });
    let fail = |error: anyx_protocol_generate::Error| panic!("schemas/{audience}: {error}");
    Generated {
        rust: anyx_protocol_generate::rust(&schema).unwrap_or_else(fail),
        typescript: anyx_protocol_generate::typescript(&schema).unwrap_or_else(fail),
        hash: hash(&schema.files),
    }
}

/// FNV-1a over each file's path and source, in hex. Written out here rather
/// than taken from `std`, whose hashers may change between Rust releases.
fn hash(files: &[anyx_protocol::schema::SourceFile]) -> String {
    let hash = files
        .iter()
        .flat_map(|file| [file.path.as_bytes(), &[0], file.source.as_bytes(), &[0]])
        .flatten()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
        });
    format!("{hash:016x}")
}
