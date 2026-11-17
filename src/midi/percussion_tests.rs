use super::write_midi;
use midly::{MidiMessage, Smf, TrackEventKind};

fn parse_write(input: &str) -> Vec<u8> {
    let doc = crate::parser::parse(input, "test", &[]).unwrap();
    let score = crate::grouper::group(doc).unwrap();
    write_midi(&score).unwrap()
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

fn note_off_events(midi_bytes: &[u8]) -> Vec<(u8, u8)> {
    let smf = Smf::parse(midi_bytes).expect("valid MIDI");
    smf.tracks
        .iter()
        .flat_map(|t| t.iter())
        .filter_map(|e| match e.kind {
            TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOff { key, .. },
            } => Some((channel.as_int(), key.as_int())),
            TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOn { key, vel },
            } if vel.as_int() == 0 => Some((channel.as_int(), key.as_int())),
            _ => None,
        })
        .collect()
}

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

fn controller_events_on_channel(midi_bytes: &[u8], target_channel: u8) -> usize {
    let smf = Smf::parse(midi_bytes).expect("valid MIDI");
    smf.tracks
        .iter()
        .flat_map(|t| t.iter())
        .filter(|e| match e.kind {
            TrackEventKind::Midi {
                channel,
                message: MidiMessage::Controller { .. },
            } => channel.as_int() == target_channel,
            _ => false,
        })
        .count()
}

fn two_part_fixture() -> &'static str {
    r#"metadata { title "t", author "a" },

parts ["Snare": percussion { sound 38 }, "Kick": percussion { sound 36 }],

score """
    time=4/4 key=C4 bpm=120
    [Snare] x 0 x 0
    [Kick] x 0 x 0
    """,
"#
}

#[test]
fn both_percussion_parts_route_to_channel_9_with_correct_keys() {
    let midi_bytes = parse_write(two_part_fixture());
    let events = note_on_events(&midi_bytes);
    assert_eq!(
        events.len(),
        4,
        "2 hits per part x 2 parts = 4 NoteOn events"
    );
    for (channel, _) in &events {
        assert_eq!(
            *channel, 9,
            "all percussion hits must be on GM drum channel 9"
        );
    }
    let keys: Vec<u8> = events.iter().map(|(_, key)| *key).collect();
    assert!(keys.contains(&38), "snare key 38 must appear: {keys:?}");
    assert!(keys.contains(&36), "kick key 36 must appear: {keys:?}");
}

#[test]
fn exactly_one_program_change_on_channel_9_regardless_of_percussion_part_count() {
    let midi_bytes = parse_write(two_part_fixture());
    let program_changes: Vec<(u8, u8)> = program_change_events(&midi_bytes)
        .into_iter()
        .filter(|(channel, _)| *channel == 9)
        .collect();
    assert_eq!(
        program_changes.len(),
        1,
        "exactly one GM Standard Kit program change should be emitted on channel 9 \
         even with two percussion parts, got {program_changes:?}"
    );
    assert_eq!(program_changes[0].1, 0, "GM Standard Kit is program 0");
}

#[test]
fn no_volume_controller_events_on_percussion_channel() {
    let midi_bytes = parse_write(two_part_fixture());
    assert_eq!(
        controller_events_on_channel(&midi_bytes, 9),
        0,
        "percussion shares channel 9, so no per-part CC7 volume should be emitted on it"
    );
}

#[test]
fn melodic_channel_numbering_unaffected_by_percussion_presence() {
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Snare": percussion { sound 38 }],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Snare] x 0 x 0
    """,
"#;
    let midi_bytes = parse_write(input);
    let events = note_on_events(&midi_bytes);
    let melody_channels: Vec<u8> = events
        .iter()
        .filter(|(_, key)| *key < 36 || *key > 81) // outside typical GM percussion key range
        .map(|(channel, _)| *channel)
        .collect();
    assert!(
        melody_channels.iter().all(|c| *c == 0),
        "melody should keep using channel 0 regardless of percussion parts, got {melody_channels:?}"
    );
}

#[test]
fn tied_hit_defers_note_off_until_tie_resolves() {
    let input = r#"metadata { title "t", author "a" },

parts ["Snare": percussion { sound 38 }],

score """
    time=4/4 key=C4 bpm=120
    [Snare] x~x 0 0
    """,
"#;
    let midi_bytes = parse_write(input);
    let note_offs: Vec<(u8, u8)> = note_off_events(&midi_bytes)
        .into_iter()
        .filter(|(channel, key)| *channel == 9 && *key == 38)
        .collect();
    assert_eq!(
        note_offs.len(),
        1,
        "tied hits should merge into a single NoteOff after the tie resolves, got {note_offs:?}"
    );
}
