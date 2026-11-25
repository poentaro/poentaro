//! One part's measure as an MEI `<layer>`: notes, rests, ties, beams and the
//! slurs its `( )` groups open and close.

use super::duration::{split_duration, split_undotted_duration, SimpleDuration};
use super::lyrics::{verses_xml, MeasureSyllables};
use super::pitch::{spell, AccidentalTracker};
use super::staves::StaffPart;
use crate::ast::grouped::{GroupedNote, LyricsSlice, NoteEvent, PartSlice};
use crate::ast::parsed::KeyChange;
use itertools::Itertools;

/// Sixteenths in a quarter note: shorter values are beamed.
const QUARTER: u32 = 4;

/// Per-staff state carried from measure to measure.
#[derive(Default)]
pub(super) struct StaffState {
    tied_in: bool,
    /// Notes that opened a still-unclosed slur, innermost last.
    open_slurs: Vec<OpenSlur>,
    /// `group_continuation` of the previous note, i.e. slurs still open.
    previous_continuation: u8,
}

struct OpenSlur {
    start_id: String,
    start_measure: usize,
}

/// A slur whose both ends are known. Verovio wants it encoded in the
/// measure of its start note.
pub(super) struct Slur {
    /// 1-based staff number.
    pub(super) staff: usize,
    pub(super) start_measure: usize,
    pub(super) start_id: String,
    pub(super) end_id: String,
}

/// Everything about where a layer sits that doesn't change while writing it.
pub(super) struct LayerContext<'a> {
    pub(super) staff: &'a StaffPart,
    /// 1-based staff number.
    pub(super) staff_number: usize,
    pub(super) measure_index: usize,
    pub(super) key: &'a KeyChange,
    pub(super) verses: &'a [MeasureSyllables],
}

/// One written note or rest, before beaming.
struct Piece {
    xml: String,
    /// Sixteenths from the start of the measure.
    start: u32,
    sixteenths: u32,
    is_note: bool,
}

/// The layer's inner XML; finished slurs are pushed onto `slurs`.
pub(super) fn layer_xml(
    slice: &PartSlice,
    context: &LayerContext,
    state: &mut StaffState,
    slurs: &mut Vec<Slur>,
) -> String {
    let events = &slice.notes.events;
    if events
        .iter()
        .all(|event| matches!(event, NoteEvent::Rest(rest) if !rest.fermata))
    {
        state.tied_in = false;
        return "<mRest/>".to_string();
    }

    let multiplier = slice.resolution_multiplier.max(1);
    let mut accidentals = AccidentalTracker::new(context.key);
    let mut start = 0;
    let mut pieces = Vec::new();
    for (event_index, event) in events.iter().enumerate() {
        let durations = split_duration(event.duration() / multiplier);
        match event {
            NoteEvent::Note(note) => {
                let written = WrittenNote {
                    note,
                    event_index,
                    start,
                    durations: &durations,
                };
                pieces.extend(note_pieces(&written, context, state, &mut accidentals));
                track_slurs(note, &note_id(note), context, state, slurs);
                state.tied_in = note.tie_to_next();
            }
            NoteEvent::Rest(_) | NoteEvent::Chord(_) | NoteEvent::Percussion(_) => {
                state.tied_in = false;
                let fermata = matches!(event, NoteEvent::Rest(rest) if rest.fermata);
                pieces.extend(rest_pieces(start, &durations, fermata));
            }
        }
        start += durations
            .iter()
            .map(|duration| duration.sixteenths)
            .sum::<u32>();
    }
    beam_pieces(&pieces, slice.beat_group_size.max(1))
}

fn note_id(note: &GroupedNote) -> String {
    format!("n{}-{}", note.event_span.start, note.event_span.end)
}

struct WrittenNote<'a> {
    note: &'a GroupedNote,
    event_index: usize,
    start: u32,
    durations: &'a [SimpleDuration],
}

/// A note as one or more tied pieces; only the first carries the
/// accidental, the lyrics and the source-span id.
fn note_pieces(
    written: &WrittenNote,
    context: &LayerContext,
    state: &StaffState,
    accidentals: &mut AccidentalTracker,
) -> Vec<Piece> {
    let note = written.note;
    let octave = i32::from(note.octave) + i32::from(context.staff.octave_offset);
    let pitch = spell(&note.pitch, &note.accidental, octave, context.key);
    let accidental = accidentals.attribute(pitch);
    let id = note_id(note);
    let syllables = verses_xml(context.verses, written.event_index);
    let last = written.durations.len().saturating_sub(1);
    written
        .durations
        .iter()
        .enumerate()
        .scan(written.start, |start, (index, duration)| {
            let piece_start = *start;
            *start += duration.sixteenths;
            let first = index == 0;
            let tie = tie_attribute(state.tied_in || !first, note.tie_to_next() || index < last);
            let fermata = fermata_attribute(first && note.fermata);
            let open = format!(
                "<note xml:id=\"{}\" pname=\"{}\" oct=\"{}\" dur=\"{}\"{}{}{tie}{fermata}",
                if first {
                    id.clone()
                } else {
                    format!("{id}-{index}")
                },
                pitch.letter.pname(),
                pitch.octave,
                duration.dur,
                duration.dots_attribute(),
                if first { accidental.as_str() } else { "" },
            );
            let xml = if first && !syllables.is_empty() {
                format!("{open}>{syllables}</note>")
            } else {
                format!("{open}/>")
            };
            Some(Piece {
                xml,
                start: piece_start,
                sixteenths: duration.sixteenths,
                is_note: true,
            })
        })
        .collect()
}

