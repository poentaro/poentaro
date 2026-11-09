//! Cucumber harness for the unified per-kind text style metadata syntax
//! (see `tests/features/text_style_metadata_syntax.feature`):
//! `<kind> = { font_size: N, horizontal_padding_pt: N, vertical_padding_pt: N }`
//! replacing the old flat per-component keys (`lyrics_font_size`,
//! `lyric_click_target_padding_pt`, `notes_horizontal_padding_pt`, etc.)
//! with one object-valued key per text kind. `part_label_width_pt` is a
//! separate flat scalar field (not part of any `TextStyle` object), since
//! it's a layout constant rather than a text style component.
//!
//! Each scenario's `metadata` lines are spliced into a minimal otherwise-valid
//! `.poentaro` document and run through the public `compile` entry point, so
//! "then" steps can read the fully-resolved `Metadata` (`score.metadata`)
//! directly, rather than only checking that parsing produced no error.
//!
//! Clippy's `allow-*-in-tests` (clippy.toml) only recognizes `#[test]`-
//! attributed functions as test code; cucumber's `#[given]`/`#[when]`/
//! `#[then]` step functions don't qualify even though this whole file only
//! ever runs under `cargo test`. Mirrors `tests/cucumber.rs`'s
//! `#![allow(clippy::disallowed_macros)]` for the same reason.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::disallowed_macros,
    clippy::needless_pass_by_value
)]

#[path = "support/metadata_field.rs"]
mod metadata_field;

use cucumber::{given, then, when, World as _};
use metadata_field::metadata_field;
use poentaro::ast::grouped::{Metadata, TextStyle};
use poentaro::compile;

#[derive(Debug, Default, cucumber::World)]
struct TextStyleWorld {
    metadata_lines: Vec<String>,
    metadata: Option<Metadata>,
    errors: Vec<String>,
}

#[given(expr = "{string} sets {string} to {string}")]
fn given_metadata_line(world: &mut TextStyleWorld, _section: String, key: String, value: String) {
    world.metadata_lines.push(metadata_field(&key, &value));
}

#[when(expr = "it is compiled")]
fn when_compiled(world: &mut TextStyleWorld) {
    let metadata_block = world.metadata_lines.join(", ");
    let source = format!(
        "metadata {{ {metadata_block} }},\n\nparts [\"Melody\" : notes],\n\nscore \"\"\"\n[Melody] 1 2 3 4\n\"\"\",\n"
    );
    let score =
        compile(&source, "test.poentaro", &[]).unwrap_or_else(|err| panic!("{}", err.message()));
    world.errors = score
        .document_diagnostics
        .iter()
        .map(|d| d.message())
        .collect();
    world.metadata = Some(score.metadata);
}

/// Looks up one kind's resolved `TextStyle` on `metadata` by its `metadata`
/// key name (e.g. `"lyrics"`, `"part_label"`).
fn text_style_of<'a>(metadata: &'a Metadata, kind: &str) -> &'a TextStyle {
    match kind {
        "title" => &metadata.title_style,
        "subtitle" => &metadata.subtitle_style,
        "author" => &metadata.author_style,
        "sequence" => &metadata.sequence,
        "part_legend" => &metadata.part_legend,
        "measure_number" => &metadata.measure_number,
        "section_label" => &metadata.section_label,
        "page_number" => &metadata.page_number,
        "part_label" => &metadata.part_label,
        "lyrics" => &metadata.lyrics,
        "notes" => &metadata.notes,
        "chords" => &metadata.chords,
        "note_dash" => &metadata.note_dash,
        other => panic!("unrecognized text style kind {other:?}"),
    }
}

/// Reads one component off a resolved `TextStyle` by its `metadata`
/// object-literal field name (e.g. `"horizontal_padding_pt"`).
fn text_style_field(style: &TextStyle, field: &str) -> u32 {
    match field {
        "font_size" => style.font_size,
        "horizontal_padding_pt" => style.horizontal_padding_pt,
        "vertical_padding_pt" => style.vertical_padding_pt,
        other => panic!("unrecognized TextStyle field {other:?}"),
    }
}

#[then(expr = "the resolved {word} TextStyle has {word} equal to {int}")]
fn then_text_style_field(world: &mut TextStyleWorld, kind: String, field: String, value: i64) {
    assert!(
        world.errors.is_empty(),
        "expected `{kind}.{field}` to parse as recognized TextStyle syntax with no errors, got: {:?}",
        world.errors
    );
    let metadata = world.metadata.as_ref().expect("compiled score");
    let resolved = text_style_field(text_style_of(metadata, &kind), &field);
    assert_eq!(
        resolved, value as u32,
        "expected `{kind}.{field}` to resolve to {value}, got {resolved}"
    );
}

#[then(
    expr = "the resolved {word} TextStyle's font_size equals the default {word} font size for row_height {int}"
)]
fn then_default_font_size(
    world: &mut TextStyleWorld,
    kind: String,
    default_kind: String,
    row_height: i64,
) {
    assert!(
        world.errors.is_empty(),
        "expected `{kind}`'s unset components to fall back to their defaults with no parse errors, got: {:?}",
        world.errors
    );
    assert_eq!(
        kind, default_kind,
        "step wording mismatch: kind and default_kind should always match"
    );
    let metadata = world.metadata.as_ref().expect("compiled score");
    let resolved = text_style_of(metadata, &kind).font_size;
    let expected = (row_height as f64 * 0.6).round() as u32;
    assert_eq!(
        resolved, expected,
        "expected `{kind}.font_size` to default to {expected} at row_height {row_height}, got {resolved}"
    );
}

#[then(expr = "compiling reports an unknown metadata field {string}")]
fn then_unknown_field(world: &mut TextStyleWorld, field: String) {
    assert!(
        world.errors.iter().any(|e| e.contains(&field)),
        "expected an \"unknown metadata field\" error mentioning {field:?}, got: {:?}",
        world.errors
    );
}

#[then(expr = "compiling reports a metadata parse error on the {string} line")]
fn then_parse_error(world: &mut TextStyleWorld, key: String) {
    assert!(
        !world.errors.is_empty(),
        "expected a metadata parse error on the {key:?} line, got no errors"
    );
}

#[tokio::main]
async fn main() {
    TextStyleWorld::run("tests/features/text_style_metadata_syntax.feature").await;
}
