//! Confirms `duration_to_ticks`'s tuplet-multiplier division (Step 6 of `TUPLET_PLAN.md`):
//! a tuplet-rescaled measure's note durations must convert to the same total tick span as
//! the musically equivalent non-tuplet measure, since `GroupedNote::duration` etc. is
//! already scaled up by `PartSlice::resolution_multiplier` before it ever reaches MIDI
//! export (see `crate::tuplet`).

use super::midi_notes::duration_to_ticks;
use super::one_measure_score;
use crate::ast::grouped::GroupedNote;
use crate::ast::grouped::{NoteEvent, Notes, PartRow, PartSlice};
use crate::ast::parsed::{Accidental, NumberedPitch, PartKind, Soundfont};
use crate::error::Span;
use crate::midi::measure_start_times_seconds;

fn triplet_eighth_note_event() -> NoteEvent {
    // One of three eighth notes in a `3:{1_1_1_}` triplet: written eighth-note duration
    // (2) rescaled by `resolution_multiplier = lcm(3) = 3`, then compressed by the
    // triplet's `den/num = 2/3` ratio: `2 * 3 * 2 / 3 = 4`. Three of these together
    // occupy exactly one real quarter-beat, matching `plain_quarter_note_event`'s
    // single (unscaled) duration of `4`.
    NoteEvent::Note(GroupedNote {
        pitch: NumberedPitch::One,
        accidental: Accidental::Natural,
        octave: 0,
        duration: 4,
        slur: false,
        tie_to_next_span: None,
        event_span: Span::new(0, 0),
        group_membership: 0,
        group_continuation: 0,
        dotted: false,
        double_dotted: false,
        slur_group_close_at_duration: None,
        tuplet: None,
    })
}

fn plain_quarter_note_event() -> NoteEvent {
    NoteEvent::Note(GroupedNote {
        pitch: NumberedPitch::One,
        accidental: Accidental::Natural,
        octave: 0,
        duration: 4,
        slur: false,
        tie_to_next_span: None,
        event_span: Span::new(0, 0),
        group_membership: 0,
        group_continuation: 0,
        dotted: false,
        double_dotted: false,
        slur_group_close_at_duration: None,
        tuplet: None,
    })
}

#[test]
fn duration_to_ticks_divides_out_the_tuplet_multiplier() {
    // Three rescaled triplet-eighth notes (duration 4 each, multiplier 3) must sum to
    // the same tick span as one plain quarter note (duration 4, multiplier 1) — both
    // represent one real quarter-beat.
    let triplet_total: u32 = (0..3).map(|_| duration_to_ticks(4, 3)).sum();
    let plain_total = duration_to_ticks(4, 1);
    assert_eq!(
        triplet_total, plain_total,
        "3 rescaled triplet-eighth-note ticks must sum to the same total as 1 plain quarter note"
    );
    assert_eq!(
        plain_total, 480,
        "1 plain quarter note is 480 ticks at TPQ=480"
    );
}

#[test]
fn triplet_measure_produces_same_total_ticks_as_equivalent_non_tuplet_measure() {
    // Full end-to-end check through `write_midi`'s measure-timing pipeline
    // (`measure_start_times_seconds`, which walks `process_measure` internally): a
    // measure containing a `3:{1_1_1_}`-style rescaled triplet (resolution_multiplier
    // 3, three duration-4 notes) must take exactly as long to play as a measure
    // containing the one plain quarter note it's musically equivalent to.
    let mut triplet_score = one_measure_score();
    triplet_score.measures[0].parts[0] = PartRow::Timed(PartSlice {
        name: None,
        resolution_multiplier: 3,
        beat_group_size: 4,
        kind: PartKind::Notes,
        soundfont: Soundfont::CHOIR_AAHS,
        volume: 100,
        octave_offset: 0,
        range_instrument: None,
        singing_voice: None,
        notes: Notes {
            events: vec![
                triplet_eighth_note_event(),
                triplet_eighth_note_event(),
                triplet_eighth_note_event(),
            ],
        },
        lyrics: None,
        has_error: false,
    });

    let mut plain_score = one_measure_score();
    plain_score.measures[0].parts[0] = PartRow::Timed(PartSlice {
        name: None,
        resolution_multiplier: 1,
        beat_group_size: 4,
        kind: PartKind::Notes,
        soundfont: Soundfont::CHOIR_AAHS,
        volume: 100,
        octave_offset: 0,
        range_instrument: None,
        singing_voice: None,
        notes: Notes {
            events: vec![plain_quarter_note_event()],
        },
        lyrics: None,
        has_error: false,
    });

    let triplet_end = measure_start_times_seconds(&triplet_score).unwrap()[1];
    let plain_end = measure_start_times_seconds(&plain_score).unwrap()[1];
    assert!(
        (triplet_end - plain_end).abs() < 1e-9,
        "triplet measure ({triplet_end}s) must take the same real time as its \
         non-tuplet equivalent ({plain_end}s)"
    );
}
