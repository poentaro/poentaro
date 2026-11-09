use super::write_midi_for_measure_range;
use crate::audio_source::MeasureRangeSelection;
use midly::{MidiMessage, Smf, TrackEventKind};

fn program_change_events(midi_bytes: &[u8]) -> Vec<(u8, u8)> {
    let smf = Smf::parse(midi_bytes).expect("valid MIDI");
    smf.tracks
        .iter()
        .flat_map(|t| t.iter())
        .filter_map(|e| match e.kind {
            TrackEventKind::Midi {
                channel,
                message: MidiMessage::ProgramChange { program },
            } => Some((channel.as_int(), program.as_int())),
            _ => None,
        })
        .collect()
}

fn note_on_events(midi_bytes: &[u8]) -> Vec<(u8, u8)> {
    let smf = Smf::parse(midi_bytes).expect("valid MIDI");
    smf.tracks
        .iter()
        .flat_map(|t| t.iter())
        .filter_map(|e| match e.kind {
            TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOn { key, vel },
            } if vel.as_int() > 0 => Some((channel.as_int(), key.as_int())),
            _ => None,
        })
        .collect()
}

/// `sequence` is `X, Y(-b), Y, X`: part `b`'s note (`5`) is omitted from
/// the first `Y` occurrence but present on the second. Selecting the
/// `Y(-b), Y` sequence-entry range's first measure is exactly the occurrence
/// that omits `b` — regression coverage for the bug where that measure being
/// first meant `b`'s instrument (Choir Aahs) never got a program change, so
/// its note on the *second* `Y` occurrence fell back to GM channel default
/// (Acoustic Grand Piano).
fn fixture() -> &'static str {
    r#"metadata { title "t", author "a" },

parts ["a": notes { sound 0 }, "b": notes { sound 52 }],

sequence ["X", "Y" WITHOUT [b], "Y", "X"],

score """
    label="X"
    [a]1

    label="Y"
    [a]2
    [b]5
    """,
"#
}

fn write_midi_for_sequence_entry_range(source: &str, start: usize, end: usize) -> Vec<u8> {
    let score = crate::compile(source, "test", &[]).unwrap();
    let selection = MeasureRangeSelection {
        range: start..=end,
        extend_to_last_occurrence: false,
        respect_sequence: true,
        sequence_entry_range: Some(start..=end),
    };
    let (score, resolved_start, resolved_end) = crate::midi::expand_for_measure_range(
        &score,
        *selection.range.start(),
        *selection.range.end(),
        selection.extend_to_last_occurrence,
        selection.respect_sequence,
        selection.sequence_entry_range,
    )
    .unwrap();
    write_midi_for_measure_range(&score, resolved_start, resolved_end).unwrap()
}

#[test]
fn part_b_keeps_its_own_instrument_when_first_selected_measure_omits_it() {
    // Sequence entries 1 and 2 are `Y(-b)` then `Y`.
    let midi_bytes = write_midi_for_sequence_entry_range(fixture(), 1, 2);

    let programs = program_change_events(&midi_bytes);
    assert!(
        programs.contains(&(1, 52)),
        "part b's channel must get a Choir Aahs (program 52) program change \
         even though it's absent from the range's first measure, got {programs:?}"
    );

    let notes = note_on_events(&midi_bytes);
    // Degree 5 in key C4 is MIDI note 67 (see `degree_five_c4_is_g4`).
    let note_67_channels: Vec<u8> = notes
        .iter()
        .filter(|(_, key)| *key == 67)
        .map(|(channel, _)| *channel)
        .collect();
    assert_eq!(
        note_67_channels,
        vec![1],
        "part b's note 5 must sound on part b's own channel (with Choir Aahs), not channel 0 (piano), got {notes:?}"
    );
}
