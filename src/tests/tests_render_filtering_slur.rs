use super::*;

#[test]
fn hiding_part_a_still_renders_tie_arc_on_part_b_across_merged_rest_measures() {
    // [a] has plain notes in measures 1-2 and measure 3; [b] only has content in
    // measure 3, where a tie chain (`6m __~_6m`) connects two chord symbols.
    // Hiding [a] turns measures 1-2 into all-rest for [b], so they collapse into
    // one multi-measure-rest block (see
    // `hiding_a_track_lets_plain_rest_measures_from_hidden_track_collapse` in
    // `compiler::tests_multi_measure_rest`). The tie span was recorded against
    // the pre-merge global measure index (2), which no longer lines up with the
    // post-merge block list, so the arc should still render at the tied notes
    // but currently disappears instead.
    let input = r#"metadata { title "t", author "a" },

parts ["a": notes, "b": chords],

score """
    [a] 1

    [a]1

    [a]1
    [b] 6m __~_6m__0_
    """,
"#;

    let shown = render_svgs_from_source(input, "test.poentaro", &[])
        .unwrap()
        .svgs;
    let arc_count_shown = shown[0].matches(r#"data-variant="tie-or-slur""#).count();
    assert_eq!(
        arc_count_shown, 1,
        "sanity check: with part a visible, part b's tie should render once, svg={}",
        &shown[0]
    );

    let hidden =
        render_svgs_from_source_filtered(input, "test.poentaro", Some(&["b".to_string()]), &[])
            .unwrap()
            .svgs;
    let arc_count_hidden = hidden[0].matches(r#"data-variant="tie-or-slur""#).count();
    assert_eq!(
        arc_count_hidden, 1,
        "hiding part a should not make part b's tie arc disappear, svg={}",
        &hidden[0]
    );
}
