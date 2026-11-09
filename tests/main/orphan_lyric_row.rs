#![allow(clippy::disallowed_macros)]
use poentaro::grid_layout::types::{GridContent, Header};
use poentaro::render_config::RenderConfig;

const SOURCE: &str = r#"parts [
    "Soprano" [s1] : notes,
    "Soprano" [s2] : follow [s1],
    "Alto 1" [a1] : follow [s1],
    "Alto 2" [a2] : follow [s1],
    "Tenor" [t] : follow [s1],
],

score """
    [t] 6,
    [a2] 6,
    [a1] 3
    [s2] 6
    [s1] 0
    """,
"#;

/// Reported bug: the rendered score shows an unexplained vertical gap between
/// the `s2` row and the `a1` row that isn't present between any other pair of
/// adjacent rows.
///
/// Root cause: `s1` is an all-rest measure (`0`), so it's hidden by the
/// default `hide_resting_parts` setting. But `s2`/`a1`/`a2`/`t` all
/// `follow[s1]` — even though none of these parts, including `s1` itself,
/// ever supplies lyric text anywhere in the score. Each follower must not
/// get an implicit, empty lyric slot; identical empty lyric rows merging
/// into a single leftover row would render as a blank gap with no visible
/// content.
#[test]
fn no_orphan_empty_lyric_row_when_no_part_has_lyric_text() {
    let score = poentaro::compile(SOURCE, "test.poentaro", &[]).unwrap();
    let compile_result = poentaro::compiler::compile(&score);
    let compile_result = poentaro::consolidator::consolidate(compile_result);
    let config = RenderConfig::from_metadata(&score.metadata);
    let header = Header {
        title: None,
        subtitle: None,
        author: None,
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
    let pages =
        poentaro::grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None).pages;

    let has_orphan_lyric_row = pages[0].rows.iter().any(|row| {
        !row.elements.is_empty()
            && row.elements.iter().all(|e| {
                matches!(&e.content, GridContent::LyricSyllable { text, .. } if text.trim().is_empty())
            })
    });

    assert!(
        !has_orphan_lyric_row,
        "no part supplies lyric text in this score, so no lyric row should be rendered"
    );
}
