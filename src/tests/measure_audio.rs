use super::write_wav_for_measure_range_from_source;
use super::MeasureRangeAudioOptions;
use super::MeasureRangeSelection;
use super::*;

#[cfg(feature = "wav")]
static SF2_BYTES: &[u8] = include_bytes!("../../fonts/GeneralUser_GS.sf2");

fn two_measure_source() -> &'static str {
    r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4

    [Melody] 5 6 7 1
    """,
"#
}

#[test]
fn find_measure_at_byte_offset_finds_first_measure() {
    let source = two_measure_source();
    let score = compile(source, "test.poentaro", &[]).unwrap();
    let first_note_offset = source.find("1 2 3 4").unwrap();
    assert_eq!(
        find_measure_at_byte_offset(&score, first_note_offset),
        Some(0)
    );
}

#[test]
fn find_measure_at_byte_offset_finds_second_measure() {
    let source = two_measure_source();
    let score = compile(source, "test.poentaro", &[]).unwrap();
    let second_note_offset = source.rfind("5 6 7 1").unwrap();
    assert_eq!(
        find_measure_at_byte_offset(&score, second_note_offset),
        Some(1)
    );
}

#[test]
fn find_measure_at_byte_offset_returns_none_for_header() {
    let source = two_measure_source();
    let score = compile(source, "test.poentaro", &[]).unwrap();
    assert_eq!(find_measure_at_byte_offset(&score, 0), None);
}

fn two_part_source() -> &'static str {
    r#"metadata { title "t", author "a" },

parts ["Chord": chords, "Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Chord] 1
    [Melody] 1 2 3 4

    [Chord] 5
    [Melody] 5 6 7 1
    """,
"#
}

#[test]
fn find_measure_at_byte_offset_finds_measure_on_second_part_line() {
    let source = two_part_source();
    let score = compile(source, "test.poentaro", &[]).unwrap();
    // The cursor is on the Melody (second part) line "1 2 3 4", not the Chord line "1"
    let melody_note_offset = source.find("1 2 3 4").unwrap();
    assert_eq!(
        find_measure_at_byte_offset(&score, melody_note_offset),
        Some(0),
        "cursor on second part's line should still find the correct measure"
    );
}

fn twinkle_source() -> &'static str {
    r#"metadata { title "Twinkle Twinkle Little Star", author "Mozart", row height 24 },

