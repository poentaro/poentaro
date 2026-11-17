use super::*;

const SOURCE: &str = r#"parts ["S": notes],

score """
    time=4/4 key=D4 bpm=120
    [S] 1 2 3 4'

    [S] 5. 6_ 7 1
    """,
"#;

/// The text of every `note-head` `<text>` element, in document order.
fn note_head_texts(svg: &str) -> Vec<String> {
    svg.split("<text")
        .skip(1)
        .filter(|element| element.contains("data-variant=\"note-head\""))
        .filter_map(|element| {
            let (_, after_open) = element.split_once('>')?;
            let (text, _) = after_open.split_once("</text>")?;
            Some(text.to_string())
        })
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
fn letter_names_replace_the_degree_digits_of_every_note_head() {
    assert_eq!(
        note_head_texts(&render(NoteLabels::LetterNames)),
        ["D", "E", "F\u{266F}", "G", "A", "B", "C\u{266F}", "D"]
    );
}

#[test]
fn guitar_chords_follow_every_note_head_s_digit_with_its_letter_name() {
    assert_eq!(
        note_head_texts(&render(NoteLabels::GuitarChords)),
        [
            "1 (D)",
            "2 (E)",
            "3 (F\u{266F})",
            "4 (G)",
            "5 (A)",
            "6 (B)",
            "7 (C\u{266F})",
            "1 (D)"
        ]
    );
}

#[test]
fn degrees_keep_the_digits() {
    assert_eq!(
        note_head_texts(&render(NoteLabels::Degrees)),
        ["1", "2", "3", "4", "5", "6", "7", "1"]
    );
}

#[test]
fn letter_names_keep_octave_dots_and_augmentation_dots() {
    let count_dots = |svg: &str| {
        svg.split("<circle")
            .skip(1)
            .filter(|element| element.contains("data-variant=\"note-head\""))
            .count()
    };
    assert_eq!(
        count_dots(&render(NoteLabels::LetterNames)),
        count_dots(&render(NoteLabels::Degrees))
    );
    assert!(count_dots(&render(NoteLabels::Degrees)) >= 2);
}
