//! Finds `pub` items in `src/` that no other crate in the workspace actually
//! references — the blind spot `#![deny(dead_code)]` can't cover, because an
//! item that's `pub` (and re-exported up to the crate root) counts as
//! reachable "public API" to rustc even when nothing calls it.
//!
//! This walks real syntax trees (via `syn`), not source text, so it
//! correctly ignores identifiers that only appear in comments/doc-links or
//! string literals, and correctly excludes `pub use` re-export declarations
//! and `#[cfg(test)]`/`#[test]` code from counting as "real" usage.
//!
//! It is still a name-based heuristic, not full type-resolved reachability:
//! two unrelated items sharing an identifier (e.g. two `pub fn new`) can mask
//! each other. Mark a deliberately-unused-for-now item with
//! `#[allow(dead_code)]` to exempt it.

mod attrs;
mod mod_tree;
mod pub_defs;
mod usage;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use mod_tree::{collect_rs_files, walk_mod_tree};
use pub_defs::{collect_pub_defs, PubDef};
use syn::visit::Visit;
use usage::UsageVisitor;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("dead-pub-check must live at <repo-root>/tools/<name>")
        .to_path_buf();

    let def_scan_root = root.join("src");
    let usage_scan_roots = [
        root.join("src"),
        root.join("crates/poentaro-wasm/src"),
        root.join("tests"),
    ];

    let mut all_files = Vec::new();
    for r in &usage_scan_roots {
        collect_rs_files(r, &mut all_files);
    }

    let mut parsed_cache: HashMap<PathBuf, syn::File> = HashMap::new();
    for file in &all_files {
        let Ok(content) = fs::read_to_string(file) else {
            continue;
        };
        let Ok(parsed) = syn::parse_file(&content) else {
            eprintln!(
                "dead-pub-check: skipping unparsable file {}",
                file.display()
            );
            continue;
        };
        parsed_cache.insert(file.clone(), parsed);
    }

    let mut test_only_files: HashSet<PathBuf> = HashSet::new();
    for crate_root in [
        root.join("src/lib.rs"),
        root.join("crates/poentaro-wasm/src/lib.rs"),
    ] {
        let mut visited = HashSet::new();
        walk_mod_tree(
            &crate_root,
            false,
            &parsed_cache,
            &mut test_only_files,
            &mut visited,
        );
    }

    let mut def_files = Vec::new();
    collect_rs_files(&def_scan_root, &mut def_files);
    let mut pub_defs = Vec::new();
    for file in &def_files {
        if test_only_files.contains(file) {
            continue;
        }
        if let Some(parsed) = parsed_cache.get(file) {
            collect_pub_defs(&parsed.items, file, false, &mut pub_defs);
        }
    }

    let mut visitor = UsageVisitor {
        counts: HashMap::new(),
        test_depth: 0,
    };
    for file in &all_files {
        if test_only_files.contains(file) {
            continue;
        }
        if let Some(parsed) = parsed_cache.get(file) {
            visitor.visit_file(parsed);
        }
    }

    let mut dead: Vec<&PubDef> = pub_defs
        .iter()
        .filter(|def| visitor.counts.get(&def.name).copied().unwrap_or(0) <= 1)
        .collect();
    dead.sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));

    if dead.is_empty() {
        println!("dead-pub-check: no unreferenced `pub` items found.");
        return;
    }

    eprintln!(
        "dead-pub-check found {} possibly-dead `pub` item(s):\n",
        dead.len()
    );
    for def in &dead {
        eprintln!(
            "  {}:{}: `{}` is never referenced outside its own definition/re-exports",
            def.file.display(),
            def.line,
            def.name
        );
    }
    eprintln!(
        "\nThis is a name-based heuristic, not full type-resolved reachability: a match \
         elsewhere with the same identifier (e.g. another `pub fn new`) suppresses the \
         warning, so a clean run doesn't prove there's no dead code. If an item above is \
         genuinely dead, remove it. If it's real public API not yet consumed, mark it \
         `#[allow(dead_code)]` with a comment saying why."
    );
    std::process::exit(1);
}
