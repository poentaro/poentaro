use std::collections::HashMap;

use crate::ast::grouped::Score;
use crate::ast::parsed::{KeyChange, PartKind};
use crate::error::IrrecoverableError;
use crate::parser::parts_parser::DEFAULT_PART_VOLUME;

pub use crate::ast::parsed::NumberedPitch;

mod channel_volume;
mod chord_voicing;
mod event_processing;
mod metronome;
mod midi_notes;
mod navigation;
mod timing;
mod timing_note_events;
mod timing_note_timings;
mod timing_range;
use channel_volume::channel_volume_control_value;
use event_processing::{
    flush_pending_ties, flush_pending_ties_at_tick, process_chord_events, process_measure_notes,
    process_percussion_events,
};
pub use navigation::{
    earliest_playback_position, expand_for_measure_range, expand_navigation,
    expand_navigation_with_note_positions, expand_navigation_with_origins, ExpandedMeasureOrigin,
};
pub(crate) const TPQ: u16 = 480; // ticks per quarter note
const VELOCITY: u8 = 80;
const CHORD_CHANNEL: u8 = 3;
const PERCUSSION_CHANNEL: u8 = 9;
const GM_STANDARD_KIT_PROGRAM: u8 = 0;

fn part_index_to_midi_channel(index: usize) -> u8 {
    // Skip CHORD_CHANNEL (3) and GM drum channel (9).
    let raw = index as u8;
    let after_chord = if raw >= CHORD_CHANNEL { raw + 1 } else { raw };
    if after_chord >= 9 {
        after_chord + 1
    } else {
        after_chord
    }
}

/// A melodic part's MIDI channel and instrument, keyed by the part's own
/// (stable, unique) name rather than its positional index within any single
/// measure's `parts` vec — a `sequence` `(-abbrev ...)` omission removes
/// that part from some measures' `parts` vecs (see `navigation::build_expanded`),
/// which shifts positional indices of the parts that remain, but never
/// changes their relative order or a part's own name.
pub(crate) struct PartChannelAssignment {
    name: Option<String>,
    channel: u8,
    soundfont: u8,
    volume: u8,
}

/// Assigns each distinct melodic part appearing anywhere in `measures` a
/// stable channel, in order of first appearance across all measures (not
/// just the first one) — so a part that's absent from the first measure
/// (e.g. because that occurrence omitted it) still gets a channel and
/// instrument from wherever it's first seen.
fn build_part_channel_assignments(
    measures: &[crate::ast::grouped::MultiPartMeasure],
) -> Vec<PartChannelAssignment> {
    let mut assignments: Vec<PartChannelAssignment> = Vec::new();
    for measure in measures {
        for slice in measure
            .parts
            .iter()
            .map(|r| r.slice())
            .filter(|p| is_melodic(p.kind))
        {
            if assignments.iter().any(|a| a.name == slice.name) {
                continue;
            }
            let channel = part_index_to_midi_channel(assignments.len());
            assignments.push(PartChannelAssignment {
                name: slice.name.clone(),
                channel,
                soundfont: slice.soundfont.0,
                volume: slice.volume,
            });
        }
    }
    assignments
}

// `assignments` is always built from the same measures being rendered here
// (see call sites), so every part looked up is present; channel 0 is an
// arbitrary but harmless fallback that's never actually reached.
fn channel_for_part(assignments: &[PartChannelAssignment], name: &Option<String>) -> u8 {
    assignments
        .iter()
        .find(|a| &a.name == name)
        .map_or(0, |a| a.channel)
}

fn is_melodic(kind: PartKind) -> bool {
    matches!(kind, PartKind::Notes)
}

fn parts_matching(
    measure: &crate::ast::grouped::MultiPartMeasure,
    matches: impl Fn(PartKind) -> bool,
) -> Vec<&crate::ast::grouped::PartSlice> {
    measure
        .parts
        .iter()
        .map(|r| r.slice())
        .filter(|p| matches(p.kind))
        .collect()
}

pub(crate) struct RawEvent {
    pub(crate) tick: u32,
    pub(crate) kind: RawKind,
}

