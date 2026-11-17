use super::*;

/// Extracts the `width` attribute of every `measure-click-target-rect`
/// element in a serialized SVG string, in document order (one per measure).
fn measure_click_target_widths(svg: &str) -> Vec<f32> {
    let marker = r#"data-variant="measure-click-target-rect""#;
    let mut widths = Vec::new();
    let mut search_from = 0;
    while let Some(found) = svg[search_from..].find(marker) {
        let marker_pos = search_from + found;
        let rect_start = svg[..marker_pos]
            .rfind("<rect ")
            .unwrap_or_else(|| panic!("no <rect start before marker at {marker_pos}"));
        let width_attr_start =
            svg[rect_start..marker_pos].find(r#"width=""#).unwrap() + rect_start + 7;
        let width_attr_end = svg[width_attr_start..].find('"').unwrap() + width_attr_start;
        widths.push(svg[width_attr_start..width_attr_end].parse().unwrap());
        search_from = marker_pos + marker.len();
    }
    widths
}

/// Renders `input` and returns the two measures' click-target rect widths.
fn two_measure_widths(input: &str) -> (f32, f32) {
    let svgs = render_svgs_from_source(input, "test.poentaro", &[])
        .unwrap()
        .svgs;
    let widths = measure_click_target_widths(&svgs[0]);
    assert_eq!(widths.len(), 2, "expected exactly two measures");
    (widths[0], widths[1])
}

#[test]
fn measure_with_chord_bass_note_is_wider_than_measure_without() {
    // Control: both measures have the plain chord `1`, so any width
    // difference here is purely from the two measures' positions in the
    // system (e.g. the first measure absorbing the leading bar-line
    // column), not from chord content.
    let control = r#"parts ["c": chords],

score """
    [c] 1

    [c] 1
    """,
"#;
    let (control_first, control_second) = two_measure_widths(control);

    // Experiment: second measure's chord has a bass note (`2m/5`).
    let experiment = r#"parts ["c": chords],

score """
    [c] 1

    [c] 2m/5
    """,
"#;
    let (experiment_first, experiment_second) = two_measure_widths(experiment);

    assert!(
        experiment_second > experiment_first,
        "measure with chord bass note (2m/5) should be wider than measure without (1): \
         measure1 width={experiment_first:.1}, measure2 width={experiment_second:.1}"
    );

    // The gap between measures must exceed the position-only baseline gap
    // from the control, otherwise the bass note isn't actually contributing
    // any width of its own.
    let control_gap = control_second - control_first;
    let experiment_gap = experiment_second - experiment_first;
    assert!(
        experiment_gap > control_gap,
        "chord bass note should widen its column beyond the position-only baseline gap: \
         control gap={control_gap:.1} (first={control_first:.1}, second={control_second:.1}), \
         experiment gap={experiment_gap:.1} (first={experiment_first:.1}, second={experiment_second:.1})"
    );
}
