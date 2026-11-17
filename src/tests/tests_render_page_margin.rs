// ── `page_margin_pt`: the blank margin on every side of the page ────────────

use super::*;
use itertools::Itertools;

const TITLE: &str = "Margin Test";
const MEASURE: &str = "[S] 1_ 2_ 3_ 4_ 5_ 6_ 7_ 1'_";

/// A titled score of `measure_count` dense measures, with `page_margin_pt`
/// set when `margin` is `Some`.
fn source(margin: Option<u32>, measure_count: usize) -> String {
    let margin_line = margin.map_or(String::new(), |pt| format!("page margin pt : {pt}, "));
    format!(
        "metadata {{ {margin_line}title \"{TITLE}\", max measures per system : 8 }},\n\nparts [\"S\" : notes],\n\nscore \"\"\"\n{}\n\"\"\",\n",
        std::iter::repeat_n(MEASURE, measure_count).join("\n\n")
    )
}

fn render(source: &str) -> RenderOutput {
    render_svgs_from_source_with_visibility(
        source,
        "test.poentaro",
        &ResolvedPartVisibility::default(),
        &[],
        NoteLabels::Degrees,
    )
    .unwrap()
}

/// The `y` attribute of the title's `<text>` element on the first page.
fn title_y(output: &RenderOutput) -> f32 {
    let svg = &output.svgs[0];
    let title_end = svg.find(&format!(">{TITLE}")).expect("title text");
    let tag_start = svg[..title_end].rfind("<text").expect("title <text> tag");
    let tag = &svg[tag_start..title_end];
    let y_start = tag.find(" y=\"").expect("y attribute") + 4;
    let y_len = tag[y_start..].find('"').expect("closing quote");
    tag[y_start..y_start + y_len].parse().expect("numeric y")
}

/// One `part-label` per system, since the score has a single part.
fn system_count(output: &RenderOutput) -> usize {
    output
        .svgs
        .concat()
        .matches(r#"data-tag="part-label""#)
        .count()
}

#[test]
fn a_wider_margin_moves_the_header_down_by_the_difference() {
    let default_y = title_y(&render(&source(None, 1)));
    let wide_y = title_y(&render(&source(Some(50), 1)));
    assert!(
        (wide_y - default_y - 25.0).abs() < 0.01,
        "50pt margin should sit 25pt below the default 25pt margin: {default_y} -> {wide_y}"
    );
}

#[test]
fn a_wider_margin_narrows_systems_so_measures_wrap_sooner() {
    assert_eq!(system_count(&render(&source(None, 4))), 1);
    assert!(system_count(&render(&source(Some(150), 4))) > 1);
}
