//! Describes a single selected note or chord in letter names — the data the
//! web editor's pitch drawer presents (see `describe_selection`).

mod chord_template;
mod guitar_diagram_svg;
mod guitar_voicing;
pub use guitar_voicing::GuitarVoicing;
mod piano_diagram_svg;
pub(crate) mod spelling;

use itertools::Itertools;

use crate::ast::grouped::{GroupedChordNote, NoteEvent, Score};
use crate::ast::parsed::{Accidental, Extension, KeyChange, Note, NoteName, TriadQuality};
use crate::error::Span;
use crate::source_edit::ByteRange;

use chord_template::chord_template;
use guitar_diagram_svg::render_guitar_diagram_svg;
use guitar_voicing::{chords_db_bass_name, find_guitar_voicings};
use piano_diagram_svg::{
    interval_function_name, interval_label, render_piano_diagram_svg, PianoTone,
};
use spelling::{Interval, SpelledPitch};

/// What one selected note or chord means in letter names, resolved against
/// the key in effect at its measure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PitchDescription {
    Note(NoteDescription),
    Chord(ChordDescription),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteDescription {
    /// e.g. `"F#"`. Octave is not included.
    pub letter_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChordDescription {
    /// e.g. `"Cm7"`, `"C/G"`.
    pub chord_name: String,
    /// Chord tones from the root up, e.g. `["C", "Eb", "G", "Bb"]`. Excludes
    /// a slash chord's bass note, which is in `bass_note`.
    pub tone_names: Vec<String>,
    /// A slash chord's bass note, e.g. `"G"` for `C/G`.
    pub bass_note: Option<String>,
    /// One chord-box SVG per chords-db voicing, most common first (usually
    /// the open shape, then shapes further up the neck); empty when chords-db
    /// has no voicing for this chord kind (e.g. `7sus2`). A slash chord with
    /// no slash-specific voicing falls back to its plain chord's.
    pub guitar_diagram_svgs: Vec<String>,
    /// Keyboard SVG spanning just the chord tones plus padding; each tone is
    /// labelled with its interval from the chord root (`1 3 5`, `1 b3 5`).
    /// Always root position, unless the slash bass is one of the triad's
    /// tones, in which case that tone is lowest.
    pub piano_diagram_svg: String,
}

/// Describes the selection when it covers exactly one sounding note or
/// chord — a tied run counts as one — and `None` otherwise: an empty
/// (caret-only) selection, more than one note, a rest, a percussion hit, or
/// a source that fails to compile.
pub fn describe_selection(source: &str, ranges: &[ByteRange]) -> Option<PitchDescription> {
    let score = crate::compile(source, "input.poentaro", &[]).ok()?;
    let selected = selected_tie_chains(&score, ranges);
    match selected.as_slice() {
        [only] => describe_event(only.event, &only.key),
        _ => None,
    }
}

/// Identifies one tied run of events within one part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TieChainId {
    part_index: usize,
    chain_index: usize,
}

struct SelectedEvent<'score> {
    tie_chain: TieChainId,
    event: &'score NoteEvent,
    key: KeyChange,
}

/// Every event overlapping `ranges`, deduplicated to the first event of each
/// tie chain, in score order.
fn selected_tie_chains<'score>(
    score: &'score Score,
    ranges: &[ByteRange],
) -> Vec<SelectedEvent<'score>> {
    // Overwritten by the first measure, whose `key` the grouper always sets.
    let mut active_key = KeyChange {
        note: Note {
            name: NoteName::C,
            octave: 4,
            accidental: Accidental::Natural,
        },
    };
    let mut next_chain_index = 0;
    let mut open_chain_per_part: Vec<Option<usize>> = Vec::new();
    let mut selected = Vec::new();

    for measure in &score.measures {
        if let Some(key) = &measure.key {
            active_key = key.clone();
        }
        open_chain_per_part.resize(open_chain_per_part.len().max(measure.parts.len()), None);
        for (part_index, part_row) in measure.parts.iter().enumerate() {
            for event in &part_row.slice().notes.events {
                let open_chain = open_chain_per_part.get_mut(part_index);
                let chain_index = match open_chain.as_deref().copied().flatten() {
                    Some(chain_index) => chain_index,
                    None => {
                        next_chain_index += 1;
                        next_chain_index
                    }
                };
                if let Some(open_chain) = open_chain {
                    *open_chain = event_ties_to_next(event).then_some(chain_index);
                }
                if event_overlaps(event, ranges) {
                    selected.push(SelectedEvent {
                        tie_chain: TieChainId {
                            part_index,
                            chain_index,
                        },
                        event,
                        key: active_key.clone(),
                    });
                }
            }
        }
    }

    selected
        .into_iter()
        .unique_by(|selected_event| selected_event.tie_chain)
        .collect()
}

fn event_ties_to_next(event: &NoteEvent) -> bool {
    match event {
        NoteEvent::Note(note) => note.tie_to_next(),
        NoteEvent::Chord(chord) => chord.tie_to_next(),
        NoteEvent::Percussion(hit) => hit.tie_to_next(),
        NoteEvent::Rest(_) => false,
    }
}

