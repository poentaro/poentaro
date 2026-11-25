use crate::ast::grouped::{
    GroupedChordNote, GroupedMeasure, GroupedNote, GroupedPart, GroupedPercussionHit, GroupedRest,
    NoteEvent, Notes, TimeSignature,
};
use crate::ast::parsed::{
    ParsedChordNote, ParsedMeasureSlot, ParsedNote, ParsedPercussionHit, ParsedRest,
    ParsedTimedTrack, PartKind, ScoreEvent, Soundfont,
};
use crate::error::{Diagnostic, IrrecoverableError, RecoverableError, Span, Warning};

use super::empty_note_measures::{align_empty_note_measures, MeasureSlot, PerMeasureErrors};
use super::lyrics_pairing::pair_lyrics_measures;

#[path = "part_grouper_group.rs"]
mod part_grouper_group;
pub(super) use part_grouper_group::{group_lyrics_track, group_timed_track};

struct PartGrouper {
    part_kind: PartKind,
    soundfont: Soundfont,
    slots: Vec<MeasureSlot>,
    current_notes: Vec<NoteEvent>,
    current_beat: u32,
    capacity: u32,
    /// Quarter-beat width of one beam group under the time signature currently in
    /// effect (see `GroupedMeasure::beat_group_size`): `4` for simple meters, `6`
    /// for compound meters (6/8, 9/8, 12/8, ...).
    beat_group_size: u32,
    /// Tuplet-rescale factor for the measure currently being accumulated (see
    /// `crate::tuplet::apply_resolution_multiplier`), set via `begin_measure_slot` before that
    /// measure's events are pushed. `1` for measures with no tuplets.
    resolution_multiplier: u32,
    part_name: Option<String>,
    measure_span_start: Option<usize>,
    measure_span_end: usize,
    pending_overflow_error: Option<Warning>,
    pending_dotted_eighth_errors: Vec<Diagnostic>,
    pending_extension_no_preceding_event_error: Option<RecoverableError>,
}

impl PartGrouper {
    fn new(part: &ParsedTimedTrack) -> Self {
        let current_time_sig = TimeSignature {
            numerator: 4,
            denominator: 4,
        };
        let capacity = Self::measure_capacity(&current_time_sig);

        Self {
            part_kind: part.kind,
            soundfont: part.soundfont,
            slots: Vec::new(),
            current_notes: Vec::new(),
            current_beat: 0,
            capacity,
            beat_group_size: Self::beat_group_size(&current_time_sig),
            resolution_multiplier: 1,
            part_name: Some(part.abbreviation.clone()),
            measure_span_start: None,
            measure_span_end: 0,
            pending_overflow_error: None,
            pending_dotted_eighth_errors: Vec::new(),
            pending_extension_no_preceding_event_error: None,
        }
    }

    fn measure_capacity(ts: &TimeSignature) -> u32 {
        (ts.numerator as u32) * 16 / (ts.denominator as u32)
    }

    /// Compound meters (denominator 8, numerator a multiple of 3 greater than 3 —
    /// 6/8, 9/8, 12/8, ...) beam in groups of one dotted quarter (3 eighth notes);
    /// every other meter beams in groups of one quarter note.
    fn beat_group_size(ts: &TimeSignature) -> u32 {
        if ts.denominator == 8 && ts.numerator % 3 == 0 && ts.numerator > 3 {
            6
        } else {
            4
        }
    }

    /// Sets the tuplet-rescale multiplier to apply to `self.capacity` while accumulating
    /// the measure about to be pushed via `process_event`. Stays in effect until that
    /// measure is actually flushed (`flush_measure` resets it back to `1`), regardless of
    /// how many `process_event` calls that takes — a measure isn't always flushed by the
    /// time all of its events have been pushed (e.g. the score's trailing measure, only
    /// flushed later by `finish`).
    fn begin_measure_slot(&mut self, resolution_multiplier: u32) {
        self.resolution_multiplier = resolution_multiplier;
    }

    /// `self.capacity` scaled by the current measure's tuplet-rescale multiplier — the
    /// unit `current_beat` must be compared against while a tuplet-rescaled measure is
    /// being accumulated (see `crate::tuplet::apply_resolution_multiplier`).
    fn effective_capacity(&self) -> u32 {
        self.capacity * self.resolution_multiplier
    }

