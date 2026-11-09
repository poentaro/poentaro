//! Cucumber harness for `tests/features/text_style_rendering.feature`:
//! verifies each `TextStyle` component has a real rendering/layout effect,
//! not just a resolved config value. Each scenario's `metadata` overrides are
//! spliced into a minimal document and run through the full pipeline. "Then"
//! steps compare the configured run against a freshly rendered baseline
//! rather than a hardcoded expected number.
//! Clippy's `allow-*-in-tests` skips cucumber steps, hence the allows.
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
use poentaro::compositor::types::{AbsoluteContent, AbsolutePage};
use poentaro::grid_layout::types::{GridContent, GridPage};
use poentaro::renderer::new_renderer::render_new;
use poentaro::renderer::new_types::{SvgDocument, SvgKind};
use poentaro::{coordinate_resolver, grid_layout};

/// One full pipeline run's outputs — everything a "Then" step needs to
/// inspect, at whichever stage (grid/absolute/SVG) its assertion lives at.
struct RenderResult {
    grid_pages: Vec<GridPage>,
    abs_pages: Vec<AbsolutePage>,
    svg_docs: Vec<SvgDocument>,
}

/// Bundles `config`'s four `*_horizontal_padding_pt` accessors (`RenderConfig::element_paddings` is `pub(crate)`).
fn element_paddings(
    config: &poentaro::render_config::RenderConfig,
) -> coordinate_resolver::ElementPaddings {
    coordinate_resolver::ElementPaddings {
        notes: config.notes_horizontal_padding_pt(),
        chords: config.chords_horizontal_padding_pt(),
        lyrics: config.lyrics_horizontal_padding_pt(),
        note_dash: config.note_dash_horizontal_padding_pt(),
    }
}

fn zero_vertical_padding(kind: &str) -> String {
    metadata_field(kind, "{ vertical_padding_pt: 0 }")
}

fn render_source(metadata_lines: &[String], title: &str, score_body: &str) -> RenderResult {
    let mut lines = metadata_lines.to_vec();
    lines.push(format!("title {title:?}"));
    let metadata_block = lines.join(", ");
    let source =
        format!("metadata {{ {metadata_block} }},\n\nparts [\"S\" : notes],\n\nscore \"\"\"\n{score_body}\n\"\"\",\n");
    let score = poentaro::compile(&source, "test.poentaro", &[])
        .unwrap_or_else(|err| panic!("compile returned an irrecoverable error: {}", err.message()));
    let config = poentaro::render_config::RenderConfig::from_metadata(&score.metadata);
    let header = grid_layout::types::Header {
        title: score.metadata.title.clone(),
        subtitle: score.metadata.subtitle.clone(),
        author: score.metadata.author.clone(),
        part_list: vec![grid_layout::types::PartListEntry {
            abbreviation: "S".to_string(),
            display_name: "Soprano".to_string(),
        }],
        parts_list_columns: 3,
        sequence: Some(vec![grid_layout::types::SequenceEntryInfo {
            label: "A".to_string(),
            part_filter: None,
        }]),
        title_font_size: score.metadata.title_style.font_size as f32,
        subtitle_font_size: score.metadata.subtitle_style.font_size as f32,
        author_font_size: score.metadata.author_style.font_size as f32,
        sequence_font_size: score.metadata.sequence.font_size as f32,
        part_legend_font_size: score.metadata.part_legend.font_size as f32,
        part_legend_vertical_padding_pt: score.metadata.part_legend.vertical_padding_pt as f32,
        sequence_vertical_padding_pt: score.metadata.sequence.vertical_padding_pt as f32,
        title_vertical_padding_pt: score.metadata.title_style.vertical_padding_pt as f32,
        subtitle_vertical_padding_pt: score.metadata.subtitle_style.vertical_padding_pt as f32,
        author_vertical_padding_pt: score.metadata.author_style.vertical_padding_pt as f32,
        ..Default::default()
    };
    let compile_result = poentaro::compiler::compile(&score);
    let compile_result = poentaro::consolidator::consolidate(compile_result);
    let grid_pages =
        grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None).pages;
    let abs_pages = coordinate_resolver::resolve(
        &grid_pages,
        config.note_number_width as f32,
        config.page_frame(),
        coordinate_resolver::ResolveFontSizes {
            lyric: config.lyric_font_sizes(),
            notes: config.notes_font_size(),
            chords: config.chords_font_size(),
            labels: coordinate_resolver::LabelFontSizes {
                measure_number: config.measure_number_font_size as f32,
                section_label: config.section_label_font_size as f32,
                section_label_vertical_padding_pt: config.section_label_vertical_padding_pt(),
                part_label: config.part_label_font_size as f32,
                ..Default::default()
            },
            paddings: element_paddings(&config),
            page_number_vertical_padding_pt: config.page_number_vertical_padding_pt(),
            ..Default::default()
        },
    )
    .unwrap_or_else(|err| panic!("coordinate resolver should not fail in tests: {err:?}"));
    let svg_docs = render_new(&abs_pages, &config);
    RenderResult {
        grid_pages,
        abs_pages,
        svg_docs,
    }
}

