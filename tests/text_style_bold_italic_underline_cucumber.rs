//! Cucumber harness for
//! `tests/features/text_style_bold_italic_underline.feature`: verifies that
//! `bold`/`italic`/`underline` (see `text_style_metadata_syntax.feature`)
//! have a real rendering effect for every one of the 13 `TextStyle` kinds,
//! not just `notes` (the kind the original bug report confirmed working).
//!
//! Unlike `tests/text_style_rendering_cucumber.rs` (which drives the
//! pipeline stage-by-stage to inspect intermediate `Header`/`GridPage`
//! values), this file goes through the single public
//! `render_svgs_from_source` entry point and inspects the final serialized
//! SVG string directly — the same artifact a real viewer renders — since
//! what's under test here (do the style attributes reach the actual SVG
//! markup) is exactly what `crate::serializer` emits, and every kind
//! converges on that one serialization step regardless of which internal
//! path (plain `SvgKind::Text` vs. tspan-based `SvgKind::TextWithTspans`)
//! produced it.
//!
//! Each kind gets its own minimal `.poentaro` fixture in [`kind_fixture`]
//! producing one distinctively-named text element; the "Then" step locates
//! that element's `<text>`/`<tspan>` tag in the raw SVG and asserts its
//! attributes. Content strings are chosen to be unique within their own
//! fixture's render, but not necessarily globally (e.g. `notes`' note-head
//! digit and `measure_number`'s bar-number digit can coincide) — this is
//! safe because a `<tspan>` and a plain `<text>` are distinguished by tag,
//! and only the first matching tag in document order is used (see
//! `sequence`'s fixture, which deliberately renders the same label text
//! twice: once on the "Sequence: " summary line under test, and once as
//! the measure's own inline `label="..."` directive, styled independently
//! per `section_label` — the summary line renders first in document order).
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

/// One kind's minimal fixture: the `parts`/`score`/`sequence` blocks
/// needed to make that kind's text appear at all, plus the exact text
/// content to look for and whether it renders as a `<tspan>` (directive-line
/// content: `measure_number`, `section_label`, `sequence`) or a plain
/// `<text>` element (every other kind).
struct KindFixture {
    parts_block: &'static str,
    score_block: &'static str,
    sequence_block: &'static str,
    extra_metadata: &'static str,
    content: &'static str,
    is_tspan: bool,
}

/// First half of [`kind_fixture`]'s match — split purely to keep each
/// function under clippy's line-count cap, no grouping significance.
fn kind_fixture_header(kind: &str) -> Option<KindFixture> {
    Some(match kind {
        "title" => KindFixture {
            parts_block: "\"S\" : notes",
            score_block: "[S] 1\n",
            sequence_block: "",
            extra_metadata: "title \"TitleGlyph\"",
            content: "TitleGlyph",
            is_tspan: false,
        },
        "subtitle" => KindFixture {
            parts_block: "\"S\" : notes",
            score_block: "[S] 1\n",
            sequence_block: "",
            extra_metadata: "subtitle \"SubtitleGlyph\"",
            content: "SubtitleGlyph",
            is_tspan: false,
        },
        "author" => KindFixture {
            parts_block: "\"S\" : notes",
            score_block: "[S] 1\n",
            sequence_block: "",
            extra_metadata: "author \"AuthorGlyph\"",
            content: "AuthorGlyph",
            is_tspan: false,
        },
        "sequence" => KindFixture {
            parts_block: "\"S\" : notes",
            score_block: "time=4/4 key=C4 bpm=120 label=\"SeqLbl\"\n[S] 1 2 3 4\n",
            sequence_block: "sequence [SeqLbl],\n\n",
            extra_metadata: "",
            content: "SeqLbl",
            is_tspan: true,
        },
        "part_legend" => KindFixture {
            parts_block: "\"Violin\" [V] : notes",
            score_block: "[V] 1\n",
            sequence_block: "",
            extra_metadata: "",
            content: "V \u{2014} Violin",
            is_tspan: false,
        },
        "measure_number" => KindFixture {
            parts_block: "\"S\" : notes",
            score_block: "[S] 1\n",
            sequence_block: "",
            extra_metadata: "",
            content: "1",
            is_tspan: true,
        },
        "section_label" => KindFixture {
            parts_block: "\"S\" : notes",
            score_block: "time=4/4 key=C4 bpm=120 label=\"SecLbl\"\n[S] 1\n",
            sequence_block: "",
            extra_metadata: "",
            content: "SecLbl",
            is_tspan: true,
        },
        _ => return None,
    })
}

