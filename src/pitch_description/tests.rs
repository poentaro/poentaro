use super::*;

/// Byte range of the `occurrence`-th (0-based) match of `token` on the line
/// starting with `line_prefix`.
fn token_range(source: &str, line_prefix: &str, token: &str, occurrence: usize) -> ByteRange {
    let line_start = source.find(&format!("\n    {line_prefix}")).unwrap() + 5 + line_prefix.len();
    let line_end = line_start + source[line_start..].find('\n').unwrap();
    let start = line_start
        + source[line_start..line_end]
            .match_indices(token)
            .nth(occurrence)
            .unwrap()
            .0;
    ByteRange {
        start_byte: start as u32,
        end_byte: (start + token.len()) as u32,
    }
}

fn notes_score(key: &str, notes: &str) -> String {
    format!(
        r#"parts ["Melody" [M]: notes],

score """
    key={key}
    [M] {notes}
    """,
"#
    )
}

fn chords_score(key: &str, chords: &str) -> String {
    format!(
        r#"parts ["Chords" [C]: chords],

score """
    key={key}
    [C] {chords}
    """,
"#
    )
}

fn describe_note(source: &str, line_prefix: &str, token: &str, occurrence: usize) -> String {
    let range = token_range(source, line_prefix, token, occurrence);
    match describe_selection(source, &[range]) {
        Some(PitchDescription::Note(note)) => note.letter_name,
        other => panic!("expected a note description, got {other:?}"),
    }
}

fn describe_chord_token(source: &str, token: &str) -> ChordDescription {
    let range = token_range(source, "[C] ", token, 0);
    match describe_selection(source, &[range]) {
        Some(PitchDescription::Chord(chord)) => chord,
        other => panic!("expected a chord description, got {other:?}"),
    }
}

#[test]
fn note_one_in_c_is_c() {
    let source = notes_score("C4", "1 2 3 4");
    assert_eq!(describe_note(&source, "[M] ", "1", 0), "C");
}

#[test]
fn note_seven_in_g_is_spelled_f_sharp() {
    let source = notes_score("G4", "7 1 2 3");
    assert_eq!(describe_note(&source, "[M] ", "7", 0), "F#");
}

#[test]
fn note_four_in_f_is_spelled_b_flat() {
    let source = notes_score("F4", "4 5 6 7");
    assert_eq!(describe_note(&source, "[M] ", "4", 0), "Bb");
}

#[test]
fn written_accidental_is_applied() {
    let source = notes_score("C4", "4# 7b 1 1");
    assert_eq!(describe_note(&source, "[M] ", "4#", 0), "F#");
    assert_eq!(describe_note(&source, "[M] ", "7b", 0), "Bb");
}

#[test]
fn flat_key_tonic_is_spelled_with_flat() {
    let source = notes_score("Bb4", "1 3 1 1");
    assert_eq!(describe_note(&source, "[M] ", "1", 0), "Bb");
    assert_eq!(describe_note(&source, "[M] ", "3", 0), "D");
}

#[test]
fn octave_markers_do_not_change_the_letter_name() {
    let source = notes_score("C4", "1' 5, 1 1");
    assert_eq!(describe_note(&source, "[M] ", "5,", 0), "G");
}

#[test]
fn letter_name_follows_the_measure_key() {
    let source = r#"parts ["Melody" [M]: notes],

score """
    key=C4
    [M] 1 2 3 4

    key=G4
    [M] 1 2 3 4
    """,
"#;
    assert_eq!(describe_note(source, "key=G4\n    [M] ", "1", 0), "G");
}

#[test]
fn minor_seventh_chord_tones_and_name() {
    let source = chords_score("C4", "1m7 - - -");
    let chord = describe_chord_token(&source, "1m7");
    assert_eq!(chord.chord_name, "Cm7");
    assert_eq!(chord.tone_names, ["C", "Eb", "G", "Bb"]);
    assert_eq!(chord.bass_note, None);
}