#[derive(Debug, Default, cucumber::World)]
struct TextStyleRenderingWorld {
    metadata_lines: Vec<String>,
    title: String,
    score_body: String,
    kind: String,
    rendered: Option<RenderResultDebug>,
}

/// `RenderResult` isn't `Debug` (its fields aren't), which `cucumber::World`
/// requires of every field — this newtype just carries it through with a
/// hand-rolled stub `Debug` impl.
struct RenderResultDebug(RenderResult);

impl std::fmt::Debug for RenderResultDebug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RenderResultDebug(..)")
    }
}

#[given(expr = "{string} sets {string} to {string}")]
fn given_metadata_line(
    world: &mut TextStyleRenderingWorld,
    _section: String,
    key: String,
    value: String,
) {
    world.metadata_lines.push(metadata_field(&key, &value));
}

#[given(expr = "the score has a note followed by a dash")]
fn given_note_followed_by_dash(world: &mut TextStyleRenderingWorld) {
    world.title = "T".to_string();
    world.score_body = "time=4/4 key=C4 bpm=120\n[S] 1-\n".to_string();
}

#[given(expr = "the score title is {string}")]
fn given_title(world: &mut TextStyleRenderingWorld, title: String) {
    world.title = title;
    world.score_body = "time=4/4 key=C4 bpm=120\n[S] 1\n".to_string();
}

#[given(expr = "a minimal score using {word}")]
fn given_minimal_score_using_kind(world: &mut TextStyleRenderingWorld, kind: String) {
    world.title = "T".to_string();
    world.score_body = if kind == "section_label" {
        "time=4/4 key=C4 bpm=120 label=\"X\"\n[S] 1\n".to_string()
    } else {
        "time=4/4 key=C4 bpm=120\n[S] 1\n".to_string()
    };
    world.kind = kind;
}

#[given(expr = "a score with a title, subtitle and author")]
fn given_score_with_title_subtitle_author(world: &mut TextStyleRenderingWorld) {
    world.title = "T".to_string();
    world
        .metadata_lines
        .extend(["subtitle \"Sub\"".to_string(), "author \"Au\"".to_string()]);
    world.score_body = "time=4/4 key=C4 bpm=120\n[S] 1\n".to_string();
}

#[when(expr = "it is rendered")]
fn when_rendered(world: &mut TextStyleRenderingWorld) {
    let rendered = render_source(&world.metadata_lines, &world.title, &world.score_body);
    world.rendered = Some(RenderResultDebug(rendered));
}