pub(crate) enum RawKind {
    Tempo(u32),
    NoteOn {
        channel: u8,
        note: u8,
    },
    NoteOff {
        channel: u8,
        note: u8,
    },
    ProgramChange {
        channel: u8,
        program: u8,
    },
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
}

/// Pending-tie tracking for all three channel groups (melodic, chord, percussion), plus
/// each chord part's previous voicing (see [`ChordPartState`]), bundled so
/// `process_measure` can thread them through as a single argument.
#[derive(Default)]
pub(crate) struct TieState {
    /// Keyed by MIDI channel (a part's channel is a stable identity for the
    /// lifetime of a render, see [`PartChannelAssignment`]) rather than by
    /// position in any single measure's part list.
    per_part_ties: HashMap<u8, HashMap<u8, u32>>,
    chord_parts: Vec<ChordPartState>,
    percussion_ties: HashMap<u8, u32>,
}

/// One chord part's cross-measure playback state.
#[derive(Default)]
pub(crate) struct ChordPartState {
    pub(crate) ties: HashMap<u8, u32>,
    /// The last voicing this part sounded, which the next chord leads from (see
    /// [`chord_voicing`]).
    pub(crate) previous_voicing: Option<chord_voicing::ChordVoicing>,
}

impl TieState {
    fn flush(mut self, raw: &mut Vec<RawEvent>, current_tick: u32) {
        flush_pending_ties(raw, self.per_part_ties.into_iter().collect());
        for chord_part in &mut self.chord_parts {
            flush_pending_ties_at_tick(&mut chord_part.ties, current_tick, raw, CHORD_CHANNEL);
        }
        flush_pending_ties_at_tick(
            &mut self.percussion_ties,
            current_tick,
            raw,
            PERCUSSION_CHANNEL,
        );
    }
}

fn write_program_change_preamble(
    score: &Score,
    part_channels: &[PartChannelAssignment],
    metronome: bool,
    raw: &mut Vec<RawEvent>,
) {
    for assignment in part_channels {
        raw.push(RawEvent {
            tick: 0,
            kind: RawKind::ProgramChange {
                channel: assignment.channel,
                program: assignment.soundfont,
            },
        });
        raw.push(RawEvent {
            tick: 0,
            kind: RawKind::ControlChange {
                channel: assignment.channel,
                controller: 7,
                value: channel_volume_control_value(assignment.volume),
            },
        });
    }

    let chord_part = score.measures.iter().find_map(|measure| {
        measure
            .parts
            .iter()
            .find(|r| r.slice().kind == PartKind::Chords)
    });
    let chord_program = chord_part.map(|r| r.slice().soundfont.0).unwrap_or(0);
    let chord_volume = chord_part
        .map(|r| r.slice().volume)
        .unwrap_or(DEFAULT_PART_VOLUME);
    raw.push(RawEvent {
        tick: 0,
        kind: RawKind::ProgramChange {
            channel: CHORD_CHANNEL,
            program: chord_program,
        },
    });
    raw.push(RawEvent {
        tick: 0,
        kind: RawKind::ControlChange {
            channel: CHORD_CHANNEL,
            controller: 7,
            value: channel_volume_control_value(chord_volume),
        },
    });

    let has_percussion = score.measures.iter().any(|measure| {
        measure
            .parts
            .iter()
            .any(|r| r.slice().kind == PartKind::Percussion)
    });
    if has_percussion || metronome {
        // Percussion parts and the metronome share channel 9; per-part channel volume (CC7)
        // isn't meaningful on a shared channel without a per-note-velocity refactor, so it's
        // skipped here. Exactly one Standard Kit program change covers all of them.
        raw.push(RawEvent {
            tick: 0,
            kind: RawKind::ProgramChange {
                channel: PERCUSSION_CHANNEL,
                program: GM_STANDARD_KIT_PROGRAM,
            },
        });
    }
}

pub fn write_midi(score: &Score) -> Result<Vec<u8>, IrrecoverableError> {
    write_midi_with_metronome(score, false)
}

