use super::metronome::{ACCENT_KEY, BEAT_KEY};
use super::{write_midi, write_midi_for_measure_range_with_metronome, write_midi_with_metronome};
use midly::{MetaMessage, MidiMessage, Smf, TrackEventKind};

const PERCUSSION_CHANNEL: u8 = 9;

struct Click {
    tick: u32,
    key: u8,
}

fn score_source(time_and_measures: &str) -> String {
    format!(
        r#"metadata {{ title "t", author "a" }},

parts ["a": notes {{ sound 0 }}],

score """
    {time_and_measures}
    """,
"#
    )
}

fn compile(source: &str) -> crate::ast::grouped::Score {
    crate::compile(source, "test", &[]).unwrap()
}

fn clicks(midi_bytes: &[u8]) -> Vec<Click> {
    let smf = Smf::parse(midi_bytes).expect("valid MIDI");
    let mut tick = 0u32;
    smf.tracks
        .iter()
        .flat_map(|track| track.iter())
        .filter_map(|event| {
            tick += event.delta.as_int();
            match event.kind {
                TrackEventKind::Midi {
                    channel,
                    message: MidiMessage::NoteOn { key, vel },
                } if channel.as_int() == PERCUSSION_CHANNEL && vel.as_int() > 0 => Some(Click {
                    tick,
                    key: key.as_int(),
                }),
                _ => None,
            }
        })
        .collect()
}

fn click_keys(midi_bytes: &[u8]) -> Vec<u8> {
    clicks(midi_bytes).into_iter().map(|c| c.key).collect()
}

fn percussion_program_changes(midi_bytes: &[u8]) -> usize {
    Smf::parse(midi_bytes)
        .expect("valid MIDI")
        .tracks
        .iter()
        .flat_map(|track| track.iter())
        .filter(|event| {
            matches!(
                event.kind,
                TrackEventKind::Midi {
                    channel,
                    message: MidiMessage::ProgramChange { .. },
                } if channel.as_int() == PERCUSSION_CHANNEL
            )
        })
        .count()
}

#[test]
fn metronome_is_silent_by_default() {
    let score = compile(&score_source("time=4/4 key=C4 bpm=120\n[a] 1 2 3 4\n"));
    assert!(clicks(&write_midi(&score).unwrap()).is_empty());
    assert!(clicks(&write_midi_with_metronome(&score, false).unwrap()).is_empty());
}

#[test]
fn four_four_clicks_four_times_per_measure_accented_on_the_first_beat() {
    let score = compile(&score_source(
        "time=4/4 key=C4 bpm=120\n[a] 1 2 3 4\n\n[a] 5 6 7 1\n",
    ));
    let keys = click_keys(&write_midi_with_metronome(&score, true).unwrap());
    let (accent, beat) = (ACCENT_KEY, BEAT_KEY);
    assert_eq!(
        keys,
        vec![accent, beat, beat, beat, accent, beat, beat, beat]
    );
}

#[test]
fn clicks_are_one_quarter_note_apart_in_four_four() {
    let score = compile(&score_source("time=4/4 key=C4 bpm=120\n[a] 1 2 3 4\n"));
    let ticks: Vec<u32> = clicks(&write_midi_with_metronome(&score, true).unwrap())
        .into_iter()
        .map(|c| c.tick)
        .collect();
    assert_eq!(ticks, vec![0, 480, 960, 1440]);
}

#[test]
fn time_signature_sets_the_number_of_clicks_and_carries_forward() {
    let score = compile(&score_source(
        "time=3/4 key=C4 bpm=120\n[a] 1 2 3\n\n[a] 4 5 6\n",
    ));
    let keys = click_keys(&write_midi_with_metronome(&score, true).unwrap());
    let (accent, beat) = (ACCENT_KEY, BEAT_KEY);
    assert_eq!(keys, vec![accent, beat, beat, accent, beat, beat]);
}