/// Font size (points) the rendered note-dash `SvgKind::Text` element uses —
/// the dash's own U+2014 glyph, distinct from `notes_font_size`.
fn dash_font_size(result: &RenderResult) -> f32 {
    result
        .svg_docs
        .iter()
        .flat_map(|doc| doc.elements.iter())
        .find_map(|e| match &e.kind {
            SvgKind::Text {
                content, font_size, ..
            } if content == "\u{2014}" => Some(*font_size),
            _ => None,
        })
        .expect("note dash element should be present")
}

/// Total height in points of the first page's body rows, excluding the
/// footer — the footer's `remaining_height` always expands to fill
/// whatever the body doesn't use, so including it would cancel out exactly
/// the body-height difference this measures (see
/// `notes_vertical_padding_pt_grows_the_note_head_sub_row` in
/// `src/tests/tests_render_rendering.rs`, the unit-test precedent).
fn notes_body_height_pt(result: &RenderResult) -> f32 {
    let rows = &result.grid_pages[0].rows;
    rows[..rows.len() - 1].iter().map(|r| r.height_pt).sum()
}

/// `label_box_height` off the resolved `AbsoluteContent::DirectiveLine`
/// carrying the score's section label ("X" — see
/// `given_minimal_score_using_kind`).
fn section_label_box_height_pt(result: &RenderResult) -> f32 {
    result
        .abs_pages
        .iter()
        .flat_map(|p| p.elements.iter())
        .find_map(|e| match &e.content {
            AbsoluteContent::DirectiveLine {
                label,
                label_box_height,
                ..
            } if label.as_deref() == Some("X") => Some(*label_box_height),
            _ => None,
        })
        .expect("directive line with a section label should be present")
}

/// The footer page-number text's distance from the page's bottom edge —
/// grows exactly by `page_number.vertical_padding_pt` since the footer row
/// itself already fills all remaining page height regardless of padding
/// (see `resolve_row_element`'s `bottom_padding`,
/// `page_number_vertical_padding_pt_pushes_the_footer_text_up` in
/// `src/tests/tests_render_rendering.rs`).
fn page_number_box_height_pt(result: &RenderResult) -> f32 {
    let page = &result.grid_pages[0];
    let text_y = result
        .abs_pages
        .iter()
        .flat_map(|p| p.elements.iter())
        .find_map(|e| match &e.content {
            AbsoluteContent::Text { content, .. } if content == "1 / 1" => Some(e.y),
            _ => None,
        })
        .expect("footer page-number text element should be present");
    page.height_pt - text_y
}

/// Height of the first page's header row holding the element `is_target`
/// matches — the part legend entry or the `sequence` summary line.
fn header_row_height_pt(result: &RenderResult, is_target: impl Fn(&GridContent) -> bool) -> f32 {
    result.grid_pages[0]
        .rows
        .iter()
        .find(|row| row.elements.iter().any(|e| is_target(&e.content)))
        .map(|row| row.height_pt)
        .expect("header row should be present")
}

fn box_height_pt(result: &RenderResult, kind: &str) -> f32 {
    match kind {
        "notes" => notes_body_height_pt(result),
        "part_legend" => header_row_height_pt(
            result,
            |content| matches!(content, GridContent::Text { content, .. } if content == "S \u{2014} Soprano"),
        ),
        "sequence" => header_row_height_pt(result, |content| {
            matches!(content, GridContent::SequenceLine { .. })
        }),
        "section_label" => section_label_box_height_pt(result),
        "page_number" => page_number_box_height_pt(result),
        other => panic!("unrecognized text style kind {other:?}"),
    }
}

/// The `title` text element's resolved `y` — used as the "unrelated
/// element" reference, since the header (where the title sits) is always
/// laid out before every system/footer row, so its position can't move
/// regardless of which kind's `vertical_padding_pt` is under test.
fn title_y(result: &RenderResult, title: &str) -> f32 {
    result
        .abs_pages
        .iter()
        .flat_map(|p| p.elements.iter())
        .find_map(|e| match &e.content {
            AbsoluteContent::Text { content, .. } if content == title => Some(e.y),
            _ => None,
        })
        .expect("title text element should be present")
}