fn tie_attribute(tied_in: bool, tied_out: bool) -> &'static str {
    match (tied_in, tied_out) {
        (true, true) => " tie=\"m\"",
        (true, false) => " tie=\"t\"",
        (false, true) => " tie=\"i\"",
        (false, false) => "",
    }
}

fn fermata_attribute(marked: bool) -> &'static str {
    if marked {
        " fermata=\"above\""
    } else {
        ""
    }
}

fn rest_pieces(start: u32, durations: &[SimpleDuration], fermata: bool) -> Vec<Piece> {
    durations
        .iter()
        .enumerate()
        .scan(start, |start, (index, duration)| {
            let piece_start = *start;
            *start += duration.sixteenths;
            Some(Piece {
                xml: format!(
                    "<rest dur=\"{}\"{}{}/>",
                    duration.dur,
                    duration.dots_attribute(),
                    fermata_attribute(index == 0 && fermata)
                ),
                start: piece_start,
                sixteenths: duration.sixteenths,
                is_note: false,
            })
        })
        .collect()
}

/// Wraps runs of eighths-and-shorter that fall in the same beat in `<beam>`.
fn beam_pieces(pieces: &[Piece], beat: u32) -> String {
    pieces
        .iter()
        .chunk_by(|piece| (piece.sixteenths < QUARTER).then_some(piece.start / beat))
        .into_iter()
        .map(|(beat_index, run)| {
            let run = run.collect_vec();
            let inner = run.iter().map(|piece| piece.xml.as_str()).join("");
            let notes = run.iter().filter(|piece| piece.is_note).count();
            if beat_index.is_some() && notes >= 2 {
                format!("<beam>{inner}</beam>")
            } else {
                inner
            }
        })
        .join("")
}

/// Opens and closes slurs from the note's `( )` group depth.
fn track_slurs(
    note: &GroupedNote,
    id: &str,
    context: &LayerContext,
    state: &mut StaffState,
    slurs: &mut Vec<Slur>,
) {
    let opens = note
        .group_membership
        .saturating_sub(state.previous_continuation);
    state.open_slurs.extend((0..opens).map(|_| OpenSlur {
        start_id: id.to_string(),
        start_measure: context.measure_index,
    }));
    let closes = note
        .group_membership
        .saturating_sub(note.group_continuation);
    for _ in 0..closes {
        let Some(open) = state.open_slurs.pop() else {
            break;
        };
        if open.start_id != id {
            slurs.push(Slur {
                staff: context.staff_number,
                start_measure: open.start_measure,
                start_id: open.start_id,
                end_id: id.to_string(),
            });
        }
    }
    state.previous_continuation = note.group_continuation;
}

/// A lyrics-only staff's layer: its target's rhythm as invisible notes
/// carrying the verses, with rests as spaces. No beams, ties or slurs, so
/// nothing but the syllables is drawn.
pub(super) fn lyrics_only_layer_xml(
    lyrics: &LyricsSlice,
    resolution_multiplier: u32,
    verses: &[MeasureSyllables],
) -> String {
    let events = &lyrics.target_events;
    if events
        .iter()
        .all(|event| !matches!(event, NoteEvent::Note(_)))
    {
        return "<mSpace/>".to_string();
    }
    let multiplier = resolution_multiplier.max(1);
    events
        .iter()
        .enumerate()
        .flat_map(|(event_index, event)| {
            let is_note = matches!(event, NoteEvent::Note(_));
            let syllables = verses_xml(verses, event_index);
            split_undotted_duration(event.duration() / multiplier)
                .into_iter()
                .enumerate()
                .map(move |(index, duration)| {
                    let dur = format!("dur=\"{}\"", duration.dur);
                    match (is_note, index == 0 && !syllables.is_empty()) {
                        (false, _) => format!("<space {dur}/>"),
                        (true, true) => format!("<note {INVISIBLE_NOTE} {dur}>{syllables}</note>"),
                        (true, false) => format!("<note {INVISIBLE_NOTE} {dur}/>"),
                    }
                })
                .collect_vec()
        })
        .join("")
}

/// Sits on the single line of a lyrics-only staff, so no ledger lines.
const INVISIBLE_NOTE: &str = r#"pname="g" oct="4" head.visible="false" stem.visible="false""#;
