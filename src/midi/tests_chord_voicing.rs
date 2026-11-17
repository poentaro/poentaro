use super::*;

fn note_ons_of(input: &str) -> Vec<u8> {
    let doc = crate::parser::parse(input, "test", &[]).unwrap();
    let score = crate::grouper::group(doc).unwrap();
    note_on_keys(&write_midi(&score).unwrap())
}

#[test]
fn first_chord_sounds_bass_plus_root_position() {
    let input = r#"metadata { title "", author "" },

parts ["C": chords],

score """
    time=4/4 key=C4 bpm=120
    [C] 1
    """,
"#;
    assert_eq!(
        note_ons_of(input),
        vec![48, 60, 64, 67],
        "1 in C should be bass C3 plus C4 E4 G4"
    );
}

#[test]
fn next_chord_uses_the_nearest_inversion() {
    let input = r#"metadata { title "", author "" },

parts ["C": chords],

score """
    time=4/4 key=C4 bpm=120
    [C] 1 5
    """,
"#;
    assert_eq!(
        note_ons_of(input),
        vec![48, 60, 64, 67, 43, 59, 62, 67],
        "1 → 5 should keep the common tone G and move to B D G, over bass G2"
    );
}

#[test]
fn slash_chord_bass_replaces_the_root_in_the_bass() {
    let input = r#"metadata { title "", author "" },

parts ["C": chords],

score """
    time=4/4 key=C4 bpm=120
    [C] 1/5
    """,
"#;
    assert_eq!(
        note_ons_of(input),
        vec![43, 60, 64, 67],
        "1/5 should be bass G2 plus C4 E4 G4"
    );
}

#[test]
fn seventh_chord_sounds_bass_plus_four_upper_voices() {
    let input = r#"metadata { title "", author "" },

parts ["C": chords],

score """
    time=4/4 key=C4 bpm=120
    [C] 57
    """,
"#;
    assert_eq!(
        note_ons_of(input),
        vec![43, 55, 59, 62, 65],
        "57 in C should be bass G2 plus G3 B3 D4 F4"
    );
}

#[test]
fn voice_leading_carries_across_measures_and_rests() {
    let input = r#"metadata { title "", author "" },

parts ["C": chords],

score """
    time=4/4 key=C4 bpm=120
    [C] 1 - - 0

    [C] 5 - - -
    """,
"#;
    assert_eq!(
        note_ons_of(input),
        vec![48, 60, 64, 67, 43, 59, 62, 67],
        "the rest and barline shouldn't reset 5 back to root position"
    );
}

#[test]
fn voice_leading_stays_within_a_fixed_range() {
    // A cycle of fourths would keep drifting downward forever if every chord just took
    // the nearest inversion with no range limit.
    let input = r#"metadata { title "", author "" },

parts ["C": chords],

score """
    time=4/4 key=C4 bpm=120
    [C] 1 4 7b 3b

    [C] 6b 2b 5b 7

    [C] 3 6 2 5

    [C] 1 4 7b 3b
    """,
"#;
    let notes = note_ons_of(input);
    assert_eq!(notes.len(), 16 * 4);
    assert!(
        notes.chunks(4).all(|chord| (43..=54).contains(&chord[0])
            && (55..=66).contains(&chord[1])
            && chord[3] <= 77),
        "chord drifted out of range: {notes:?}"
    );
}
