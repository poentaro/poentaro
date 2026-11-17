use super::midi_notes::{duration_to_ticks, resolve_midi_note, resolve_midi_note_with_accidental};
use super::*;
use crate::ast::grouped::Metadata;
use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName, Offset};
use crate::error::Span;
use midly::{MidiMessage, Smf, TrackEventKind};

#[path = "tests_chord_voicing.rs"]
mod tests_chord_voicing;
#[path = "tests_chords.rs"]
mod tests_chords;
#[path = "tests_tied_notes.rs"]
mod tests_tied_notes;
#[path = "tests_timing.rs"]
mod tests_timing;
#[path = "tests_tuplets.rs"]
mod tests_tuplets;

fn text_style(
    font_size: u32,
    horizontal_padding_pt: u32,
    vertical_padding_pt: u32,
) -> crate::ast::grouped::TextStyle {
    crate::ast::grouped::TextStyle {
        font_size,
        horizontal_padding_pt,
        vertical_padding_pt,
        ..Default::default()
    }
}

fn default_test_metadata() -> Metadata {
    Metadata {
        title: None,
        subtitle: None,
        author: None,
        row_height: 24,
        max_measures_per_system: 28,
        note_number_width: 8,
        parts_list_columns: 3,
        part_label_width_pt: 40,
        page_margin_pt: 25,
        lyrics: text_style(14, 4, 12),
        notes: text_style(14, 4, 0),
        chords: text_style(14, 4, 0),
        note_dash: text_style(14, 4, 0),
        title_style: text_style(36, 0, 0),
        subtitle_style: text_style(19, 0, 0),
        author_style: text_style(14, 0, 0),
        sequence: text_style(12, 0, 0),
        part_legend: text_style(12, 0, 0),
        measure_number: text_style(10, 0, 0),
        section_label: text_style(12, 0, 0),
        part_label: text_style(12, 0, 0),
        page_number: text_style(14, 0, 0),
        running_title: text_style(14, 0, 0),
        merge_duplicate_measures_across_parts: true,
        hide_resting_parts: true,
        hide_system_dividers: false,
        directive_row_offset: Offset::default(),
    }
}

fn count_note_on_events(midi_bytes: &[u8]) -> usize {
    note_on_keys(midi_bytes).len()
}

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

fn key(name: NoteName, octave: u8) -> KeyChange {
    KeyChange {
        note: Note {
            name,
            octave,
            accidental: Accidental::Natural,
        },
    }
}

#[test]
fn middle_c_degree_one() {
    assert_eq!(
        resolve_midi_note(&NumberedPitch::One, 0, &key(NoteName::C, 4)),
        60
    );
}

#[test]
fn degree_five_c4_is_g4() {
    assert_eq!(
        resolve_midi_note(&NumberedPitch::Five, 0, &key(NoteName::C, 4)),
        67
    );
}

#[test]
fn octave_up_shifts_by_12() {
    assert_eq!(
        resolve_midi_note(&NumberedPitch::One, 1, &key(NoteName::C, 4)),
        72
    );
}

#[test]
fn key_g4_degree_one_is_midi_67() {
    assert_eq!(
        resolve_midi_note(&NumberedPitch::One, 0, &key(NoteName::G, 4)),
        67
    );
}

#[test]
fn duration_quarter_note_is_480_ticks() {
    assert_eq!(duration_to_ticks(4, 1), 480);
}

#[test]
fn duration_eighth_note_is_240_ticks() {
    assert_eq!(duration_to_ticks(2, 1), 240);
}

#[test]
fn duration_half_note_is_960_ticks() {
    assert_eq!(duration_to_ticks(8, 1), 960);
}

