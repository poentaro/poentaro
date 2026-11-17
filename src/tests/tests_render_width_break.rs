use super::*;
use crate::error::WarningKind;
use itertools::Itertools;

const MEASURE: &str = "[S] 1_ 2_ 3_ 4_ 5_ 6_ 7_ 1'_";

/// `measure_count` dense measures under `max_measures_per_system=8`.
fn source(measure_count: usize) -> String {
    format!(
        r#"metadata {{ max measures per system 8 }},

parts ["S": notes],

score """
    {}
    """,
"#,
        std::iter::repeat_n(MEASURE, measure_count).join("\n\n")
    )
}

fn render(source: &str, note_labels: NoteLabels) -> RenderOutput {
    render_svgs_from_source_with_visibility(
        source,
        "test.poentaro",
        &ResolvedPartVisibility::default(),
        &[],
        note_labels,
    )
    .unwrap()
}

/// One `part-label` per system, since the score has a single part.
fn system_count(output: &RenderOutput) -> usize {
    output
        .svgs
        .concat()
        .matches(r#"data-tag="part-label""#)
        .count()
}

fn overflow_warning_count(output: &RenderOutput) -> usize {
    output
        .diagnostics
        .iter()
        .filter(|d| matches!(d, Diagnostic::Warning(w) if w.kind == WarningKind::MeasureOverflow))
        .count()
}

#[test]
fn the_numbered_view_fits_four_measures_in_one_system() {
    let output = render(&source(4), NoteLabels::Degrees);
    assert_eq!(system_count(&output), 1);
    assert_eq!(overflow_warning_count(&output), 0);
}

#[test]
fn the_guitar_view_breaks_systems_early_instead_of_overflowing_the_page() {
    let output = render(&source(4), NoteLabels::GuitarChords);
    assert!(system_count(&output) > 1);
    assert_eq!(overflow_warning_count(&output), 0);
}