/// Second half of [`kind_fixture`]'s match — see [`kind_fixture_header`].
fn kind_fixture_body(kind: &str) -> KindFixture {
    match kind {
        "page_number" => KindFixture {
            parts_block: "\"S\" : notes",
            score_block: "[S] 1\n",
            sequence_block: "",
            extra_metadata: "",
            content: "1 / 1",
            is_tspan: false,
        },
        "part_label" => KindFixture {
            parts_block: "\"S\" : notes",
            score_block: "[S] 1\n",
            sequence_block: "",
            extra_metadata: "",
            content: "S",
            is_tspan: false,
        },
        "lyrics" => KindFixture {
            parts_block: "\"S\" : notes, \"L\" : lyrics [S]",
            score_block: "[S] 1\n[L] LyricWord\n",
            sequence_block: "",
            extra_metadata: "",
            content: "LyricWord",
            is_tspan: false,
        },
        "notes" => KindFixture {
            parts_block: "\"S\" : notes",
            score_block: "[S] 1\n",
            sequence_block: "",
            extra_metadata: "",
            content: "1",
            is_tspan: false,
        },
        "chords" => KindFixture {
            parts_block: "\"S\" : notes, \"C\" : chords",
            score_block: "[S] 5\n[C] 1\n",
            sequence_block: "",
            extra_metadata: "",
            content: "1",
            is_tspan: false,
        },
        "note_dash" => KindFixture {
            parts_block: "\"S\" : notes",
            score_block: "[S] 1-\n",
            sequence_block: "",
            extra_metadata: "",
            content: "\u{2014}",
            is_tspan: false,
        },
        other => panic!("unrecognized text style kind {other:?}"),
    }
}

fn kind_fixture(kind: &str) -> KindFixture {
    kind_fixture_header(kind).unwrap_or_else(|| kind_fixture_body(kind))
}

#[derive(Debug, Default, cucumber::World)]
struct BoldItalicUnderlineWorld {
    metadata_lines: Vec<String>,
    kind: String,
    svgs: Vec<String>,
}

#[given(expr = "{string} sets {string} to {string}")]
fn given_metadata_line(
    world: &mut BoldItalicUnderlineWorld,
    _section: String,
    key: String,
    value: String,
) {
    world.metadata_lines.push(metadata_field(&key, &value));
}

#[given(expr = "a minimal score exercising {word}")]
fn given_minimal_score_exercising_kind(world: &mut BoldItalicUnderlineWorld, kind: String) {
    world.kind = kind;
}

#[when(expr = "it is rendered to SVG")]
fn when_rendered_to_svg(world: &mut BoldItalicUnderlineWorld) {
    let fixture = kind_fixture(&world.kind);
    let mut metadata_lines = world.metadata_lines.clone();
    if !fixture.extra_metadata.is_empty() {
        metadata_lines.push(fixture.extra_metadata.to_string());
    }
    let metadata_block = metadata_lines.join(", ");
    let source = format!(
        "metadata {{ {metadata_block} }},\n\nparts [{}],\n\n{}score \"\"\"\n{}\"\"\",\n",
        fixture.parts_block, fixture.sequence_block, fixture.score_block
    );
    let result = poentaro::render_svgs_from_source(&source, "test.poentaro", &[])
        .unwrap_or_else(|err| panic!("compile returned an irrecoverable error: {}", err.message()));
    world.svgs = result.svgs;
}

/// Style attributes read off the first `<text>`/`<tspan>` tag whose body is
/// exactly `content`, searched in document order across all pages.
struct StyleAttrs {
    bold: bool,
    italic: bool,
    underline: bool,
}

/// Extracts the attribute string between the tag name and its closing `>`,
/// given the byte offset where the body text starts (i.e. right after that
/// `>`).
fn attrs_before(svg: &str, tag_start: &str, body_start: usize) -> String {
    let open = svg[..body_start].rfind(tag_start).unwrap_or_else(|| {
        panic!("expected an opening `{tag_start}` before position {body_start} in {svg:?}")
    });
    svg[open + tag_start.len()..body_start - 1].to_string()
}

fn find_style(svgs: &[String], content: &str, is_tspan: bool) -> StyleAttrs {
    let (tag_start, close_tag) = if is_tspan {
        ("<tspan", "</tspan>")
    } else {
        ("<text", "</text>")
    };
    let needle = format!(">{content}{close_tag}");
    for svg in svgs {
        if let Some(body_end) = svg.find(&needle) {
            let body_start = body_end + 1;
            let attrs = attrs_before(svg, tag_start, body_start);
            return StyleAttrs {
                bold: attrs.contains(r#"font-weight="bold""#),
                italic: attrs.contains(r#"font-style="italic""#),
                underline: attrs.contains(r#"text-decoration="underline""#),
            };
        }
    }
    panic!("expected a `{tag_start} ...{needle}` element in the rendered SVG(s): {svgs:?}");
}

#[then(expr = "the {word} text renders bold, italic, and underlined")]
fn then_text_renders_bold_italic_underlined(world: &mut BoldItalicUnderlineWorld, kind: String) {
    let fixture = kind_fixture(&kind);
    let style = find_style(&world.svgs, fixture.content, fixture.is_tspan);
    assert!(
        style.bold,
        "expected `{kind}`'s {:?} element to render font-weight=\"bold\"",
        fixture.content
    );
    assert!(
        style.italic,
        "expected `{kind}`'s {:?} element to render font-style=\"italic\"",
        fixture.content
    );
    assert!(
        style.underline,
        "expected `{kind}`'s {:?} element to render text-decoration=\"underline\"",
        fixture.content
    );
}

#[tokio::main]
async fn main() {
    BoldItalicUnderlineWorld::run("tests/features/text_style_bold_italic_underline.feature").await;
}
