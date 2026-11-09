use super::*;
use compositor::types::AbsoluteContent;

#[test]
fn render_svgs_from_source_smoke() {
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#;
    let svgs = render_svgs_from_source(input, "test.poentaro", &[])
        .unwrap()
        .svgs;
    assert_eq!(svgs.len(), 1);
    assert!(svgs[0].starts_with("<svg"));
    assert!(svgs[0].ends_with("</svg>"));
}

#[test]
fn bpm_change_mid_score_renders_both_bpm_labels() {
    let input = r#"parts ["b": notes],

score """
    bpm=60
    [b] 1

    bpm=130
    [b] 1
    """,
"#;
    let svgs = render_svgs_from_source(input, "test.poentaro", &[])
        .unwrap()
        .svgs;
    assert_eq!(svgs.len(), 1);
    let svg = &svgs[0];
    assert!(
        svg.contains("\u{2669}=60"),
        "should render the initial bpm=60 label"
    );
    assert!(
        svg.contains("\u{2669}=130"),
        "should render the changed bpm=130 label"
    );
}

#[test]
fn lyrics_underflow_render_returns_svgs_and_non_empty_errors() {
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b
    """,
"#;
    let output = render_svgs_from_source(input, "test.poentaro", &[])
        .expect("underflow must not abort the render");
    assert!(
        !output.svgs.is_empty(),
        "should produce at least one SVG page"
    );
    assert_eq!(
        output.diagnostics.len(),
        1,
        "should report one underflow error"
    );
    assert!(output.diagnostics[0].message().contains("underflow"));
    assert!(
        output.svgs[0].contains(r#"data-testid="error-highlight""#),
        "SVG should contain an error-highlight rect"
    );
}

#[test]
fn follow_part_with_tied_note_produces_exactly_one_arc() {
    // p1 has 1~1 (a tied note), p2 follows p1, a is a chord track.
    // After consolidation p1 and p2 merge into one visual row.
    // The tie arc should appear exactly once, not twice (once for p1 and
    // once for p2 displaced above the chord row).
    let input = r#"metadata { title "", author "" },

parts ["Pluck" [p1]: notes, "Pluck 2" [p2]: follow [p1], "Accompaniment" [a]: chords],

score """
    time=4/4 key=C4 bpm=120
    [p1] 1~1
    [a] 3
    """,
"#;
    let svgs = render_svgs_from_source(input, "test.poentaro", &[])
        .unwrap()
        .svgs;
    let arc_count = svgs[0].matches("data-variant=\"tie-or-slur\"").count();
    assert_eq!(
        arc_count, 1,
        "expected exactly one tie arc; got {arc_count}. SVG: {}",
        &svgs[0]
    );
}

#[test]
fn lex_unexpected_char_renders_error_highlight_and_reports_error() {
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 @ 3 4
    """,
"#;
    let output = render_svgs_from_source(input, "test.poentaro", &[])
        .expect("LexUnexpectedChar must not abort the render");
    assert!(
        !output.svgs.is_empty(),
        "should produce at least one SVG page"
    );
    assert_eq!(output.diagnostics.len(), 1, "should report one lex error");
    assert!(
        output.diagnostics[0].message().contains("unexpected"),
        "error message should mention unexpected char, got: {}",
        output.diagnostics[0].message()
    );
    assert!(
        output.svgs[0].contains(r#"data-testid="error-highlight""#),
        "SVG should contain an error-highlight rect for the erroneous measure"
    );
}

#[test]
fn adjacent_beat_group_underlines_have_gap_between_them() {
    // "2_3=4=" is beat 2 and "6_7_" is beat 3 — both get a level-0 beam underline.
    // The underline for beat 2 must end strictly before the underline for beat 3 starts.
    let source = r#"metadata { title "t", author "a" },

parts ["S": notes],

score """
    time=4/4 key=C4 bpm=120
    [S] 0 2_3=4= 6_7_ 0
    """,
"#;
    let score = compile(source, "test", &[]).unwrap();
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
    let abs = coordinate_resolver::resolve(
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
    .expect("coordinate resolver should not fail in tests");

    let mut underlines: Vec<(f32, f32)> = abs[0]
        .elements
        .iter()
        .filter_map(|e| {
            if let AbsoluteContent::Underline { width, level: 0 } = &e.content {
                Some((e.x, *width))
            } else {
                None
            }
        })
        .collect();
    underlines.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    assert_eq!(underlines.len(), 2, "expected two level-0 underlines");
    let (x1, w1) = underlines[0];
    let (x2, _) = underlines[1];
    assert!(
        x2 > x1 + w1,
        "underlines should have a gap but they touch: beat2 ends at {:.1}, beat3 starts at {:.1}",
        x1 + w1,
        x2
    );
}

/// Shared fixture for the two density regression tests below: a sparse
/// system ("a" only) followed by a denser one (extra chord-symbol part "b"),
/// forced into two systems via `max_measures_per_system = 2` — union-of-parts
/// packing packs purely by count now, so without this cap all 3 measures
/// (despite "a"-only vs "a"+"b" differing parts) would land in a single
/// system and there'd be nothing cross-system to compare. Returns page 0's
/// resolved absolute elements.
fn density_fixture_page_elements() -> Vec<compositor::types::AbsoluteElement> {
    let input = r#"metadata { max measures per system 2 },

parts ["a": notes, "b": chords],

score """
    [a] 1

    [a]1

    [a]1
    [b] 6m __~_6m__0_
    """,
"#;
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
    let mut abs = coordinate_resolver::resolve(
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
    .expect("coordinate resolver should not fail in tests");
    abs.swap_remove(0).elements
}

#[test]
fn part_label_width_is_consistent_across_systems_of_differing_density() {
    // Regression test: a system containing only a sparse part ("a") must
    // render its part label at the same x-position as a denser system
    // containing an extra chord-symbol part ("b"), even though the two
    // systems have very different musical column counts.
    let elements = density_fixture_page_elements();

    let label_x_positions: Vec<f32> = elements
        .iter()
        .filter_map(|e| match &e.content {
            AbsoluteContent::Text { content, .. } if content == "a" => Some(e.x),
            _ => None,
        })
        .collect();

    assert!(
        label_x_positions.len() >= 2,
        "expected at least two 'a' part labels across systems, got {}",
        label_x_positions.len()
    );
    let first = label_x_positions[0];
    assert!(
        label_x_positions.iter().all(|&x| (x - first).abs() < 0.01),
        "part label x-position should be identical across systems regardless of \
         musical density: {label_x_positions:?}"
    );
}

#[test]
fn leading_and_trailing_bar_lines_align_across_systems_of_differing_density() {
    // Regression test: the leading bar line (right after the part label) and
    // the trailing bar line (closing the last measure) of every system must
    // land at the same x-position, even though systems differ in musical
    // density — a sparse one-measure system ("a" only) vs. a denser system
    // with an extra chord-symbol part ("b"). Uses the same fixture as
    // `part_label_width_is_consistent_across_systems_of_differing_density`.
    let elements = density_fixture_page_elements();
    // Bar lines are drawn once per system on the note-head sub-row of the
    // first part, so their y-coordinate uniquely identifies which system a
    // bar line belongs to.
    let mut bar_lines_by_row: std::collections::BTreeMap<i64, Vec<f32>> =
        std::collections::BTreeMap::new();
    for e in &elements {
        if matches!(e.content, AbsoluteContent::BarLine { .. }) {
            bar_lines_by_row
                .entry((e.y * 1000.0).round() as i64)
                .or_default()
                .push(e.x);
        }
    }

    assert!(
        bar_lines_by_row.len() >= 2,
        "expected bar lines across at least two systems, got {}",
        bar_lines_by_row.len()
    );

    let leading_x_positions: Vec<f32> = bar_lines_by_row
        .values()
        .map(|xs| xs.iter().cloned().fold(f32::INFINITY, f32::min))
        .collect();
    let trailing_x_positions: Vec<f32> = bar_lines_by_row
        .values()
        .map(|xs| xs.iter().cloned().fold(f32::NEG_INFINITY, f32::max))
        .collect();

    let first_leading = leading_x_positions[0];
    assert!(
        leading_x_positions
            .iter()
            .all(|&x| (x - first_leading).abs() < 0.01),
        "leading bar line x-position should be identical across systems: {leading_x_positions:?}"
    );

    let first_trailing = trailing_x_positions[0];
    assert!(
        trailing_x_positions
            .iter()
            .all(|&x| (x - first_trailing).abs() < 0.01),
        "trailing bar line x-position should be identical across systems: {trailing_x_positions:?}"
    );
}

#[test]
fn tie_operator_on_notes_renders_exactly_two_arcs() {
    // 7_6=5=~5~5 has two ~ tie operators, so exactly 2 arcs should be rendered.
    let input = r#"metadata { title "Untitled", author "" },

parts ["Melody" [m]: notes],

score """
    [m]7_6=5=~5~5
    """,
"#;
    let svgs = render_svgs_from_source(input, "test.poentaro", &[])
        .unwrap()
        .svgs;
    let arc_count = svgs[0].matches(r#"data-variant="tie-or-slur""#).count();
    assert_eq!(arc_count, 2, "expected 2 arcs but got {arc_count}");
}