/// Whether `event`'s source token shares a byte with any of `ranges`. An
/// implicit-fill rest has no token of its own, so it never overlaps.
fn event_overlaps(event: &NoteEvent, ranges: &[ByteRange]) -> bool {
    let span: Span = match event {
        NoteEvent::Note(note) => note.event_span,
        NoteEvent::Chord(chord) => chord.event_span,
        NoteEvent::Percussion(hit) => hit.event_span,
        NoteEvent::Rest(rest) if rest.implicit_fill => return false,
        NoteEvent::Rest(rest) => rest.event_span,
    };
    ranges
        .iter()
        .any(|range| span.start < range.end_byte as usize && span.end > range.start_byte as usize)
}

fn describe_event(event: &NoteEvent, key: &KeyChange) -> Option<PitchDescription> {
    let tonic = SpelledPitch::key_tonic(key);
    match event {
        NoteEvent::Note(note) => Some(PitchDescription::Note(NoteDescription {
            letter_name: tonic.scale_degree(&note.pitch, &note.accidental).name(),
        })),
        NoteEvent::Chord(chord) => Some(PitchDescription::Chord(describe_chord(chord, tonic))),
        NoteEvent::Rest(_) | NoteEvent::Percussion(_) => None,
    }
}

fn describe_chord(chord: &GroupedChordNote, tonic: SpelledPitch) -> ChordDescription {
    let root = tonic.scale_degree(&chord.degree, &chord.accidental);
    let bass = chord
        .bass
        .as_ref()
        .map(|bass| tonic.scale_degree(&bass.degree, &bass.accidental));
    let template = chord_template(&chord.triad, chord.extension.as_ref());
    let chord_name = format!(
        "{}{}{}",
        root.name(),
        template.name_suffix,
        bass.map(|bass| format!("/{}", bass.name()))
            .unwrap_or_default(),
    );
    let guitar_diagram_svgs = guitar_voicings(&chord.triad, chord.extension.as_ref(), root, bass)
        .into_iter()
        .map(|voicing| render_guitar_diagram_svg(voicing, &chord_name))
        .collect();
    let piano_diagram_svg =
        render_piano_diagram_svg(&piano_tones(root, &template.intervals, bass), &chord_name);
    ChordDescription {
        piano_diagram_svg,
        tone_names: template
            .intervals
            .iter()
            .map(|&interval| root.up(interval).name())
            .collect(),
        bass_note: bass.map(SpelledPitch::name),
        guitar_diagram_svgs,
        chord_name,
    }
}

/// The chord tones as keys, stacked upward from the root. A slash bass that
/// is one of the triad's tones (not the seventh) inverts the chord so that
/// tone is lowest; any other bass leaves it in root position.
fn piano_tones(
    root: SpelledPitch,
    intervals: &[Interval],
    bass: Option<SpelledPitch>,
) -> Vec<PianoTone> {
    const ROOT_MIDI_OCTAVE_START: i16 = 60;
    let root_midi = ROOT_MIDI_OCTAVE_START + i16::from(root.pitch_class());
    let inverted_from = bass.and_then(|bass| {
        intervals
            .iter()
            .take(TRIAD_TONE_COUNT)
            .position(|&interval| root.up(interval).pitch_class() == bass.pitch_class())
    });
    intervals
        .iter()
        .enumerate()
        .map(|(index, &interval)| {
            let raised_an_octave = inverted_from.is_some_and(|bass_index| index < bass_index);
            PianoTone {
                midi: root_midi
                    + i16::from(interval.semitones)
                    + if raised_an_octave { 12 } else { 0 },
                label: interval_label(interval),
                function_name: interval_function_name(interval),
            }
        })
        .collect()
}

const TRIAD_TONE_COUNT: usize = 3;

/// Every chords-db voicing of the chord rooted on `root`, most common first;
/// empty when chords-db has none (e.g. `7sus2`). A slash chord with no
/// slash-specific voicing falls back to its plain chord's.
pub(crate) fn guitar_voicings(
    triad: &TriadQuality,
    extension: Option<&Extension>,
    root: SpelledPitch,
    bass: Option<SpelledPitch>,
) -> Vec<&'static GuitarVoicing> {
    let slash_voicings = bass
        .map(|bass| slash_chord_voicings(triad, extension, root, bass))
        .unwrap_or_default();
    if !slash_voicings.is_empty() {
        return slash_voicings;
    }
    chord_template(triad, extension)
        .chords_db_suffix
        .map(|suffix| find_guitar_voicings(root.pitch_class(), suffix))
        .unwrap_or_default()
}

/// chords-db only has slash voicings for plain major/minor triads, under
/// suffixes like `"/G"` and `"m/C"`.
fn slash_chord_voicings(
    triad: &TriadQuality,
    extension: Option<&Extension>,
    root: SpelledPitch,
    bass: SpelledPitch,
) -> Vec<&'static GuitarVoicing> {
    if extension.is_some() {
        return vec![];
    }
    let triad_prefix = match triad {
        TriadQuality::Major => "",
        TriadQuality::Minor => "m",
        _ => return vec![],
    };
    let suffix = format!("{triad_prefix}/{}", chords_db_bass_name(bass.pitch_class()));
    find_guitar_voicings(root.pitch_class(), &suffix)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
