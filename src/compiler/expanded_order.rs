use super::{compile_in_order, compile_measure, CompileResult, PartCrossState};
use crate::ast::grouped::{MultiPartMeasure, Score};
use crate::midi::ExpandedMeasureOrigin;

pub(super) struct ExpandedOrder<'a> {
    origins: &'a [ExpandedMeasureOrigin],
    note_states: Vec<Vec<NoteState>>,
}

#[derive(Clone, Copy)]
struct NoteState {
    next_note_id: usize,
    tie_note_id: Option<usize>,
}

pub(crate) fn compile_expanded(
    written: &Score,
    expanded: &Score,
    origins: &[ExpandedMeasureOrigin],
) -> CompileResult {
    let mut cross_states = Vec::new();
    let note_states = written
        .measures
        .iter()
        .enumerate()
        .map(|(index, measure)| {
            cross_states.resize_with(measure.parts.len(), PartCrossState::new);
            let states = cross_states
                .iter()
                .map(|state| NoteState {
                    next_note_id: state.next_note_id,
                    tie_note_id: state.prev_tie_note_id,
                })
                .collect();
            compile_measure(
                measure,
                index,
                &mut cross_states,
                &mut Vec::new(),
                &mut Vec::new(),
                None,
            );
            states
        })
        .collect();
    compile_in_order(
        expanded,
        Some(&ExpandedOrder {
            origins,
            note_states,
        }),
    )
}

impl ExpandedOrder<'_> {
    pub(super) fn prepare_measure(
        &self,
        measure: &MultiPartMeasure,
        index: usize,
        states: &mut Vec<PartCrossState>,
    ) -> Option<&[usize]> {
        let origin = self.origins.get(index)?;
        let written = self.note_states.get(origin.written_measure_index)?;

        // Each occurrence starts fresh for arc geometry. Event IDs still
        // follow the written score, even when a different line is played.
        if measure.label.is_some() {
            states.clear();
        }
        states.resize_with(measure.parts.len(), PartCrossState::new);
        for (state, &part) in states.iter_mut().zip(&origin.part_written_indices) {
            if let Some(written) = written.get(part) {
                state.next_note_id = written.next_note_id;
                state.prev_tie_note_id = written.tie_note_id;
            }
        }
        Some(&origin.part_written_indices)
    }
}