    fn flush_measure(&mut self) {
        if self.current_notes.is_empty() {
            return;
        }
        let source_span = Span::new(self.measure_span_start.unwrap_or(0), self.measure_span_end);
        let measure = GroupedMeasure {
            notes: Notes {
                events: std::mem::take(&mut self.current_notes),
            },
            source_span,
            paired_lyrics: Vec::new(),
            lyrics_error: Vec::new(),
            beat_overflow_error: self.pending_overflow_error.take(),
            dotted_eighth_errors: std::mem::take(&mut self.pending_dotted_eighth_errors),
            chord_errors: Vec::new(),
            lex_error: None,
            lyrics_parse_error: None,
            extension_no_preceding_event_error: self
                .pending_extension_no_preceding_event_error
                .take(),
            resolution_multiplier: self.resolution_multiplier,
            beat_group_size: self.beat_group_size,
        };
        self.slots.push(MeasureSlot::Real(Box::new(measure)));
        self.current_beat = 0;
        self.measure_span_start = None;
        self.measure_span_end = 0;
        self.resolution_multiplier = 1;
    }

    fn push_empty_note_slot(&mut self, span: Span) {
        self.slots.push(MeasureSlot::EmptyNote { span });
    }

    fn flush_if_full(&mut self) {
        if self.current_beat >= self.effective_capacity() {
            self.flush_measure();
        }
    }

    fn with_part_prefix(&self, message: String) -> String {
        match &self.part_name {
            Some(name) => format!("[{name}] {message}"),
            None => message,
        }
    }

    fn push_timed_event(
        &mut self,
        span: Span,
        duration: u32,
        event: NoteEvent,
        overflow_label: &str,
    ) -> Result<(), IrrecoverableError> {
        self.flush_if_full();
        if self.measure_span_start.is_none() {
            self.measure_span_start = Some(span.start);
        }
        self.measure_span_end = span.end;
        self.current_notes.push(event);
        self.current_beat += duration;
        let effective_capacity = self.effective_capacity();
        if self.current_beat > effective_capacity {
            self.current_notes.pop();
            self.current_beat -= duration;
            let message = self.with_part_prefix(format!(
                "beat overflow: {overflow_label} exceeds measure capacity of {} quarter-beats; note dropped",
                self.capacity,
            ));
            self.pending_overflow_error
                .get_or_insert_with(|| Warning::new(span, message));
            self.flush_measure();
            return Ok(());
        }
        if self.current_beat == effective_capacity {
            self.flush_measure();
        }
        Ok(())
    }

    fn handle_extension(
        &mut self,
        span: Span,
        dotted: bool,
        double_dotted: bool,
    ) -> Result<(), IrrecoverableError> {
        self.measure_span_end = span.end.max(self.measure_span_end);
        let extension_beats = if double_dotted {
            7
        } else if dotted {
            6
        } else {
            4
        } * self.resolution_multiplier;
        match self.current_notes.last_mut() {
            Some(NoteEvent::Note(n)) => {
                n.duration += extension_beats;
                self.current_beat += extension_beats;
            }
            Some(NoteEvent::Chord(c)) => {
                c.duration += extension_beats;
                self.current_beat += extension_beats;
            }
            Some(NoteEvent::Percussion(p)) => {
                p.duration += extension_beats;
                self.current_beat += extension_beats;
            }
            Some(NoteEvent::Rest(r)) => {
                r.duration += extension_beats;
                self.current_beat += extension_beats;
            }
            None => {
                let chord_track = self.part_kind == PartKind::Chords;
                self.pending_extension_no_preceding_event_error
                    .get_or_insert_with(|| {
                        RecoverableError::extension_no_preceding_event(span, chord_track)
                    });
                return Ok(());
            }
        }
        if self.current_beat >= self.effective_capacity() {
            self.flush_measure();
        }
        Ok(())
    }

    fn handle_tie_marker(&mut self, _span: Span) -> Result<(), IrrecoverableError> {
        let last_event = self.current_notes.last_mut().or_else(|| {
            self.slots.iter_mut().rev().find_map(|slot| match slot {
                MeasureSlot::Real(m) => m.notes.events.last_mut(),
                MeasureSlot::EmptyNote { .. } => None,
            })
        });
        match last_event {
            Some(NoteEvent::Note(n)) => {
                n.slur = true;
                Ok(())
            }
            Some(NoteEvent::Chord(c)) => {
                c.slur = true;
                Ok(())
            }
            // TieMarker is a legacy event that is never emitted by the parser;
            // this arm is dead code but kept for exhaustiveness.
            _ => Ok(()),
        }
    }

