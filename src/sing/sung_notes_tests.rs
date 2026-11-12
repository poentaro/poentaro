use super::{sung_notes, SungNote, SungSyllable};

fn source_of(melody: &str, score_body: &str) -> String {
    format!(
        "parts [\"Melody\" [M]: {melody}, \"Lyrics\" [L]: lyrics [M]],\nscore \"\"\"\n{score_body}\"\"\",\n"
    )
}

fn notes_of(score_body: &str) -> Vec<SungNote> {
    let source = source_of("notes { sound: chimera soprano }", score_body);
    let score = crate::compile(&source, "test.poentaro", &[]).expect("compiles");
    sung_notes(&score, "M").expect("sung notes")
}

fn word(text: &str) -> SungSyllable {
    SungSyllable::Word(text.to_string())
}

#[test]
fn pairs_each_note_with_its_syllable_at_its_time_and_pitch() {
    let notes = notes_of("time=4/4 key=C4 bpm=60\n[M] 1 2 3 4\n[L] 我 们 有 约\n");
    assert_eq!(
        notes
            .iter()
            .map(|note| (
                note.start_seconds,
                note.end_seconds,
                note.midi,
                note.syllable.clone()
            ))
            .collect::<Vec<_>>(),
        vec![
            (0.0, 1.0, 60, word("我")),
            (1.0, 2.0, 62, word("们")),
            (2.0, 3.0, 64, word("有")),
            (3.0, 4.0, 65, word("约")),
        ]
    );
}

#[test]
fn a_tie_onto_the_same_pitch_lengthens_the_note_across_the_bar() {
    let notes = notes_of("time=2/4 key=C4 bpm=60\n[M] 1 5~\n[L] 记 得\n\n[M] 5 3\n[L] 我\n");
    assert_eq!(
        notes
            .iter()
            .map(|note| (note.start_seconds, note.end_seconds, note.syllable.clone()))
            .collect::<Vec<_>>(),
        vec![
            (0.0, 1.0, word("记")),
            (1.0, 3.0, word("得")),
            (3.0, 4.0, word("我")),
        ]
    );
}

#[test]
fn a_held_marker_carries_the_vowel_on_into_a_new_pitch() {
    let notes = notes_of("time=4/4 key=C4 bpm=60\n[M] 1 2 (3 4)\n[L] 有 - 约 -\n");
    assert_eq!(
        notes
            .iter()
            .map(|note| note.syllable.clone())
            .collect::<Vec<_>>(),
        vec![
            word("有"),
            SungSyllable::Held,
            word("约"),
            SungSyllable::Held
        ]
    );
}

#[test]
fn a_no_lyrics_measure_sings_nothing() {
    let notes = notes_of("time=2/4 key=C4 bpm=60\n[M] 1 2\n[L] _\n");
    assert!(notes
        .iter()
        .all(|note| note.syllable == SungSyllable::Silent));
}

#[test]
fn a_bpm_change_moves_later_notes_and_octave_offset_shifts_pitch() {
    let source = source_of(
        "notes { sound: chimera soprano, octave: - 1 }",
        "time=2/4 key=C4 bpm=60\n[M] 1 1\n[L] 一 二\n\nbpm=120\n[M] 1 1\n[L] 三 四\n",
    );
    let score = crate::compile(&source, "test.poentaro", &[]).expect("compiles");
    let notes = sung_notes(&score, "M").expect("sung notes");
    assert_eq!(
        notes
            .iter()
            .map(|note| (note.start_seconds, note.midi))
            .collect::<Vec<_>>(),
        vec![(0.0, 48), (1.0, 48), (2.0, 48), (2.5, 48)]
    );
}

#[test]
fn a_hidden_singer_sings_nothing() {
    let source = source_of(
        "notes { sound: chimera soprano }",
        "time=2/4 key=C4 bpm=60\n[M] 1 1\n[L] 一 二\n",
    );
    let mut score = crate::compile(&source, "test.poentaro", &[]).expect("compiles");
    crate::filter_tracks(&mut score, &["L".to_string()]);
    assert!(sung_notes(&score, "M").expect("sung notes").is_empty());
}

#[test]
fn a_singer_holds_the_syllable_over_notes_slurred_where_the_paired_part_has_one_note() {
    let source = "parts [\"S\" [S]: notes { sound: chimera soprano }, \"A\" [A]: notes, \"Words\" [i]: lyrics [S, A]],\nscore \"\"\"\ntime=3/4 key=C4 bpm=60\n[S] 3 3 (2_1_)\n[A] 6,6,5,\n[i] 我們道\n\"\"\",\n";
    let score = crate::compile(source, "test.poentaro", &[]).expect("compiles");
    let notes = sung_notes(&score, "S").expect("sung notes");
    assert_eq!(
        notes
            .iter()
            .map(|note| (note.start_seconds, note.midi, note.syllable.clone()))
            .collect::<Vec<_>>(),
        vec![
            (0.0, 64, word("我")),
            (1.0, 64, word("們")),
            (2.0, 62, word("道")),
            (2.5, 60, SungSyllable::Held),
        ]
    );
}

#[test]
fn a_slur_into_a_measure_without_words_holds_the_syllable_to_its_end() {
    let notes = notes_of("time=3/4 key=C4 bpm=60\n[M] (1--~\n[L] 露\n\n[M] 1_2_3_2_1_2_)\n");
    assert_eq!(
        notes
            .iter()
            .map(|note| note.syllable.clone())
            .collect::<Vec<_>>(),
        vec![
            word("露"),
            SungSyllable::Held,
            SungSyllable::Held,
            SungSyllable::Held,
            SungSyllable::Held,
            SungSyllable::Held,
        ]
    );
}