#[test]
fn time_signature_change_mid_score_changes_the_click_pattern() {
    let score = compile(&score_source(
        "time=4/4 key=C4 bpm=120\n[a] 1 2 3 4\n\ntime=2/4\n[a] 5 6\n",
    ));
    let keys = click_keys(&write_midi_with_metronome(&score, true).unwrap());
    let (accent, beat) = (ACCENT_KEY, BEAT_KEY);
    assert_eq!(keys, vec![accent, beat, beat, beat, accent, beat]);
}

#[test]
fn compound_time_clicks_on_the_dotted_quarter() {
    let score = compile(&score_source("time=6/8 key=C4 bpm=60\n[a] 1 2 3 4 5 6\n"));
    let clicks = clicks(&write_midi_with_metronome(&score, true).unwrap());
    let ticks: Vec<u32> = clicks.iter().map(|c| c.tick).collect();
    assert_eq!(ticks, vec![0, 720]);
    assert_eq!(clicks[0].key, ACCENT_KEY);
    assert_eq!(clicks[1].key, BEAT_KEY);
}

#[test]
fn metronome_selects_the_standard_drum_kit_even_without_percussion_parts() {
    let score = compile(&score_source("time=4/4 key=C4 bpm=120\n[a] 1 2 3 4\n"));
    assert_eq!(
        percussion_program_changes(&write_midi_with_metronome(&score, true).unwrap()),
        1
    );
    assert_eq!(percussion_program_changes(&write_midi(&score).unwrap()), 0);
}

#[test]
fn metronome_shares_one_drum_kit_program_change_with_percussion_parts() {
    let source = r#"metadata { title "t", author "a" },

parts ["Snare": percussion { sound 38 }],

score """
    time=4/4 key=C4 bpm=120
    [Snare] x 0 x 0
    """,
"#;
    let score = compile(source);
    let midi_bytes = write_midi_with_metronome(&score, true).unwrap();
    assert_eq!(percussion_program_changes(&midi_bytes), 1);
    let keys = click_keys(&midi_bytes);
    assert_eq!(keys.iter().filter(|&&k| k == 38).count(), 2);
    assert_eq!(keys.iter().filter(|&&k| k == ACCENT_KEY).count(), 1);
    assert_eq!(keys.iter().filter(|&&k| k == BEAT_KEY).count(), 3);
}

#[test]
fn metronome_does_not_change_the_length_of_the_piece() {
    let score = compile(&score_source(
        "time=4/4 key=C4 bpm=120\n[a] 1 2 3 4\n\n[a] 5 6 7 1\n",
    ));
    let end_tick = |midi_bytes: &[u8]| -> u32 {
        Smf::parse(midi_bytes).unwrap().tracks[0]
            .iter()
            .map(|event| event.delta.as_int())
            .sum()
    };
    assert_eq!(
        end_tick(&write_midi(&score).unwrap()),
        end_tick(&write_midi_with_metronome(&score, true).unwrap())
    );
}

#[test]
fn measure_range_keeps_the_time_signature_set_before_the_range() {
    let score = compile(&score_source(
        "time=3/4 key=C4 bpm=120\n[a] 1 2 3\n\n[a] 4 5 6\n\n[a] 7 1 2\n",
    ));
    let keys =
        click_keys(&write_midi_for_measure_range_with_metronome(&score, 1, 2, true).unwrap());
    let (accent, beat) = (ACCENT_KEY, BEAT_KEY);
    assert_eq!(keys, vec![accent, beat, beat, accent, beat, beat]);
}

#[test]
fn tempo_events_are_unaffected_by_the_metronome() {
    let score = compile(&score_source("time=4/4 key=C4 bpm=90\n[a] 1 2 3 4\n"));
    let tempos = |midi_bytes: &[u8]| -> usize {
        Smf::parse(midi_bytes).unwrap().tracks[0]
            .iter()
            .filter(|event| matches!(event.kind, TrackEventKind::Meta(MetaMessage::Tempo(_))))
            .count()
    };
    assert_eq!(
        tempos(&write_midi(&score).unwrap()),
        tempos(&write_midi_with_metronome(&score, true).unwrap())
    );
}
