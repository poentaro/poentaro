//! Generates `src/fonts.rs`'s actual constants from `fonts/fonts.json` (the
//! single source of truth for which font file backs each `FontFamily`
//! role — see that file's own comments for the full picture, including the
//! Serif-vs-alias asymmetry). Output lands in `OUT_DIR/fonts_generated.rs`,
//! `include!`-d by `src/fonts.rs`.
//!
//! Also generates `src/pitch_description/guitar_voicing.rs`'s lookup table
//! from the vendored `vendor/chords-db/guitar.json` (see
//! `emit_guitar_voicings`), keeping only each position's frets, base fret
//! and barres so the wasm binary carries a compact table instead of the
//! whole JSON.
//!
//! Also generates `src/file_tree/extension.rs`'s constants from
//! `file-extension.json`, which the web app imports too, so both sides name
//! files with the same extension.

use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileExtensionManifest {
    extension: String,
    legacy_extension: String,
}

fn emit_file_extension(manifest_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
    let path = PathBuf::from(manifest_dir).join("file-extension.json");
    let manifest: FileExtensionManifest = serde_json::from_str(&fs::read_to_string(path)?)?;
    Ok(format!(
        "/// Every file the app saves ends in this.\n\
         pub const FILE_EXTENSION: &str = {:?};\n\
         /// What files ended in before the app was renamed to Poentaro.\n\
         pub const LEGACY_FILE_EXTENSION: &str = {:?};\n",
        manifest.extension, manifest.legacy_extension
    ))
}

#[derive(Deserialize)]
struct FontEntry {
    filename: String,
    name: Option<String>,
    #[serde(rename = "familyCss")]
    family_css: String,
}

#[derive(Deserialize)]
struct FontsManifest {
    serif: FontEntry,
    #[serde(rename = "sansSerif")]
    sans_serif: FontEntry,
    monospace: FontEntry,
}

/// Emits one role's constants into `out`. `bytes_const_doc` documents the
/// `_FONT_BYTES` const; `name_const` is `None` for the `serif` role, which
/// has no `fontdb` generic-alias name (see the manifest's own comment).
fn emit_role(
    out: &mut String,
    manifest_dir: &str,
    role_prefix: &str,
    entry: &FontEntry,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(name) = &entry.name {
        out.push_str(&format!(
            "pub const {role_prefix}_FONT_NAME: &str = {name:?};\n"
        ));
    }
    out.push_str(&format!(
        "pub const {role_prefix}_FONT_FAMILY_CSS: &str = {family_css:?};\n",
        family_css = entry.family_css,
    ));
    let absolute_font_path = PathBuf::from(manifest_dir)
        .join("fonts")
        .join(&entry.filename);
    let absolute_font_path_str = absolute_font_path
        .to_str()
        .ok_or("font path is not valid UTF-8")?;
    out.push_str("#[cfg(not(target_arch = \"wasm32\"))]\n");
    out.push_str(&format!(
        "pub const {role_prefix}_FONT_BYTES: &[u8] = include_bytes!({absolute_font_path_str:?});\n"
    ));
    Ok(())
}

#[derive(Deserialize)]
struct ChordsDb {
    chords: std::collections::BTreeMap<String, Vec<ChordsDbChord>>,
}

#[derive(Deserialize)]
struct ChordsDbChord {
    suffix: String,
    positions: Vec<ChordsDbPosition>,
}

#[derive(Deserialize)]
struct ChordsDbPosition {
    frets: Vec<i8>,
    #[serde(rename = "baseFret")]
    base_fret: u8,
    barres: Vec<u8>,
}

/// Maps a chords-db root key (`"Csharp"`, `"Eb"`, ...) to its pitch class
/// (C = 0).
fn chords_db_root_pitch_class(root: &str) -> Result<u8, Box<dyn std::error::Error>> {
    let pitch_class = match root {
        "C" => 0,
        "Csharp" => 1,
        "D" => 2,
        "Eb" => 3,
        "E" => 4,
        "F" => 5,
        "Fsharp" => 6,
        "G" => 7,
        "Ab" => 8,
        "A" => 9,
        "Bb" => 10,
        "B" => 11,
        other => return Err(format!("unknown chords-db root {other:?}").into()),
    };
    Ok(pitch_class)
}

/// Emits one `GuitarVoicingEntry` per chords-db chord position, in
/// chords-db's order (most common first) — `find_guitar_voicings` relies on
/// that order.
fn emit_guitar_voicings(manifest_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
    let json_path = PathBuf::from(manifest_dir).join("vendor/chords-db/guitar.json");
    let db: ChordsDb = serde_json::from_str(&fs::read_to_string(json_path)?)?;

    let mut out = String::from("pub(super) static GUITAR_VOICINGS: &[GuitarVoicingEntry] = &[\n");
    for (root, chords) in &db.chords {
        let root_pitch_class = chords_db_root_pitch_class(root)?;
        for chord in chords {
            for position in &chord.positions {
                out.push_str(&format!(
                    "    GuitarVoicingEntry {{ root_pitch_class: {root_pitch_class}, suffix: {suffix:?}, voicing: GuitarVoicing {{ frets: {frets:?}, base_fret: {base_fret}, barres: &{barres:?} }} }},\n",
                    suffix = chord.suffix,
                    frets = position.frets,
                    base_fret = position.base_fret,
                    barres = position.barres,
                ));
            }
        }
    }
    out.push_str("];\n");
    Ok(out)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=fonts/fonts.json");
    println!("cargo:rerun-if-changed=vendor/chords-db/guitar.json");
    println!("cargo:rerun-if-changed=file-extension.json");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let manifest_json_path = PathBuf::from(&manifest_dir).join("fonts/fonts.json");
    let manifest_json = fs::read_to_string(&manifest_json_path)?;
    let manifest: FontsManifest = serde_json::from_str(&manifest_json)?;

    let mut out = String::new();
    emit_role(&mut out, &manifest_dir, "SERIF", &manifest.serif)?;
    emit_role(&mut out, &manifest_dir, "SANS_SERIF", &manifest.sans_serif)?;
    emit_role(&mut out, &manifest_dir, "MONOSPACE", &manifest.monospace)?;

    let out_dir = env::var("OUT_DIR")?;
    let out_path = PathBuf::from(out_dir).join("fonts_generated.rs");
    fs::write(out_path, out)?;

    let guitar_voicings_path =
        PathBuf::from(env::var("OUT_DIR")?).join("guitar_voicings_generated.rs");
    fs::write(guitar_voicings_path, emit_guitar_voicings(&manifest_dir)?)?;

    let file_extension_path =
        PathBuf::from(env::var("OUT_DIR")?).join("file_extension_generated.rs");
    fs::write(file_extension_path, emit_file_extension(&manifest_dir)?)?;

    Ok(())
}
