use super::{midi_note_name, parse_midi_note_name};

#[test]
fn reads_naturals_flats_and_sharps() {
    assert_eq!(parse_midi_note_name("C4"), Some(60));
    assert_eq!(parse_midi_note_name("c3"), Some(48));
    assert_eq!(parse_midi_note_name("Bb3"), Some(58));
    assert_eq!(parse_midi_note_name("F#3"), Some(54));
    assert_eq!(parse_midi_note_name("G9"), Some(127));
}

#[test]
fn rejects_what_is_not_a_pitch() {
    for text in ["", "C", "H3", "C#", "Cx3", "C3.5", "G#9", "violin"] {
        assert_eq!(parse_midi_note_name(text), None, "{text}");
    }
}

#[test]
fn names_read_back_as_the_same_note() {
    // From C0: an octave below 0 has no spelling here.
    for midi in 12..=127 {
        assert_eq!(parse_midi_note_name(&midi_note_name(midi)), Some(midi));
    }
}
