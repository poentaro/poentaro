use super::*;

/// Extracts the `x` attribute of the `<text ...>` element immediately following
/// a `data-section-label="{label}"` marker in a serialized SVG string.
fn section_label_x(svg: &str, label: &str) -> f32 {
    let marker = format!(r#"data-section-label="{label}""#);
    let marker_pos = svg
        .find(&marker)
        .unwrap_or_else(|| panic!("label {label:?} not found in SVG"));
    let text_start = svg[marker_pos..]
        .find("<text ")
        .unwrap_or_else(|| panic!("no <text> element after label {label:?}"));
    let text_tag_start = marker_pos + text_start;
    let x_attr_start = svg[text_tag_start..].find(r#"x=""#).unwrap() + text_tag_start + 3;
    let x_attr_end = svg[x_attr_start..].find('"').unwrap() + x_attr_start;
    svg[x_attr_start..x_attr_end].parse().unwrap()
}

#[test]
fn section_labels_align_when_leading_measures_are_hidden_via_part_filter() {
    // With part `a` filtered out below, measures 0-1 have zero parts each —
    // vacuously "all rest" — so `merge_rest_runs` collapses them into a
    // single `MultiMeasureRest` block (labeled XXX) before layout ever sees
    // them; measure 2 (`b`+`c`) is a second, separate block. That leaves only
    // 2 blocks total by the time `pack_into_systems` runs, so
    // `max_measures_per_system = 1` is needed to force them into two
    // separate systems — union-of-parts packing packs purely by count, so
    // without this cap both blocks would land in one system and XXX/YYY
    // would legitimately sit at different measure-column x-positions rather
    // than both at each system's own start.
    let input = r#"metadata { max measures per system 1 },

parts ["a": notes, "b": notes, "c": notes],

score """
    label="XXX"
    [a] 1

    [a] 1

    label="YYY"
    [b] 2
    [c] 3
    """,
"#;

    let svgs = render_svgs_from_source_filtered(
        input,
        "test.poentaro",
        Some(&["b".into(), "c".into()]),
        &[],
    )
    .unwrap()
    .svgs;

    let xxx_x = section_label_x(&svgs[0], "XXX");
    let yyy_x = section_label_x(&svgs[0], "YYY");

    assert_eq!(
        xxx_x, yyy_x,
        "section labels XXX and YYY should start at the same x position when part `a` is hidden \
         (XXX at x={xxx_x}, YYY at x={yyy_x})"
    );
}
