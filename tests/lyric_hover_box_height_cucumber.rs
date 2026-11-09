//! Cucumber harness for the lyric click-target box height (see
//! `tests/features/lyric_hover_box_height.feature`): regression coverage for
//! a bug where the invisible hover/click-target rect drawn behind a lyric
//! syllable was sized as a flat `row_height * 1.5` for the whole verse row
//! (`grid_layout::layout_heights::lyric_row_height`), independent of the
//! syllable's own resolved font size (`RenderConfig::lyric_font_size` /
//! `lyric_cjk_font_size`) — so a large `lyrics_font_size` override, CJK
//! syllables first since they render 20% larger than Latin ones at the same
//! setting, could overflow its own click-target box. `lyric_row_height` now
//! takes the row's own resolved max font size (see
//! `grid_layout::expand::lyric_part_max_font_size`) instead of `row_height`.
//!
//! Clippy's `allow-*-in-tests` (clippy.toml) only recognizes `#[test]`-
//! attributed functions as test code; cucumber's `#[given]`/`#[when]`/
//! `#[then]` step functions don't qualify even though this whole file only
//! ever runs under `cargo test`. Mirrors `tests/cucumber.rs`'s
//! `#![allow(clippy::disallowed_macros)]` for the same reason.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::disallowed_macros,
    clippy::needless_pass_by_value
)]

#[path = "support/metadata_field.rs"]
mod metadata_field;

use cucumber::{given, then, when, World as _};
use metadata_field::metadata_field;
use poentaro::compositor::types::{AbsoluteContent, DominantBaseline};
use poentaro::fonts::SERIF_FONT_BYTES;
use poentaro::render_config::RenderConfig;
use poentaro::renderer::new_renderer::render_new;
use poentaro::renderer::new_types::SvgKind;
use poentaro::{coordinate_resolver, grid_layout};

/// Mirrors `font_metrics::is_cjk_char`'s range (CJK Unified Ideographs
/// only) — that function is crate-private, so it can't be imported from an
/// integration test; `utils::is_cjk_char` is public but covers a wider
/// range (Hiragana/Katakana/Hangul too) and would disagree with the
/// renderer's actual font-size choice for those scripts.
fn is_cjk_char(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c)
}

#[derive(Debug, Default, cucumber::World)]
struct LyricHoverBoxWorld {
    metadata_overrides: Vec<(String, String)>,
    syllable_text: String,
    click_target_height: Option<f32>,
    resolved_font_size: Option<f32>,
    text_y: Option<f32>,
    text_baseline: Option<DominantBaseline>,
    click_target_y: Option<f32>,
}

#[given(expr = "{string} sets {string} to {string}")]
fn given_metadata_override(
    world: &mut LyricHoverBoxWorld,
    _section: String,
    key: String,
    value: String,
) {
    world.metadata_overrides.push((key, value));
}

#[given(expr = "the score has a note with lyric syllable {string}")]
fn given_lyric_syllable(world: &mut LyricHoverBoxWorld, syllable: String) {
    world.syllable_text = syllable;
}

/// Bundles `config`'s four `*_horizontal_padding_pt` accessors, factored out
/// of `when_rendered` to keep that function under clippy's line-count cap.
fn element_paddings(config: &RenderConfig) -> coordinate_resolver::ElementPaddings {
    coordinate_resolver::ElementPaddings {
        notes: config.notes_horizontal_padding_pt(),
        chords: config.chords_horizontal_padding_pt(),
        lyrics: config.lyrics_horizontal_padding_pt(),
        note_dash: config.note_dash_horizontal_padding_pt(),
    }
}

/// Bundles `config`'s label/lyric/notes/chords font sizes into `resolve`'s
/// `ResolveFontSizes`, factored out of `when_rendered` to keep that function
/// under clippy's line-count cap.
fn resolve_font_sizes(config: &RenderConfig) -> coordinate_resolver::ResolveFontSizes {
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
        paddings: element_paddings(config),
        page_number_vertical_padding_pt: config.page_number_vertical_padding_pt(),
        ..Default::default()
    }
}

