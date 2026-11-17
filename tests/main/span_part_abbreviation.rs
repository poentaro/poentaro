#![allow(clippy::disallowed_macros)]
use poentaro::lyric_spans::{
    group_selected_lyrics_into_contiguous_runs, list_lyric_spans_from_source, LyricCell,
};
use poentaro::note_spans::{
    group_selected_notes_into_contiguous_runs, list_note_spans_from_source, NoteCell,
};

const THREE_PART_SOURCE: &str = r#"metadata { title "t" },

parts ["Soprano" [S]: notes, "Alto" [A]: notes, "Bass" [B]: notes, "Bass lyrics" [Bv1]: lyrics [B]],

score """
    [S] 1 2 3 4
    [A] 5 6 7 1'
    [B] 1 1 5 5
    [Bv1] do re mi fa
    """,
"#;

fn enabled_tracks_without_alto() -> Vec<String> {
    vec!["S".to_string(), "B".to_string(), "Bv1".to_string()]
}

#[test]
fn note_spans_carry_the_abbreviation_of_the_part_after_hidden_parts_are_compacted_out() {
    let enabled_tracks = enabled_tracks_without_alto();
    let spans =
        list_note_spans_from_source(THREE_PART_SOURCE, "test.poentaro", Some(&enabled_tracks))
            .unwrap()
            .spans;

    // With Alto hidden, Bass is compacted down to index 1 — its spans must
    // still name Bass, not the declaration at index 1 (Alto).
    let bass_spans: Vec<_> = spans
        .iter()
        .filter(|span| span.source_part_index == 1)
        .collect();
    assert_eq!(bass_spans.len(), 4);
    assert!(bass_spans
        .iter()
        .all(|span| span.part_abbreviation.as_deref() == Some("B")));
    assert!(spans
        .iter()
        .filter(|span| span.source_part_index == 0)
        .all(|span| span.part_abbreviation.as_deref() == Some("S")));
    assert!(spans
        .iter()
        .all(|span| span.part_abbreviation.as_deref() != Some("A")));

    let bass_cell = NoteCell {
        source_part_index: 1,
        note_id: bass_spans[0].note_id,
    };
    let runs = group_selected_notes_into_contiguous_runs(&[bass_cell], &spans);
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].part_abbreviation.as_deref(), Some("B"));
}

#[test]
fn lyric_spans_carry_the_abbreviation_of_the_part_after_hidden_parts_are_compacted_out() {
    let enabled_tracks = enabled_tracks_without_alto();
    let spans =
        list_lyric_spans_from_source(THREE_PART_SOURCE, "test.poentaro", Some(&enabled_tracks))
            .unwrap()
            .spans;

    assert_eq!(spans.len(), 4);
    assert!(spans.iter().all(
        |span| span.source_part_index == 2 && span.part_abbreviation.as_deref() == Some("Bv1")
    ));

    let cell = LyricCell {
        source_part_index: 2,
        note_id: spans[0].note_id,
    };
    let runs = group_selected_lyrics_into_contiguous_runs(&[cell], &spans);
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].part_abbreviation.as_deref(), Some("Bv1"));
}