#[test]
fn minor_seventh_chord_has_chords_db_first_voicing_first() {
    let source = chords_score("C4", "1m7 - - -");
    let svg = describe_chord_token(&source, "1m7")
        .guitar_diagram_svgs
        .into_iter()
        .next()
        .unwrap();
    assert!(svg.contains(r#"data-guitar-frets="x 3 1 3 4 x""#), "{svg}");
    assert!(svg.contains(r#"data-guitar-base-fret="1""#), "{svg}");
}

fn guitar_diagram_frets(chord: &ChordDescription) -> Vec<String> {
    chord
        .guitar_diagram_svgs
        .iter()
        .map(|svg| {
            let (_, after) = svg.split_once(r#"data-guitar-frets=""#).unwrap();
            let (frets, _) = after.split_once('"').unwrap();
            frets.to_string()
        })
        .collect()
}

#[test]
fn major_chord_shows_every_chords_db_voicing_open_shape_first() {
    let source = chords_score("C4", "1 - - -");
    let chord = describe_chord_token(&source, "1");
    assert_eq!(
        guitar_diagram_frets(&chord),
        ["x 3 2 0 1 0", "3 3 5 5 5 3", "x x 5 5 5 8", "8 10 10 9 8 8",]
    );
}

#[test]
fn chord_without_open_shape_shows_every_barred_voicing() {
    let source = chords_score("C4", "4 - - -");
    let chord = describe_chord_token(&source, "4");
    assert_eq!(
        guitar_diagram_frets(&chord),
        [
            "1 3 3 2 1 1",
            "x 3 3 5 6 5",
            "5 8 7 5 6 5",
            "8 8 10 10 10 8",
        ]
    );
}

#[test]
fn barre_voicing_up_the_neck_reports_absolute_frets() {
    // chords-db's first Gm7 position is a barre with base fret 3.
    let source = chords_score("C4", "5m7 - - -");
    let svg = describe_chord_token(&source, "5m7")
        .guitar_diagram_svgs
        .into_iter()
        .next()
        .unwrap();
    assert!(svg.contains(r#"data-guitar-frets="3 5 3 3 3 3""#), "{svg}");
    assert!(svg.contains(r#"data-guitar-base-fret="3""#), "{svg}");
    assert!(svg.contains(">3fr</text>"), "{svg}");
    assert!(svg.contains("<rect"), "{svg}");
}

#[test]
fn chord_tones_in_a_sharp_key_use_sharps() {
    let source = chords_score("E4", "5 - - -");
    let chord = describe_chord_token(&source, "5");
    assert_eq!(chord.chord_name, "B");
    assert_eq!(chord.tone_names, ["B", "D#", "F#"]);
}

#[test]
fn diminished_seventh_symbol_is_half_diminished_like_midi_playback() {
    let source = chords_score("C4", "7o7 - - -");
    let chord = describe_chord_token(&source, "7o7");
    assert_eq!(chord.chord_name, "Bm7b5");
    assert_eq!(chord.tone_names, ["B", "D", "F", "A"]);
    assert!(!chord.guitar_diagram_svgs.is_empty());
}

#[test]
fn slash_chord_reports_bass_separately() {
    let source = chords_score("C4", "1/5 - - -");
    let chord = describe_chord_token(&source, "1/5");
    assert_eq!(chord.chord_name, "C/G");
    assert_eq!(chord.tone_names, ["C", "E", "G"]);
    assert_eq!(chord.bass_note.as_deref(), Some("G"));
    assert!(!chord.guitar_diagram_svgs.is_empty());
}

#[test]
fn slash_chord_without_slash_voicing_falls_back_to_plain_chord() {
    // chords-db has no Bb/D; the plain Bb voicing is used instead.
    let source = chords_score("F4", "4/6 - - -");
    let chord = describe_chord_token(&source, "4/6");
    assert_eq!(chord.chord_name, "Bb/D");
    assert!(!chord.guitar_diagram_svgs.is_empty());
}

#[test]
fn chord_kind_missing_from_chords_db_has_no_diagram() {
    let source = chords_score("C4", "1sus27 - - -");
    let chord = describe_chord_token(&source, "1sus27");
    assert_eq!(chord.chord_name, "C7sus2");
    assert_eq!(chord.tone_names, ["C", "D", "G", "Bb"]);
    assert!(chord.guitar_diagram_svgs.is_empty());
}

#[test]
fn major_chord_keyboard_labels_are_one_three_five() {
    let source = chords_score("D4", "1 - - -");
    let svg = describe_chord_token(&source, "1").piano_diagram_svg;
    assert!(svg.contains(r#"data-piano-tones="1:0 3:4 5:7""#), "{svg}");
}

#[test]
fn same_quality_gets_same_labels_in_any_key() {
    let in_d = describe_chord_token(&chords_score("D4", "4 - - -"), "4").piano_diagram_svg;
    let in_c = describe_chord_token(&chords_score("C4", "1 - - -"), "1").piano_diagram_svg;
    assert!(in_d.contains(r#"data-piano-tones="1:0 3:4 5:7""#), "{in_d}");
    assert!(in_c.contains(r#"data-piano-tones="1:0 3:4 5:7""#), "{in_c}");
}

#[test]
fn minor_seventh_keyboard_labels_use_flats() {
    let source = chords_score("C4", "1m7 - - -");
    let svg = describe_chord_token(&source, "1m7").piano_diagram_svg;
    assert!(
        svg.contains(r#"data-piano-tones="1:0 b3:3 5:7 b7:10""#),
        "{svg}"
    );
}

#[test]
fn augmented_keyboard_labels_use_sharp_fifth() {
    let source = chords_score("C4", "1+ - - -");
    let svg = describe_chord_token(&source, "1+").piano_diagram_svg;
    assert!(svg.contains(r#"data-piano-tones="1:0 3:4 #5:8""#), "{svg}");
}

#[test]
fn slash_bass_inside_the_triad_inverts_the_keyboard() {
    // C/G: G lowest, then C and E an octave up.
    let source = chords_score("C4", "1/5 - - -");
    let svg = describe_chord_token(&source, "1/5").piano_diagram_svg;
    assert!(svg.contains(r#"data-piano-tones="1:5 3:9 5:0""#), "{svg}");
}

#[test]
fn no_slash_bass_keeps_root_position() {
    let source = chords_score("C4", "1 - - -");
    let svg = describe_chord_token(&source, "1").piano_diagram_svg;
    assert!(svg.contains(r#"data-piano-tones="1:0 3:4 5:7""#), "{svg}");
}

#[test]
fn keyboard_spans_chord_tones_with_white_key_padding() {
    // C E G is midi 60..67: C D E F G A B C ... padded by two white keys
    // each side (A3 B3 .. A4 B4) = 9 white keys.
    let source = chords_score("C4", "1 - - -");
    let svg = describe_chord_token(&source, "1").piano_diagram_svg;
    assert!(svg.contains(r#"viewBox="0 0 198 112""#), "{svg}");
}

#[test]
fn keyboard_tones_carry_their_function_names() {
    let source = chords_score("C4", "1m7 - - -");
    let svg = describe_chord_token(&source, "1m7").piano_diagram_svg;
    for function_name in ["Root", "Minor 3rd", "Perfect 5th", "Minor 7th"] {
        assert!(
            svg.contains(&format!(r#"data-tone-function="{function_name}""#)),
            "{function_name}: {svg}"
        );
    }
}

#[test]
fn rest_is_not_described() {
    let source = notes_score("C4", "0 2 3 4");
    let range = token_range(&source, "[M] ", "0", 0);
    assert_eq!(describe_selection(&source, &[range]), None);
}

#[test]
fn two_notes_are_not_described() {
    let source = notes_score("C4", "1 2 3 4");
    let range = token_range(&source, "[M] ", "1 2", 0);
    assert_eq!(describe_selection(&source, &[range]), None);
}

#[test]
fn caret_without_selection_is_not_described() {
    let source = notes_score("C4", "1 2 3 4");
    let range = token_range(&source, "[M] ", "1", 0);
    let caret = ByteRange {
        start_byte: range.start_byte,
        end_byte: range.start_byte,
    };
    assert_eq!(describe_selection(&source, &[caret]), None);
}

#[test]
fn tied_run_counts_as_one_note() {
    let source = notes_score("C4", "3~ 3 2 1");
    let range = token_range(&source, "[M] ", "3~ 3", 0);
    assert_eq!(
        describe_selection(&source, &[range]),
        Some(PitchDescription::Note(NoteDescription {
            letter_name: "E".to_string()
        }))
    );
}