/// Like [`write_midi`], but when `metronome` is `true` also writes a click on every beat
/// (accented on the first beat of each measure) to the percussion channel.
pub fn write_midi_with_metronome(
    score: &Score,
    metronome: bool,
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut raw: Vec<RawEvent> = Vec::new();
    let part_channels = build_part_channel_assignments(&score.measures);
    write_program_change_preamble(score, &part_channels, metronome, &mut raw);

    let mut current_tick: u32 = 0;
    let mut tie_state = TieState::default();
    let mut active_key = KeyChange::default();
    let mut measure_start_ticks = vec![current_tick];

    for measure in &score.measures {
        current_tick = process_measure(
            measure,
            current_tick,
            &mut raw,
            &mut tie_state,
            &mut active_key,
            &part_channels,
        )?;
        measure_start_ticks.push(current_tick);
    }

    if metronome {
        raw.extend(metronome::metronome_events(
            &score.measures,
            &measure_start_ticks,
        ));
    }

    tie_state.flush(&mut raw, current_tick);
    sort_raw_events(&mut raw);

    let track = build_track_events(&raw);
    write_smf(track)
}

pub use timing::{
    build_measure_range_score, measure_start_times_seconds, note_timings_seconds,
    note_timings_seconds_for_literal_range, note_timings_seconds_for_range, NoteTiming,
};
pub use timing_range::{write_midi_for_measure_range, write_midi_for_measure_range_with_metronome};

pub(crate) fn process_measure(
    measure: &crate::ast::grouped::MultiPartMeasure,
    current_tick: u32,
    raw: &mut Vec<RawEvent>,
    tie_state: &mut TieState,
    active_key: &mut KeyChange,
    part_channels: &[PartChannelAssignment],
) -> Result<u32, IrrecoverableError> {
    let TieState {
        per_part_ties,
        chord_parts: chord_part_states,
        percussion_ties,
    } = tie_state;
    if let Some(bpm) = measure.bpm {
        let micros = 60_000_000 / bpm;
        raw.push(RawEvent {
            tick: current_tick,
            kind: RawKind::Tempo(micros),
        });
    }

    if let Some(key) = &measure.key {
        *active_key = key.clone();
    }

    let mut measure_duration: u32 = 0;

    // Ditto parts still sound — only rendering skips them.
    let notes_parts = parts_matching(measure, is_melodic);

    for part in &notes_parts {
        let channel = channel_for_part(part_channels, &part.name);
        let ties = per_part_ties.entry(channel).or_default();
        let part_duration =
            process_measure_notes(part, current_tick, raw, ties, active_key, channel)?;
        if part_duration > measure_duration {
            measure_duration = part_duration;
        }
    }

    let chord_parts = parts_matching(measure, |kind| kind == PartKind::Chords);

    while chord_part_states.len() < chord_parts.len() {
        chord_part_states.push(ChordPartState::default());
    }

    for (part, state) in chord_parts.iter().zip(chord_part_states.iter_mut()) {
        let chord_duration = process_chord_events(
            &part.notes.events,
            current_tick,
            raw,
            active_key,
            state,
            part.resolution_multiplier,
        );
        if chord_duration > measure_duration {
            measure_duration = chord_duration;
        }
    }

    let percussion_parts = parts_matching(measure, |kind| kind == PartKind::Percussion);

    for part in &percussion_parts {
        // All percussion parts share channel 9, so their tied notes are tracked in one
        // shared map keyed by GM key number, which is unique per percussion part by
        // construction.
        let percussion_duration =
            process_percussion_events(part, current_tick, raw, percussion_ties);
        if percussion_duration > measure_duration {
            measure_duration = percussion_duration;
        }
    }

    Ok(current_tick + measure_duration)
}

mod smf_writer;
use smf_writer::{build_track_events, sort_raw_events, write_smf};

#[cfg(test)]
mod channel_volume_tests;
#[cfg(test)]
mod percussion_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_metronome;
#[cfg(test)]
mod tests_part_channel_assignment;
