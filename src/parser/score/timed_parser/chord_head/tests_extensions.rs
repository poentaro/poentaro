use super::*;

#[test]
fn parses_sharp_accidental() {
    assert_eq!(
        parse_symbol("1#"),
        chord(
            NumberedPitch::One,
            Accidental::Sharp,
            TriadQuality::Major,
            None,
            None
        )
    );
}

#[test]
fn parses_flat_accidental() {
    assert_eq!(
        parse_symbol("3b"),
        chord(
            NumberedPitch::Three,
            Accidental::Flat,
            TriadQuality::Major,
            None,
            None
        )
    );
}

#[test]
fn parses_slash_chord() {
    let bass = BassDegree {
        degree: NumberedPitch::Five,
        accidental: Accidental::Natural,
    };
    // Goes through the full pipeline (including the lexer in Chords context) so that
    // `1/5` is not mistakenly consumed as a time signature.
    assert_eq!(
        parse_symbol("1/5"),
        chord(
            NumberedPitch::One,
            Accidental::Natural,
            TriadQuality::Major,
            None,
            Some(bass)
        )
    );
}

#[test]
fn parses_slash_chord_with_accidental_bass() {
    let bass = BassDegree {
        degree: NumberedPitch::Four,
        accidental: Accidental::Flat,
    };
    assert_eq!(
        parse_symbol("1/4b"),
        chord(
            NumberedPitch::One,
            Accidental::Natural,
            TriadQuality::Major,
            None,
            Some(bass)
        )
    );
}

#[test]
fn parses_complex_slash_chord() {
    let bass = BassDegree {
        degree: NumberedPitch::Five,
        accidental: Accidental::Natural,
    };
    assert_eq!(
        parse_symbol("6m/5"),
        chord(
            NumberedPitch::Six,
            Accidental::Natural,
            TriadQuality::Minor,
            None,
            Some(bass)
        )
    );
}

#[test]
fn parses_sharp_with_dominant_seventh() {
    assert_eq!(
        parse_symbol("1#7"),
        chord(
            NumberedPitch::One,
            Accidental::Sharp,
            TriadQuality::Major,
            Some(Extension::DominantSeventh),
            None
        )
    );
}

#[test]
fn parses_flat_with_major_seventh() {
    assert_eq!(
        parse_symbol("3bM7"),
        chord(
            NumberedPitch::Three,
            Accidental::Flat,
            TriadQuality::Major,
            Some(Extension::MajorSeventh),
            None
        )
    );
}

#[test]
fn parses_sharp_minor_dominant_seventh() {
    assert_eq!(
        parse_symbol("1#m7"),
        chord(
            NumberedPitch::One,
            Accidental::Sharp,
            TriadQuality::Minor,
            Some(Extension::DominantSeventh),
            None
        )
    );
}

#[test]
fn parses_sharp_with_slash_chord() {
    let bass = BassDegree {
        degree: NumberedPitch::Five,
        accidental: Accidental::Natural,
    };
    assert_eq!(
        parse_symbol("1#/5"),
        chord(
            NumberedPitch::One,
            Accidental::Sharp,
            TriadQuality::Major,
            None,
            Some(bass)
        )
    );
}
