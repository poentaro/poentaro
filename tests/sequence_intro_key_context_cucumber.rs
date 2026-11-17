//! Cucumber harness for `tests/features/sequence_intro_key_context.feature`:
//! a `key=` directive written on a measure that `sequence` never lists
//! (an intro before the first listed section, under no label or one the
//! sequence omits) must still carry forward as context for the measures
//! that ARE played — the same accumulated-context treatment BPM/key already
//! get for a measure-range selection (see
//! `write_wav_for_measure_range_from_source_second_measure_uses_context_key`
//! in `src/tests/measure_audio.rs`). Right now `expand_navigation` simply
//! drops those measures, so the exported MIDI/WAV/MP3 plays them back in the
//! default key (C4) instead.
//!
//!
//! Clippy's `allow-*-in-tests` (clippy.toml) only recognizes `#[test]`-
//! attributed functions as test code; cucumber's `#[given]`/`#[when]`/
//! `#[then]` step functions don't qualify even though this whole file only
//! ever runs under `cargo test`. Mirrors `tests/cucumber.rs`'s
//! `#![allow(clippy::disallowed_macros)]` for the same reason.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::disallowed_macros,
    clippy::needless_pass_by_value
)]

use cucumber::gherkin::Step;
use cucumber::{given, then, when, World as _};
use midly::{MidiMessage, Smf, TrackEventKind};
use poentaro::compile;
use poentaro::midi::{expand_navigation, write_midi};

#[derive(Debug, Default, cucumber::World)]
struct AudioKeyWorld {
    source: String,
    note_on_pitches: Vec<u8>,
}

/// Every `NoteOn` (with nonzero velocity) event's MIDI key number, in the
/// order they occur in the synthesized track — same helper as
/// `src/midi/tests.rs`'s `note_on_keys`, duplicated here since that one is
/// private to the crate's own unit tests.
fn note_on_keys(midi_bytes: &[u8]) -> Vec<u8> {
    let smf = Smf::parse(midi_bytes).expect("valid MIDI");
    smf.tracks
        .iter()
        .flat_map(|t| t.iter())
        .filter_map(|e| match e.kind {
            TrackEventKind::Midi {
                message: MidiMessage::NoteOn { key, vel, .. },
                ..
            } if vel.as_int() > 0 => Some(key.as_int()),
            _ => None,
        })
        .collect()
}

#[given(expr = "the score source:")]
fn given_score_source(world: &mut AudioKeyWorld, step: &Step) {
    world.source = step.docstring().cloned().unwrap_or_default();
}

#[when(expr = "the score's audio is generated")]
fn when_audio_generated(world: &mut AudioKeyWorld) {
    let score = compile(&world.source, "test.poentaro", &[])
        .unwrap_or_else(|err| panic!("compile returned an irrecoverable error: {}", err.message()));
    // Same order the web app's `generate_wav`/`generate_mp3` wasm bindings
    // use (see `write_wav_from_source_filtered`): expand `sequence`
    // navigation, then synthesize.
    let score = expand_navigation(&score).unwrap_or_else(|err| {
        panic!(
            "expand_navigation returned an irrecoverable error: {}",
            err.message()
        )
    });
    let midi = write_midi(&score).unwrap_or_else(|err| {
        panic!(
            "write_midi returned an irrecoverable error: {}",
            err.message()
        )
    });
    world.note_on_pitches = note_on_keys(&midi);
}

#[then(expr = "the first sounded note is MIDI pitch {int}")]
fn then_first_sounded_note_pitch(world: &mut AudioKeyWorld, expected: u8) {
    let actual = *world
        .note_on_pitches
        .first()
        .unwrap_or_else(|| panic!("no notes sounded at all"));
    assert_eq!(
        actual, expected,
        "first sounded note's MIDI pitch; full note-on sequence was {:?}",
        world.note_on_pitches
    );
}

#[tokio::main]
async fn main() {
    AudioKeyWorld::run("tests/features/sequence_intro_key_context.feature").await;
}