    fn handle_note(&mut self, span: Span, pn: ParsedNote) -> Result<(), IrrecoverableError> {
        self.push_timed_event(
            span,
            pn.duration,
            NoteEvent::Note(GroupedNote {
                slur_continuation: pn.slur_continuation,
                pitch: pn.pitch,
                accidental: pn.accidental,
                octave: pn.octave,
                duration: pn.duration,
                slur: pn.slur && pn.slur_group_close_at_duration.is_none(),
                tie_to_next_span: pn.tie_to_next_span,
                event_span: span,
                group_membership: pn.group_membership,
                group_continuation: pn.group_continuation,
                dotted: pn.dotted,
                double_dotted: pn.double_dotted,
                fermata: pn.fermata,
                slur_group_close_at_duration: pn.slur_group_close_at_duration,
                tuplet: pn.tuplet,
            }),
            "note",
        )
    }

    fn handle_chord(&mut self, span: Span, pc: ParsedChordNote) -> Result<(), IrrecoverableError> {
        self.push_timed_event(
            span,
            pc.duration,
            NoteEvent::Chord(GroupedChordNote {
                degree: pc.degree,
                accidental: pc.accidental,
                triad: pc.triad,
                extension: pc.extension,
                bass: pc.bass,
                duration: pc.duration,
                slur: pc.slur && pc.slur_group_close_at_duration.is_none(),
                tie_to_next_span: pc.tie_to_next_span,
                event_span: span,
                group_membership: pc.group_membership,
                group_continuation: pc.group_continuation,
                dotted: pc.dotted,
                double_dotted: pc.double_dotted,
                slur_group_close_at_duration: pc.slur_group_close_at_duration,
                tuplet: pc.tuplet,
            }),
            "chord",
        )
    }

    fn handle_percussion_hit(
        &mut self,
        span: Span,
        ph: &ParsedPercussionHit,
    ) -> Result<(), IrrecoverableError> {
        self.push_timed_event(
            span,
            ph.duration,
            NoteEvent::Percussion(GroupedPercussionHit {
                duration: ph.duration,
                slur: ph.slur && ph.slur_group_close_at_duration.is_none(),
                tie_to_next_span: ph.tie_to_next_span,
                event_span: span,
                group_membership: ph.group_membership,
                group_continuation: ph.group_continuation,
                dotted: ph.dotted,
                double_dotted: ph.double_dotted,
                slur_group_close_at_duration: ph.slur_group_close_at_duration,
                tuplet: ph.tuplet,
            }),
            "percussion hit",
        )
    }

    fn handle_rest(&mut self, span: Span, pr: &ParsedRest) -> Result<(), IrrecoverableError> {
        self.push_timed_event(
            span,
            pr.duration,
            NoteEvent::Rest(GroupedRest {
                duration: pr.duration,
                dotted: pr.dotted,
                double_dotted: pr.double_dotted,
                fermata: pr.fermata,
                event_span: span,
                group_membership: pr.group_membership,
                group_continuation: pr.group_continuation,
                tuplet: pr.tuplet,
                implicit_fill: pr.implicit_fill,
            }),
            "rest",
        )
    }

    fn process_event(
        &mut self,
        spanned: crate::error::Spanned<ScoreEvent>,
    ) -> Result<(), IrrecoverableError> {
        match spanned.value {
            ScoreEvent::BpmChange(_)
            | ScoreEvent::KeyChange(_)
            | ScoreEvent::LabelChange(_)
            | ScoreEvent::MergeDuplicateMeasuresAcrossPartsChange(_)
            | ScoreEvent::HideRestingPartsChange(_)
            | ScoreEvent::SystemBreak => {
                Ok(()) // handled by DirectiveGrouper
            }
            ScoreEvent::TimeSignatureChange {
                numerator,
                denominator,
            } => {
                let ts = TimeSignature {
                    numerator,
                    denominator,
                };
                self.capacity = Self::measure_capacity(&ts);
                self.beat_group_size = Self::beat_group_size(&ts);
                Ok(())
            }
            ScoreEvent::Extension {
                dotted,
                double_dotted,
            } => self.handle_extension(spanned.span, dotted, double_dotted),
            ScoreEvent::TieMarker => self.handle_tie_marker(spanned.span),
            ScoreEvent::Note(pn) => self.handle_note(spanned.span, pn),
            ScoreEvent::Chord(pc) => self.handle_chord(spanned.span, pc),
            ScoreEvent::PercussionHit(ph) => self.handle_percussion_hit(spanned.span, &ph),
            ScoreEvent::Rest(pr) => self.handle_rest(spanned.span, &pr),
        }
    }

    fn finish(mut self) -> (Vec<MeasureSlot>, Option<String>, PartKind, Soundfont) {
        self.flush_measure();

        (self.slots, self.part_name, self.part_kind, self.soundfont)
    }
}