#[when(expr = "it is rendered")]
fn when_rendered(world: &mut LyricHoverBoxWorld) {
    let metadata = std::iter::once(r#"title "t""#.to_string())
        .chain(
            world
                .metadata_overrides
                .iter()
                .map(|(key, value)| metadata_field(key, value)),
        )
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        r#"metadata {{ {metadata} }},

parts ["Melody" [M]: notes, "Lyrics" [L]: lyrics [M]],

score """
    time=4/4 key=C4 bpm=120
    [M] 1
    [L] {}
    """,
"#,
        world.syllable_text
    );

    let score = poentaro::compile(&source, "test.poentaro", &[])
        .unwrap_or_else(|err| panic!("compile returned an irrecoverable error: {}", err.message()));
    let config = RenderConfig::from_metadata(&score.metadata);
    let header = grid_layout::types::Header {
        title: score.metadata.title.clone(),
        subtitle: score.metadata.subtitle.clone(),
        author: score.metadata.author.clone(),
        part_list: vec![],
        parts_list_columns: 3,
        sequence: None,
        title_font_size: score.metadata.title_style.font_size as f32,
        subtitle_font_size: score.metadata.subtitle_style.font_size as f32,
        author_font_size: score.metadata.author_style.font_size as f32,
        sequence_font_size: score.metadata.sequence.font_size as f32,
        part_legend_font_size: score.metadata.part_legend.font_size as f32,
        ..Default::default()
    };
    let compile_result = poentaro::compiler::compile(&score);
    let compile_result = poentaro::consolidator::consolidate(compile_result);
    let grid_pages =
        grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None).pages;
    let abs = coordinate_resolver::resolve(
        &grid_pages,
        config.note_number_width as f32,
        config.page_frame(),
        resolve_font_sizes(&config),
    )
    .unwrap_or_else(|err| panic!("coordinate resolver should not fail in tests: {err:?}"));

    world.resolved_font_size = Some(if world.syllable_text.chars().any(is_cjk_char) {
        config.lyric_cjk_font_size()
    } else {
        config.lyric_font_size()
    });

    world.click_target_height =
        abs.iter()
            .flat_map(|page| page.elements.iter())
            .find_map(|element| match &element.content {
                AbsoluteContent::LyricClickTarget { height, .. } => Some(*height),
                _ => None,
            });
    world.click_target_y = abs
        .iter()
        .flat_map(|page| page.elements.iter())
        .find_map(|element| match &element.content {
            AbsoluteContent::LyricClickTarget { .. } => Some(element.y),
            _ => None,
        });

    // Render through to the actual SVG-level output — the `AbsoluteContent`
    // returned by `coordinate_resolver::resolve` carries the syllable as
    // `AbsoluteContent::Lyric { text, .. }`, with no baseline of its own;
    // the baseline (the thing this bug is actually about) is only decided
    // downstream, by `render_lyric`/`render_lyric_line` when they turn that
    // into an `SvgElement { kind: SvgKind::Text { baseline, .. } }`.
    let svg_docs = render_new(&abs, &config);
    let syllable_text = world.syllable_text.clone();
    let text_element = svg_docs
        .iter()
        .flat_map(|doc| doc.elements.iter())
        .find_map(|element| match &element.kind {
            SvgKind::Text {
                content, baseline, ..
            } if *content == syllable_text => Some((element.y, *baseline)),
            _ => None,
        });
    world.text_y = text_element.map(|(y, _)| y);
    world.text_baseline = text_element.map(|(_, baseline)| baseline);
}

#[then(expr = "the lyric click-target height should be at least the resolved lyric font size")]
fn then_click_target_covers_font_size(world: &mut LyricHoverBoxWorld) {
    let Some(height) = world.click_target_height else {
        panic!("no lyric click target found in the rendered output");
    };
    let Some(font_size) = world.resolved_font_size else {
        panic!("font size was not resolved");
    };
    assert!(
        height >= font_size,
        "lyric click-target height ({height}pt) is smaller than the resolved lyric font size \
         ({font_size}pt) for syllable {:?} — its glyph will overflow the hover box",
        world.syllable_text
    );
}

#[then(expr = "the lyric text baseline should be middle-anchored")]
fn then_baseline_is_middle(world: &mut LyricHoverBoxWorld) {
    let Some(baseline) = world.text_baseline else {
        panic!("no lyric text element found in the rendered output");
    };
    assert_eq!(
        baseline,
        DominantBaseline::Middle,
        "lyric text should be anchored by its glyph center (DominantBaseline::Middle), \
         matching every other glyph renderer that shares the row's midpoint `y` — a \
         `Hanging` baseline misuses that midpoint as the glyph's top instead"
    );
}

#[then(expr = "the lyric glyph should be fully contained within its click-target box")]
fn then_glyph_contained_in_click_target(world: &mut LyricHoverBoxWorld) {
    let Some(text_y) = world.text_y else {
        panic!("no lyric text element found in the rendered output");
    };
    let Some(font_size) = world.resolved_font_size else {
        panic!("font size was not resolved");
    };
    let Some(click_target_y) = world.click_target_y else {
        panic!("no lyric click target found in the rendered output");
    };
    let Some(click_target_height) = world.click_target_height else {
        panic!("no lyric click target found in the rendered output");
    };

    // Independently parse the pinned font in the test itself (rather than
    // calling the private production helper, which isn't reachable from
    // this external test crate) to compute the glyph's real vertical span
    // at the resolved font size.
    let face = ttf_parser::Face::parse(SERIF_FONT_BYTES, 0)
        .unwrap_or_else(|err| panic!("failed to parse pinned title font: {err}"));
    let vertical_extent = face.height() as f32 / face.units_per_em() as f32 * font_size;

    // `text_y` is the glyph's center (DominantBaseline::Middle), so its
    // real vertical span straddles it.
    let glyph_top = text_y - vertical_extent / 2.0;
    let glyph_bottom = text_y + vertical_extent / 2.0;

    let click_target_top = click_target_y;
    let click_target_bottom = click_target_y + click_target_height;

    assert!(
        glyph_top >= click_target_top && glyph_bottom <= click_target_bottom,
        "lyric glyph span [{glyph_top}, {glyph_bottom}] is not fully contained within its \
         click-target box [{click_target_top}, {click_target_bottom}] for syllable {:?}",
        world.syllable_text
    );
}

#[tokio::main]
async fn main() {
    LyricHoverBoxWorld::run("tests/features/lyric_hover_box_height.feature").await;
}
