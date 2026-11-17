use super::*;

const SOURCE: &str = r#"metadata { title "t" },

parts ["Melody" [M]: notes, "Melody lyrics" [Mv1]: lyrics [M], "Chords" [C]: chords],

score """
    (bpm=120 key=C4 time=4/4)
    [M] 1 2 3 4
    [Mv1] twin- kle twin- kle
    [C] 1 - - -
    """,
"#;

fn render(visibility: &ResolvedPartVisibility) -> String {
    render_svgs_from_source_with_visibility(
        SOURCE,
        "test.poentaro",
        visibility,
        &[],
        NoteLabels::Degrees,
    )
    .unwrap()
    .svgs
    .concat()
}

fn melody_lyrics_only() -> ResolvedPartVisibility {
    ResolvedPartVisibility {
        rendered_tracks: Some(vec!["Mv1".to_string(), "C".to_string()]),
        ..Default::default()
    }
}

/// The text of every `<text>` element whose `data-variant` is `variant`, in
/// document order.
fn texts_of_variant(svg: &str, variant: &str) -> Vec<String> {
    let marker = format!("data-variant=\"{variant}\"");
    svg.split("<text ")
        .skip(1)
        .filter(|node| node.contains(&marker))
        .filter_map(|node| {
            let start = node.find('>')? + 1;
            let end = node.find("</text>")?;
            Some(node[start..end].to_string())
        })
        .collect()
}

/// The `x` attribute of the first `<text>` element of `variant` showing `text`.
fn x_of(svg: &str, variant: &str, text: &str) -> f32 {
    let marker = format!("data-variant=\"{variant}\"");
    svg.split("<text ")
        .skip(1)
        .find(|node| node.contains(&marker) && node.contains(&format!(">{text}</text>")))
        .and_then(|node| node.split("x=\"").nth(1))
        .and_then(|rest| rest.split('"').next())
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| panic!("no {variant} text node {text:?} found"))
}

#[test]
fn lyrics_only_part_keeps_its_lyrics() {
    let svg = render(&melody_lyrics_only());

    assert_eq!(
        texts_of_variant(&svg, "lyric"),
        vec!["twin-", "kle", "twin-", "kle"]
    );
}

#[test]
fn lyrics_only_part_draws_no_note_heads() {
    let svg = render(&melody_lyrics_only());

    assert!(texts_of_variant(&svg, "note-head").is_empty());
}

#[test]
fn lyrics_only_part_leaves_other_parts_untouched() {
    let svg = render(&melody_lyrics_only());

    assert_eq!(texts_of_variant(&svg, "chord-symbol"), vec!["1"]);
}

#[test]
fn lyrics_only_part_keeps_the_lyrics_at_their_horizontal_positions() {
    let all = render(&ResolvedPartVisibility::default());
    let lyrics_only = render(&melody_lyrics_only());

    for syllable in ["kle", "twin-"] {
        assert!(
            (x_of(&all, "lyric", syllable) - x_of(&lyrics_only, "lyric", syllable)).abs() < 0.5,
            "{syllable} moved when the notes were hidden"
        );
    }
}

#[test]
fn shown_lyric_part_is_listed_in_the_part_list_legend() {
    let svg = render(&melody_lyrics_only());

    assert!(svg.contains("Mv1 — Melody lyrics"));
    assert!(!svg.contains("M — Melody"));
}

#[test]
fn lyrics_only_measure_without_lyrics_is_a_blank_lyric_row_not_a_rest() {
    let source = r#"parts ["Melody" [M]: notes, "Melody lyrics" [Mv1]: lyrics [M]],

score """
    (bpm=120 key=C4 time=4/4)
    [M] 1 2 3 4

    [M] 5 6 7 1
    [Mv1] la la la la
    """,
"#;

    let svg = render_svgs_from_source_with_visibility(
        source,
        "test.poentaro",
        &melody_lyrics_only(),
        &[],
        NoteLabels::Degrees,
    )
    .unwrap()
    .svgs
    .concat();

    // The blank first measure draws no syllable text (and no rest).
    assert_eq!(
        texts_of_variant(&svg, "lyric"),
        vec!["la", "la", "la", "la"]
    );
    assert!(texts_of_variant(&svg, "note-head").is_empty());
    assert!(texts_of_variant(&svg, "rest").is_empty());
}