#[then(
    expr = "the rendered dash width at font_size {int} differs from its width at the default note_dash font size"
)]
fn then_dash_width_differs(world: &mut TextStyleRenderingWorld, configured_font_size: i64) {
    let rendered = &world
        .rendered
        .as_ref()
        .expect("'it is rendered' must run first")
        .0;
    let padded_size = dash_font_size(rendered);
    assert!(
        (padded_size - configured_font_size as f32).abs() < 0.001,
        "dash should render at the configured font_size {configured_font_size}, got {padded_size}"
    );
    let baseline = render_source(&[], &world.title, &world.score_body);
    let default_size = dash_font_size(&baseline);
    assert!(
        (padded_size - default_size).abs() > 0.001,
        "dash width at font_size {padded_size} should differ from its width at the default \
         note_dash font size {default_size} — for a monospace glyph, a different font_size \
         necessarily measures a different width"
    );
}

#[then(expr = "the {word} element's rendered box height increases by at least {int}")]
fn then_box_height_increases(world: &mut TextStyleRenderingWorld, kind: String, padding_pt: i64) {
    let rendered = &world
        .rendered
        .as_ref()
        .expect("'it is rendered' must run first")
        .0;
    let padded = box_height_pt(rendered, &kind);
    let baseline = render_source(
        &[zero_vertical_padding(&kind)],
        &world.title,
        &world.score_body,
    );
    let unpadded = box_height_pt(&baseline, &kind);
    assert!(
        padded >= unpadded + padding_pt as f32,
        "padded {kind} box height ({padded}) should be at least {padding_pt}pt taller than \
         unpadded ({unpadded})"
    );
}

#[then(expr = "unrelated elements keep their original position")]
fn then_unrelated_elements_unchanged(world: &mut TextStyleRenderingWorld) {
    let rendered = &world
        .rendered
        .as_ref()
        .expect("'it is rendered' must run first")
        .0;
    let padded_title_y = title_y(rendered, &world.title);
    let baseline = render_source(&[], &world.title, &world.score_body);
    let unpadded_title_y = title_y(&baseline, &world.title);
    assert!(
        (padded_title_y - unpadded_title_y).abs() < 0.001,
        "the title (laid out before any system/footer row) should not move: padded y={padded_title_y}, unpadded y={unpadded_title_y}"
    );
}

#[then(expr = "the header row holding the {word} grows by {int} above and below")]
fn then_header_row_grows(world: &mut TextStyleRenderingWorld, kind: String, padding_pt: i64) {
    let rendered = &world
        .rendered
        .as_ref()
        .expect("'it is rendered' must run first")
        .0;
    let text = match kind.as_str() {
        "title" => "T",
        "subtitle" => "Sub",
        "author" => "Au",
        other => panic!("unrecognized header kind {other:?}"),
    };
    let row_height = |result: &RenderResult| {
        header_row_height_pt(
            result,
            |content| matches!(content, GridContent::Text { content, .. } if content == text),
        )
    };
    let padded = row_height(rendered);
    // Subtitle and author share a row (larger padding wins): clear all three.
    let zero_padding_metadata: Vec<String> = world
        .metadata_lines
        .iter()
        .filter(|line| !line.contains("vertical padding pt"))
        .cloned()
        .chain(["title", "subtitle", "author"].map(zero_vertical_padding))
        .collect();
    let baseline = render_source(&zero_padding_metadata, &world.title, &world.score_body);
    let unpadded = row_height(&baseline);
    assert!(
        (padded - unpadded - 2.0 * padding_pt as f32).abs() < 0.001,
        "the {kind} row ({padded}) should be {padding_pt}pt taller above and below than \
         unpadded ({unpadded})"
    );
}

#[tokio::main]
async fn main() {
    TextStyleRenderingWorld::run("tests/features/text_style_rendering.feature").await;
}