pub(super) fn one_measure_score() -> Score {
    use crate::ast::grouped::GroupedNote;
    use crate::ast::grouped::{
        MultiPartMeasure, NoteEvent, Notes, PartRow, PartSlice, Score, TimeSignature,
    };
    use crate::ast::parsed::{NumberedPitch, PartKind, Soundfont};
    Score {
        metadata: default_test_metadata(),
        measures: vec![MultiPartMeasure {
            time_signature: Some(TimeSignature {
                numerator: 4,
                denominator: 4,
            }),
            bpm: Some(120),
            key: Some(KeyChange {
                note: Note {
                    name: NoteName::C,
                    octave: 4,
                    accidental: Accidental::Natural,
                },
            }),
            label: None,
            merge_duplicate_measures_across_parts: true,
            hide_resting_parts: true,
            system_break: false,
            parts: vec![PartRow::Timed(PartSlice {
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
                    events: vec![NoteEvent::Note(GroupedNote {
                        pitch: NumberedPitch::One,
                        accidental: Accidental::Natural,
                        octave: 0,
                        duration: 16,
                        slur: false,
                        tie_to_next_span: None,
                        event_span: Span::new(0, 0),
                        group_membership: 0,
                        group_continuation: 0,
                        dotted: false,
                        double_dotted: false,
                        slur_group_close_at_duration: None,
                        tuplet: None,
                    })],
                },
                lyrics: None,
                has_error: false,
            })],
            source_span: Span::new(0, 0),
            diagnostics: vec![],
        }],
        document_diagnostics: vec![],
        sequence: None,
    }
}

#[test]
fn sharp_note_midi_pitch_is_one_semitone_higher_than_natural() {
    use crate::ast::parsed::{Note, NoteName, NumberedPitch};

    let key = KeyChange {
        note: Note {
            name: NoteName::C,
            octave: 4,
            accidental: Accidental::Natural,
        },
    };
    let natural = resolve_midi_note(&NumberedPitch::Seven, 0, &key);
    let sharp =
        resolve_midi_note_with_accidental(&NumberedPitch::Seven, &Accidental::Sharp, 0, &key);
    assert_eq!(
        sharp,
        natural + 1,
        "7# must be exactly one semitone above 7"
    );
}

fn one_note_score_with_octave_offset(octave_offset: i8) -> Score {
    use crate::ast::grouped::GroupedNote;
    use crate::ast::grouped::{
        MultiPartMeasure, NoteEvent, Notes, PartRow, PartSlice, Score, TimeSignature,
    };
    use crate::ast::parsed::{NumberedPitch, PartKind, Soundfont};

    Score {
        metadata: default_test_metadata(),
        measures: vec![MultiPartMeasure {
            time_signature: Some(TimeSignature {
                numerator: 4,
                denominator: 4,
            }),
            bpm: Some(120),
            key: Some(KeyChange {
                note: Note {
                    name: NoteName::C,
                    octave: 4,
                    accidental: Accidental::Natural,
                },
            }),
            label: None,
            merge_duplicate_measures_across_parts: true,
            hide_resting_parts: true,
            system_break: false,
            parts: vec![PartRow::Timed(PartSlice {
                name: None,
                resolution_multiplier: 1,
                beat_group_size: 4,
                kind: PartKind::Notes,
                soundfont: Soundfont::CHOIR_AAHS,
                volume: 100,
                octave_offset,
                range_instrument: None,
                singing_voice: None,
                notes: Notes {
                    events: vec![NoteEvent::Note(GroupedNote {
                        pitch: NumberedPitch::One,
                        accidental: Accidental::Natural,
                        octave: 0,
                        duration: 16,
                        slur: false,
                        tie_to_next_span: None,
                        event_span: Span::new(0, 0),
                        group_membership: 0,
                        group_continuation: 0,
                        dotted: false,
                        double_dotted: false,
                        slur_group_close_at_duration: None,
                        tuplet: None,
                    })],
                },
                lyrics: None,
                has_error: false,
            })],
            source_span: Span::new(0, 0),
            diagnostics: vec![],
        }],
        document_diagnostics: vec![],
        sequence: None,
    }
}
