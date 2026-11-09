use super::*;

// `1,` is below the harmonica's lowest hole (middle C, hole 1 blow); `4#`
// (F♯) is hole 2 draw with the slide.
const SOURCE: &str = r#"parts ["S": notes],

score """
    time=4/4 key=C4 bpm=120
    [S] 1, 1 1' 4#
    """,
"#;

/// The `<text>` elements whose `data-variant` is `variant`, in document
/// order.
fn texts_of_variant<'a>(svg: &'a str, variant: &str) -> Vec<&'a str> {
    let attribute = format!("data-variant=\"{variant}\"");
    svg.split("<text")
        .skip(1)
        .filter(|element| element.contains(&attribute))
        .collect()
}

fn text_content(element: &str) -> &str {
    let (_, after_open) = element.split_once('>').unwrap();
    after_open.split_once("</text>").unwrap().0
}

fn render() -> String {
    render_svgs_from_source_with_visibility(
        SOURCE,
        "test.poentaro",
        &ResolvedPartVisibility::default(),
        &[],
        NoteLabels::HarmonicaTabs,
    )
    .unwrap()
    .svgs
    .concat()
}

#[test]
fn harmonica_tabs_replace_the_degree_digits() {
    let svg = render();
    let tabs: Vec<&str> = texts_of_variant(&svg, "note-head")
        .into_iter()
        .map(text_content)
        .collect();
    assert_eq!(tabs, ["1", "4", "(2)"]);
}

#[test]
fn an_unplayable_note_is_drawn_in_red() {
    let svg = render();
    let unplayable = texts_of_variant(&svg, "unplayable-note-head");
    assert_eq!(unplayable.len(), 1);
    assert_eq!(text_content(unplayable[0]), "?");
    assert!(unplayable[0].contains("fill=\"#d32f2f\""));
}

#[test]
fn harmonica_tabs_draw_no_octave_dots() {
    let svg = render();
    let octave_dots = svg
        .split("<circle")
        .skip(1)
        .filter(|element| element.contains("data-variant=\"note-head\""))
        .count();
    assert_eq!(octave_dots, 0);
}

#[test]
fn a_slid_note_gets_an_arrow_above_it() {
    let svg = render();
    let strokes = svg
        .split("<line")
        .skip(1)
        .filter(|element| element.contains("data-variant=\"harmonica-slide\""))
        .count();
    // A shaft and the two strokes of its head, for the one F♯.
    assert_eq!(strokes, 3);
}
