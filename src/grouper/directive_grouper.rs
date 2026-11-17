use crate::ast::grouped::{MeasureDirectives, TimeSignature};
use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName, ScoreEvent};
use crate::error::Spanned;

pub(super) struct DirectiveGrouper {
    current_bpm: u32,
    current_time_sig: TimeSignature,
    current_key: KeyChange,
    current_merge_duplicate_measures_across_parts: bool,
    current_hide_resting_parts: bool,
    bpm_changed: bool,
    time_sig_changed: bool,
    key_changed: bool,
}

impl DirectiveGrouper {
    /// `merge_duplicate_measures_across_parts`/`hide_resting_parts` seed the sticky
    /// carry-forward state with the score-wide `metadata` default (or its hardcoded
    /// default when unset in `metadata`); a directive line can override either from
    /// the measure it appears on until the next occurrence.
    pub(super) fn new(
        merge_duplicate_measures_across_parts: bool,
        hide_resting_parts: bool,
    ) -> Self {
        Self {
            current_bpm: 120,
            current_time_sig: TimeSignature {
                numerator: 4,
                denominator: 4,
            },
            current_key: KeyChange {
                note: Note {
                    name: NoteName::C,
                    octave: 4,
                    accidental: Accidental::Natural,
                },
            },
            current_merge_duplicate_measures_across_parts: merge_duplicate_measures_across_parts,
            current_hide_resting_parts: hide_resting_parts,
            bpm_changed: true,
            time_sig_changed: true,
            key_changed: true,
        }
    }

    pub(super) fn process_all(
        mut self,
        directive_events_per_measure: &[Vec<Spanned<ScoreEvent>>],
    ) -> Vec<MeasureDirectives> {
        let mut result = Vec::new();
        for events in directive_events_per_measure {
            let mut pending_label: Option<String> = None;
            let mut pending_system_break = false;
            for event in events {
                match &event.value {
                    ScoreEvent::BpmChange(bpm) => {
                        if *bpm != self.current_bpm {
                            self.bpm_changed = true;
                        }
                        self.current_bpm = *bpm;
                    }
                    ScoreEvent::TimeSignatureChange {
                        numerator,
                        denominator,
                    } => {
                        let new_time_sig = TimeSignature {
                            numerator: *numerator,
                            denominator: *denominator,
                        };
                        if new_time_sig != self.current_time_sig {
                            self.time_sig_changed = true;
                        }
                        self.current_time_sig = new_time_sig;
                    }
                    ScoreEvent::KeyChange(kc) => {
                        if *kc != self.current_key {
                            self.key_changed = true;
                        }
                        self.current_key = kc.clone();
                    }
                    ScoreEvent::LabelChange(text) => {
                        pending_label = Some(text.clone());
                    }
                    ScoreEvent::MergeDuplicateMeasuresAcrossPartsChange(value) => {
                        self.current_merge_duplicate_measures_across_parts = *value;
                    }
                    ScoreEvent::HideRestingPartsChange(value) => {
                        self.current_hide_resting_parts = *value;
                    }
                    ScoreEvent::SystemBreak => {
                        pending_system_break = true;
                    }
                    _ => {}
                }
            }
            result.push(MeasureDirectives {
                bpm: if self.bpm_changed {
                    Some(self.current_bpm)
                } else {
                    None
                },
                time_signature: if self.time_sig_changed {
                    Some(TimeSignature {
                        numerator: self.current_time_sig.numerator,
                        denominator: self.current_time_sig.denominator,
                    })
                } else {
                    None
                },
                key: if self.key_changed {
                    Some(self.current_key.clone())
                } else {
                    None
                },
                label: pending_label,
                merge_duplicate_measures_across_parts: self
                    .current_merge_duplicate_measures_across_parts,
                hide_resting_parts: self.current_hide_resting_parts,
                system_break: pending_system_break,
            });
            self.bpm_changed = false;
            self.time_sig_changed = false;
            self.key_changed = false;
        }
        result
    }
}
