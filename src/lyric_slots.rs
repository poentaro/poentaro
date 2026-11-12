//! Which notes of a measure take a lyric syllable, and where they sit.
//!
//! A lyric part has no notes of its own: it sings along to its target's. This
//! is the one place that decides which of the target's events consume a
//! syllable (every `Note` that isn't the continuation of a tie), so the
//! grouper's tie-aware pairing, the compiler's column placement and the lyric
//! source spans can never disagree.

use crate::ast::grouped::NoteEvent;

/// One lyric slot: a note that takes a syllable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LyricSlot {
    /// Index of the note within the measure's events; also its running note
    /// id offset (a note id is the part's event count so far).
    pub(crate) event_index: usize,
    /// Measure-relative column the note starts at.
    pub(crate) column: u32,
}

/// A point in a measure, as a fraction of the measure's length, so parts
/// written at different resolutions can be compared.
#[derive(Debug, Clone, Copy)]
pub(crate) struct MeasureTime {
    pub(crate) column: u32,
    /// The measure's length in the same units as `column`.
    pub(crate) length: u32,
}

impl MeasureTime {
    pub(crate) fn same_as(self, other: MeasureTime) -> bool {
        u64::from(self.column) * u64::from(other.length)
            == u64::from(other.column) * u64::from(self.length)
    }
}

/// The lyric slots of one measure's `events`, and the state it hands to the
/// next measure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MeasureLyricSlots {
    pub(crate) slots: Vec<LyricSlot>,
    /// The measure's last note is tied into the next measure.
    pub(crate) exits_tied: bool,
    /// Measure-relative column just past the last event (where the bar line sits).
    pub(crate) end_column: u32,
}

/// `entering_tied`: the previous measure's last note was tied into this one.
pub(crate) fn measure_lyric_slots(events: &[NoteEvent], entering_tied: bool) -> MeasureLyricSlots {
    let mut tied = entering_tied;
    let mut column = 0;
    let slots = events
        .iter()
        .enumerate()
        .filter_map(|(event_index, event)| {
            let slot = match event {
                NoteEvent::Note(note) => {
                    let slot = (!tied).then_some(LyricSlot {
                        event_index,
                        column,
                    });
                    tied = note.tie_to_next();
                    slot
                }
                NoteEvent::Rest(_) | NoteEvent::Chord(_) | NoteEvent::Percussion(_) => {
                    tied = false;
                    None
                }
            };
            column += event.duration();
            slot
        })
        .collect();
    MeasureLyricSlots {
        slots,
        exits_tied: tied,
        end_column: column,
    }
}

/// Where a lyric part stands along its target's notes as a measure starts.
///
/// A lyric part has to walk its target's notes through *every* measure, even
/// one where its own row is hidden for having no syllables; otherwise a tie
/// from before such a measure leaks past it, and the note ids drift from the
/// ones its target gives the same notes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LyricCursor {
    /// The note id of the measure's first target event.
    pub(crate) next_note_id: usize,
    /// The previous measure's last target note is tied into this one.
    pub(crate) entering_tied: bool,
}

impl LyricCursor {
    /// The cursor at the start of the next measure. Ties follow the target's
    /// notes in every measure, as the grouper pairs them. Note ids advance only
    /// when `target_visible`, since a hidden target allocates none.
    pub(crate) fn advance(self, target_events: &[NoteEvent], target_visible: bool) -> Self {
        LyricCursor {
            next_note_id: if target_visible {
                self.next_note_id + target_events.len()
            } else {
                self.next_note_id
            },
            entering_tied: measure_lyric_slots(target_events, self.entering_tied).exits_tied,
        }
    }
}

/// One chord attack of a measure: a `Chord` event. A chord rest (`0`, a
/// `Rest` event) and a `-` extension (folded into the previous chord's
/// duration) are not attacks. Under a chord part the Nth bracketed lyric item
/// sits at the Nth attack, so the grouper's tally, the compiler's placement
/// and the lyric source spans all read attacks from here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ChordAttack {
    /// Index of the chord within the measure's events (and its note id offset).
    pub(crate) event_index: usize,
    /// Measure-relative column the chord starts at.
    pub(crate) column: u32,
}

pub(crate) fn chord_attacks(events: &[NoteEvent]) -> Vec<ChordAttack> {
    events
        .iter()
        .enumerate()
        .scan(0, |column, (event_index, event)| {
            let start = *column;
            *column += event.duration();
            Some((event_index, event, start))
        })
        .filter_map(|(event_index, event, column)| {
            matches!(event, NoteEvent::Chord(_)).then_some(ChordAttack {
                event_index,
                column,
            })
        })
        .collect()
}
