// ── `PageLayout::Continuous`: the preview's max view ────────────────────────

use super::*;
use grid_layout::{ContinuousPage, PageLayout};
use itertools::Itertools;

const MEASURE: &str = "[S] 1_ 2_ 3_ 4_ 5_ 6_ 7_ 1'_";

/// A titled score of `measure_count` dense measures: long enough to fill
/// several A4 pages.
fn source(measure_count: usize) -> String {
    format!(
        "metadata {{ title \"Max View\" }},\n\nparts [\"S\" : notes],\n\nscore \"\"\"\n{}\n\"\"\",\n",
        std::iter::repeat_n(MEASURE, measure_count).join("\n\n")
    )
}

fn render(page_layout: PageLayout) -> Vec<String> {
    let output = render_documents_from_source_with_visibility(
        &source(120),
        "test.poentaro",
        &ResolvedPartVisibility::default(),
        &[],
        RenderView {
            note_labels: NoteLabels::Degrees,
            page_layout,
        },
    )
    .unwrap();
    serializer::serialize(&output.documents, None)
}

fn continuous(width_pt: f32) -> PageLayout {
    PageLayout::Continuous(ContinuousPage { width_pt })
}

/// One `part-label` per system, since the score has a single part.
fn system_count(svgs: &[String]) -> usize {
    svgs.concat().matches(r#"data-tag="part-label""#).count()
}

#[test]
fn a_continuous_page_holds_the_whole_score_on_one_page() {
    assert!(render(PageLayout::A4).len() > 1);
    assert_eq!(render(continuous(595.0)).len(), 1);
}

#[test]
fn a_continuous_page_has_no_page_number() {
    let svgs = render(continuous(595.0));
    assert!(!svgs[0].contains(">1 / 1<"), "{}", svgs[0]);
}

#[test]
fn a_narrower_continuous_page_wraps_into_more_systems() {
    let wide = render(continuous(800.0));
    let narrow = render(continuous(300.0));
    assert!(
        system_count(&narrow) > system_count(&wide),
        "narrow: {}, wide: {}",
        system_count(&narrow),
        system_count(&wide)
    );
}

#[test]
fn a_continuous_page_is_as_wide_as_asked_and_as_tall_as_its_content() {
    let output = render_documents_from_source_with_visibility(
        &source(120),
        "test.poentaro",
        &ResolvedPartVisibility::default(),
        &[],
        RenderView {
            note_labels: NoteLabels::Degrees,
            page_layout: continuous(400.0),
        },
    )
    .unwrap();
    let page = &output.documents[0];
    assert_eq!(page.width_pt, 400.0);
    assert!(page.height_pt > 842.0, "height: {}", page.height_pt);
}
