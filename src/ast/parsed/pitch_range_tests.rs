use super::{midi_note_name, parse_midi_note_name, PitchRange};

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

fn range(lowest: u8, highest: u8) -> Option<PitchRange> {
    Some(PitchRange { lowest, highest })
}

#[test]
fn picked_bounds_fill_from_the_default_then_each_other_and_order() {
    let tenor = range(48, 67);
    assert_eq!(PitchRange::from_bounds(None, None, tenor), None);
    assert_eq!(
        PitchRange::from_bounds(Some(50), None, tenor),
        range(50, 67)
    );
    assert_eq!(
        PitchRange::from_bounds(None, Some(65), tenor),
        range(48, 65)
    );
    assert_eq!(PitchRange::from_bounds(Some(50), None, None), range(50, 50));
    assert_eq!(
        PitchRange::from_bounds(Some(70), None, tenor),
        range(67, 70)
    );
    assert_eq!(
        PitchRange::from_bounds(Some(60), Some(55), None),
        range(55, 60)
    );
}

#[test]
fn source_text_quotes_sharps() {
    assert_eq!(range(48, 67).unwrap().source_text(), "C3 to G4");
    assert_eq!(range(54, 70).unwrap().source_text(), "\"F#3\" to \"A#4\"");
}