parts ["Chord": chords, "Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Chord] 1 - - -
    [Melody] 1 1 5 5
    [Melodyv1] twin- kle twin- kle

    [Chord] 5 - - -
    [Melody] 6 6 5-
    [Melodyv1] lit- tle star

    [Chord] 4 - - -
    [Melody] 4 4 3 3
    [Melodyv1] how I won- der

    [Chord] 4 - - -
    [Melody] 2 2 1-
    [Melodyv1] what you are
    """,
"#
}

#[test]
fn find_measure_at_byte_offset_covers_all_lines_in_twinkle_star() {
    let source = twinkle_source();
    let score = compile(source, "test.poentaro", &[]).unwrap();
    assert_eq!(score.measures.len(), 4, "expected 4 measures");

    let check = |label: &str, offset: usize, expected: Option<usize>| {
        assert_eq!(
            find_measure_at_byte_offset(&score, offset),
            expected,
            "{label} (offset {offset})"
        );
    };

    // Measure 0: chord "1 - - -", melody "1 1 5 5", lyrics "twin- kle twin- kle"
    check(
        "measure 0 chord line",
        source.find("1 - - -").unwrap(),
        Some(0),
    );
    check(
        "measure 0 melody line",
        source.find("1 1 5 5").unwrap(),
        Some(0),
    );
    check(
        "measure 0 lyrics line",
        source.find("twin- kle twin- kle").unwrap(),
        Some(0),
    );

    // Measure 1: chord "5 - - -", melody "6 6 5-", lyrics "lit- tle star"
    check(
        "measure 1 chord line",
        source.find("5 - - -").unwrap(),
        Some(1),
    );
    check(
        "measure 1 melody line",
        source.find("6 6 5-").unwrap(),
        Some(1),
    );
    check(
        "measure 1 lyrics line",
        source.find("lit- tle star").unwrap(),
        Some(1),
    );

    // Measure 2: first "4 - - -", melody "4 4 3 3", lyrics "how I won- der"
    check(
        "measure 2 chord line",
        source.find("4 - - -").unwrap(),
        Some(2),
    );
    check(
        "measure 2 melody line",
        source.find("4 4 3 3").unwrap(),
        Some(2),
    );
    check(
        "measure 2 lyrics line",
        source.find("how I won- der").unwrap(),
        Some(2),
    );

    // Measure 3: second "4 - - -", melody "2 2 1-", lyrics "what you are"
    check(
        "measure 3 chord line",
        source.rfind("4 - - -").unwrap(),
        Some(3),
    );
    check(
        "measure 3 melody line",
        source.find("2 2 1-").unwrap(),
        Some(3),
    );
    check(
        "measure 3 lyrics line",
        source.find("what you are").unwrap(),
        Some(3),
    );
}

#[test]
fn find_measure_at_byte_offset_detects_measure_when_cursor_is_one_past_last_char() {
    // Regression: cursor placed directly after the last character of a measure
    // line (e.g. after "4" in "1 2 3 4") must still resolve to that measure.
    let source = two_measure_source();
    let score = compile(source, "test.poentaro", &[]).unwrap();
    let after_last_char = source.find("1 2 3 4").unwrap() + "1 2 3 4".len();
    assert_eq!(
        find_measure_at_byte_offset(&score, after_last_char),
        Some(0),
        "cursor one byte past last char should still detect the measure (offset {after_last_char})"
    );
}

#[cfg(feature = "wav")]
#[test]
fn write_wav_for_measure_range_from_source_second_measure_uses_context_key() {
    // Two-measure source where second measure has no explicit key directive.
    let source = two_measure_source();
    let result = write_wav_for_measure_range_from_source(
        source,
        "test.poentaro",
        &MeasureRangeSelection {
            range: 1..=1,
            extend_to_last_occurrence: false,
            respect_sequence: true,
            sequence_entry_range: None,
        },
        &MeasureRangeAudioOptions {
            enabled_tracks: None,
            metronome: false,
            trim: None,
        },
        SF2_BYTES,
        &[],
    );
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
}

#[cfg(feature = "wav")]
#[test]
fn write_wav_for_measure_range_from_source_returns_riff_wav() {
    let source = two_measure_source();
    let wav = write_wav_for_measure_range_from_source(
        source,
        "test.poentaro",
        &MeasureRangeSelection {
            range: 0..=1,
            extend_to_last_occurrence: false,
            respect_sequence: true,
            sequence_entry_range: None,
        },
        &MeasureRangeAudioOptions {
            enabled_tracks: None,
            metronome: false,
            trim: None,
        },
        SF2_BYTES,
        &[],
    )
    .unwrap();
    assert!(wav.len() > 4);
    assert_eq!(&wav[0..4], b"RIFF");
}

#[cfg(feature = "wav")]
#[test]
fn write_wav_for_measure_range_from_source_out_of_range_clamps_to_last_measure() {
    let source = two_measure_source();
    let result = write_wav_for_measure_range_from_source(
        source,
        "test.poentaro",
        &MeasureRangeSelection {
            range: 0..=99,
            extend_to_last_occurrence: false,
            respect_sequence: true,
            sequence_entry_range: None,
        },
        &MeasureRangeAudioOptions {
            enabled_tracks: None,
            metronome: false,
            trim: None,
        },
        SF2_BYTES,
        &[],
    );
    assert!(
        result.is_ok(),
        "out-of-range range end must clamp instead of failing"
    );
}
