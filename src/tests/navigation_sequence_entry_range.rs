use super::*;

fn sequence_omission_source() -> &'static str {
    r#"metadata { title "t", author "a" },

parts ["a": notes, "b": notes],

sequence ["X", "Y" WITHOUT [b], "Y", "X"],

score """
    time=4/4 key=C4 bpm=120 label="X"
    [a] 1

    label="Y"
    [a] 2
    [b] 5
    """,
"#
}

#[test]
fn play_selected_sequence_entry_range_respects_part_omission() {
    // Reproduces a user report: clicking the `Y(-b)` entry on the sequence
    // jump toolbar and pressing "play selected sequence range" still played
    // part `b`, even though `Y(-b)` marks it omitted for that occurrence.
    //
    // The sequence-jump toolbar's "play selected sequence range" action
    // (`playFromCurrentMeasure` in web/src/hooks/useMeasureAudioPlayback.ts)
    // identifies the selected `Y(-b)` entry by its 0-based `sequence`
    // entry index (1: `X, Y(-b), Y, X`) and calls `playMeasureRange(...,
    // respectSequence: true)` with that index, so `sequence`'s
    // per-occurrence part omissions apply to the generated MIDI, not just
    // the written score.
    let source = sequence_omission_source();
    let score = compile(source, "test.poentaro", &[]).unwrap();

    let (expanded, start, end) =
        midi::expand_for_measure_range(&score, 1, 1, false, true, Some(1..=1)).unwrap();
    let midi = midi::write_midi_for_measure_range(&expanded, start, end).unwrap();

    assert_eq!(
        count_note_on_events(&midi),
        1,
        "selecting the `Y(-b)` sequence entry must omit part b's note, playing only part a's"
    );
}

#[test]
fn play_selected_sequence_entry_range_disambiguates_repeated_label() {
    // Reproduces a second user report on the same score: clicking the
    // third sequence entry (the unmarked `Y`, entry index 2 in `X, Y(-b),
    // Y, X`) played only one part instead of both.
    //
    // Both `Y(-b)` and `Y` share the same written measure range (`Y` is
    // `score.measures[1]`), so resolving the selection by written index
    // alone always finds `Y(-b)` -- the *first* occurrence -- regardless of
    // which one was actually clicked. Passing the entry's own `sequence`
    // index (2) instead must select this exact occurrence.
    let source = sequence_omission_source();
    let score = compile(source, "test.poentaro", &[]).unwrap();

    let (expanded, start, end) =
        midi::expand_for_measure_range(&score, 1, 1, false, true, Some(2..=2)).unwrap();
    let midi = midi::write_midi_for_measure_range(&expanded, start, end).unwrap();

    assert_eq!(
        count_note_on_events(&midi),
        2,
        "selecting the unmarked `Y` sequence entry must play both parts a and b"
    );
}
