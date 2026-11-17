use crate::ast::grouped::{NoteEvent, Score};
use crate::ast::parsed::{Accidental, NumberedPitch};
use crate::error::{Diagnostic, RecoverableError, Span};

struct NoteInfo {
    measure_idx: usize,
    event_idx: usize,
    pitch: NumberedPitch,
    accidental: Accidental,
    octave: i8,
    tie_span: Option<Span>,
    event_span: Span,
}

struct TieCorrection {
    measure_idx: usize,
    event_idx: usize,
    error: RecoverableError,
}

fn pitch_to_char(pitch: &NumberedPitch) -> char {
    match pitch {
        NumberedPitch::One => '1',
        NumberedPitch::Two => '2',
        NumberedPitch::Three => '3',
        NumberedPitch::Four => '4',
        NumberedPitch::Five => '5',
        NumberedPitch::Six => '6',
        NumberedPitch::Seven => '7',
    }
}

fn format_pitch_octave(pitch: &NumberedPitch, octave: i8) -> String {
    let ch = pitch_to_char(pitch);
    let octave_suffix = match octave.cmp(&0) {
        std::cmp::Ordering::Greater => "'".repeat(octave as usize),
        std::cmp::Ordering::Less => ",".repeat((-octave) as usize),
        std::cmp::Ordering::Equal => String::new(),
    };
    format!("{ch}{octave_suffix}")
}

fn format_pitch_octave_accidental(
    pitch: &NumberedPitch,
    accidental: &Accidental,
    octave: i8,
) -> String {
    let acc_str = match accidental {
        Accidental::Sharp => "#",
        Accidental::Flat => "b",
        Accidental::Natural => "",
    };
    format!("{}{}", format_pitch_octave(pitch, octave), acc_str)
}

fn collect_notes_for_part(score: &Score, part_idx: usize) -> Vec<NoteInfo> {
    score
        .measures
        .iter()
        .enumerate()
        .flat_map(|(measure_idx, measure)| -> Vec<NoteInfo> {
            let Some(part_row) = measure.parts.get(part_idx) else {
                return Vec::new();
            };
            part_row
                .slice()
                .notes
                .events
                .iter()
                .enumerate()
                .filter_map(move |(event_idx, event)| {
                    if let NoteEvent::Note(n) = event {
                        Some(NoteInfo {
                            measure_idx,
                            event_idx,
                            pitch: n.pitch.clone(),
                            accidental: n.accidental.clone(),
                            octave: n.octave,
                            tie_span: n.tie_to_next_span,
                            event_span: n.event_span,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect()
}

fn tie_error_span(tie_span: Option<Span>, event_span: Span) -> Span {
    tie_span.unwrap_or(event_span)
}

fn tie_pitch_mismatch_span(
    tie_span: Option<Span>,
    tied_event_span: Span,
    next_event_span: Span,
) -> Span {
    match tie_span {
        Some(tie) => Span::new(tie.start, next_event_span.end),
        None => Span::new(tied_event_span.start, next_event_span.end),
    }
}

fn tie_corrections(notes: &[NoteInfo]) -> Vec<TieCorrection> {
    notes
        .iter()
        .enumerate()
        .filter(|(_, note)| note.tie_span.is_some())
        .filter_map(|(i, note)| {
            let next = notes.get(i + 1);
            let error = match next {
                None => Some(RecoverableError::dangling_tie(tie_error_span(
                    note.tie_span,
                    note.event_span,
                ))),
                Some(next_note)
                    if next_note.pitch != note.pitch
                        || next_note.octave != note.octave
                        || next_note.accidental != note.accidental =>
                {
                    let expected =
                        format_pitch_octave_accidental(&note.pitch, &note.accidental, note.octave);
                    let got = format_pitch_octave_accidental(
                        &next_note.pitch,
                        &next_note.accidental,
                        next_note.octave,
                    );
                    Some(RecoverableError::tie_pitch_mismatch(
                        tie_pitch_mismatch_span(
                            note.tie_span,
                            note.event_span,
                            next_note.event_span,
                        ),
                        expected,
                        got,
                    ))
                }
                Some(_) => None,
            };
            error.map(|err| TieCorrection {
                measure_idx: note.measure_idx,
                event_idx: note.event_idx,
                error: err,
            })
        })
        .collect()
}

fn apply_tie_corrections(score: &mut Score, part_idx: usize, corrections: Vec<TieCorrection>) {
    for correction in corrections {
        if let Some(measure) = score.measures.get_mut(correction.measure_idx) {
            measure
                .diagnostics
                .push(Diagnostic::Error(correction.error));
            if let Some(part_row) = measure.parts.get_mut(part_idx) {
                if let Some(NoteEvent::Note(n)) = part_row
                    .slice_mut()
                    .notes
                    .events
                    .get_mut(correction.event_idx)
                {
                    n.tie_to_next_span = None;
                }
            }
        }
    }
}

pub(super) fn validate_ties(score: &mut Score) {
    let num_parts = score.measures.first().map_or(0, |m| m.parts.len());
    for part_idx in 0..num_parts {
        let notes = collect_notes_for_part(score, part_idx);
        let corrections = tie_corrections(&notes);
        apply_tie_corrections(score, part_idx, corrections);
    }
}
