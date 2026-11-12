use super::*;
use crate::ast::grouped::{
    GroupedNote, MultiPartMeasure, NoteEvent, Notes, PartRow, PartSlice, Score, TimeSignature,
};
use crate::ast::parsed::{
    Accidental, KeyChange, Note, NoteName, NumberedPitch, PartKind, Soundfont,
};
use crate::error::Span;

fn tied_note_event(tied: bool) -> NoteEvent {
    NoteEvent::Note(GroupedNote {
        pitch: NumberedPitch::One,
        accidental: Accidental::Natural,
        octave: 0,
        duration: 4, // quarter note
        slur: false,
        tie_to_next_span: if tied { Some(Span::new(0, 1)) } else { None },
        event_span: Span::new(0, 0),
        group_membership: 0,
        group_continuation: 0,
        dotted: false,
        double_dotted: false,
        slur_group_close_at_duration: None,
        tuplet: None,
    })
}

fn tied_note_part(tied: bool) -> PartRow {
    PartRow::Timed(PartSlice {
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
            events: vec![tied_note_event(tied)],
        },
        lyrics: None,
        has_error: false,
    })
}

fn tied_note_measure(
    time_signature: Option<TimeSignature>,
    bpm: Option<u32>,
    key: Option<KeyChange>,
    tied: bool,
) -> MultiPartMeasure {
    MultiPartMeasure {
        time_signature,
        bpm,
        key,
        label: None,
        merge_duplicate_measures_across_parts: true,
        hide_resting_parts: true,
        system_break: false,
        parts: vec![tied_note_part(tied)],
        source_span: Span::new(0, 0),
        diagnostics: vec![],
    }
}

#[test]
fn tied_notes_produce_single_note_on() {
    // `1~1` — two quarter notes tied together should produce exactly one NoteOn.
    let score = Score {
        metadata: default_test_metadata(),
        measures: vec![
            tied_note_measure(
                Some(TimeSignature {
                    numerator: 4,
                    denominator: 4,
                }),
                Some(120),
                Some(KeyChange {
                    note: Note {
                        name: NoteName::C,
                        octave: 4,
                        accidental: Accidental::Natural,
                    },
                }),
                true,
            ),
            tied_note_measure(None, None, None, false),
        ],
        document_diagnostics: vec![],
        sequence: None,
    };

    let midi_bytes = write_midi(&score).unwrap();
    assert_eq!(
        count_note_on_events(&midi_bytes),
        1,
        "tied 1~1 must produce exactly one NoteOn"
    );
}
