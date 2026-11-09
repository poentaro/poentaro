use super::*;

/// Regression coverage for the lyric-verse-label feature: each verse row
/// gets its own visible `RowLabel` text (its lyric part abbreviation, same
/// as every other verse row's, rather than the part's abbreviation) plus
/// its own invisible click target, mirroring how a part gets its own
/// `RowLabel` text plus `PartLabelClickTarget`.
fn resolve_test_score(input: &str) -> Vec<compositor::types::AbsolutePage> {
    let score = compile(input, "test", &[]).unwrap();
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
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
    let compile_result = compiler::compile(&score);
    let compile_result = consolidator::consolidate(compile_result);
    let grid_pages =
        grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None).pages;
    coordinate_resolver::resolve(
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
            paddings: config.element_paddings(),
            page_number_vertical_padding_pt: config.page_number_vertical_padding_pt(),
            ..Default::default()
        },
    )
    .expect("coordinate resolver should not fail in tests")
}

const TWO_VERSE_INPUT: &str = r#"metadata { title "t" },

parts ["Melody" [M]: notes, "Melody verse 1" [Mv1]: lyrics [M], "Melody verse 2" [Mv2]: lyrics [M]],

score """
    time=4/4 key=C4 bpm=120
    [M] 1 2 3 4
    [Mv1] do re mi fa
    [Mv2] la ti da di
    """,
"#;

#[test]
fn each_verse_row_renders_its_own_label_text() {
    let abs = resolve_test_score(TWO_VERSE_INPUT);
    let label_texts: Vec<&str> = abs[0]
        .elements
        .iter()
        .filter_map(|e| match &e.content {
            compositor::types::AbsoluteContent::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        label_texts.iter().filter(|&&t| t == "M").count(),
        1,
        "expected an \"M\" label on the notes row only, got {label_texts:?}"
    );
    assert_eq!(
        label_texts
            .iter()
            .filter(|&&t| t == "Mv1" || t == "Mv2")
            .count(),
        2,
        "expected \"Mv1\" and \"Mv2\" labels on the verse rows, got {label_texts:?}"
    );
}

#[test]
fn each_verse_row_gets_its_own_click_target() {
    let abs = resolve_test_score(TWO_VERSE_INPUT);
    let targets: Vec<(usize, usize, usize)> = abs[0]
        .elements
        .iter()
        .filter_map(|e| match &e.content {
            compositor::types::AbsoluteContent::LyricLabelClickTarget {
                source_part_index,
                measure_index_start,
                measure_index_end,
                ..
            } => Some((*source_part_index, *measure_index_start, *measure_index_end)),
            _ => None,
        })
        .collect();
    assert_eq!(
        targets,
        vec![(1, 0, 0), (2, 0, 0)],
        "expected one click target per lyric part's row, scoped to the system's own measure range"
    );
}
