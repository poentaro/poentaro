use super::*;

const SOURCE: &str = r#"parts ["S": notes, "C": chords],

score """
    time=4/4 key=C4 bpm=120
    [S] 1 2 3 4
    [C] 1 6m 4 1sus27
    """,
"#;

/// The `data-guitar-frets` of every guitar diagram group, in document order.
fn diagram_frets(svg: &str) -> Vec<String> {
    svg.split(r#"data-tag="guitar-diagram" data-guitar-frets=""#)
        .skip(1)
        .filter_map(|after| Some(after.split_once('"')?.0.to_string()))
        .collect()
}

fn render(note_labels: NoteLabels) -> String {
    render_svgs_from_source_with_visibility(
        SOURCE,
        "test.poentaro",
        &ResolvedPartVisibility::default(),
        &[],
        note_labels,
    )
    .unwrap()
    .svgs
    .concat()
}

#[test]
fn the_guitar_view_draws_a_diagram_for_every_chord_with_a_voicing() {
    assert_eq!(
        diagram_frets(&render(NoteLabels::GuitarChords)),
        ["x 3 2 0 1 0", "x 0 2 2 1 0", "1 3 3 2 1 1"]
    );
}

#[test]
fn the_numbered_view_draws_no_diagrams() {
    assert_eq!(
        diagram_frets(&render(NoteLabels::Degrees)),
        Vec::<String>::new()
    );
}
