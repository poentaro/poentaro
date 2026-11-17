use crate::ast::grouped::{Score, TimeSignature};
use crate::ast::parsed::KeyChange;
#[cfg(test)]
use crate::error::IrrecoverableError;

#[cfg(test)]
use super::write_midi_with_metronome;

pub fn build_measure_range_score(
    score: &Score,
    start_index: usize,
    end_index: usize,
) -> Option<Score> {
    if score.measures.is_empty() {
        return None;
    }
    let last = score.measures.len() - 1;
    let (start_index, end_index) = if start_index > end_index {
        (end_index.min(last), start_index.min(last))
    } else {
        (start_index.min(last), end_index.min(last))
    };
    let mut accumulated_bpm: Option<u32> = None;
    let mut accumulated_key: Option<KeyChange> = None;
    let mut accumulated_time_signature: Option<TimeSignature> = None;
    for measure in score.measures.iter().take(start_index) {
        if let Some(bpm) = measure.bpm {
            accumulated_bpm = Some(bpm);
        }
        if let Some(key) = &measure.key {
            accumulated_key = Some(key.clone());
        }
        if let Some(time_signature) = &measure.time_signature {
            accumulated_time_signature = Some(time_signature.clone());
        }
    }
    let count = end_index - start_index + 1;
    let mut measures: Vec<_> = score
        .measures
        .iter()
        .skip(start_index)
        .take(count)
        .cloned()
        .collect();
    if let Some(first) = measures.first_mut() {
        if first.bpm.is_none() {
            first.bpm = accumulated_bpm;
        }
        if first.key.is_none() {
            first.key = accumulated_key;
        }
        if first.time_signature.is_none() {
            first.time_signature = accumulated_time_signature;
        }
    }
    Some(Score {
        metadata: score.metadata.clone(),
        measures,
        document_diagnostics: vec![],
        sequence: None,
    })
}

/// The MIDI of written measures `start_index..=end_index`, with the
/// metronome of [`write_midi_with_metronome`] when `metronome` is set.
/// Only tests write a range's MIDI on its own; playback builds it with its
/// vocals in `audio_source`.
#[cfg(test)]
pub(crate) fn write_midi_for_measure_range_with_metronome(
    score: &Score,
    start_index: usize,
    end_index: usize,
    metronome: bool,
) -> Result<Vec<u8>, IrrecoverableError> {
    let Some(range_score) = build_measure_range_score(score, start_index, end_index) else {
        return Ok(Vec::new());
    };
    write_midi_with_metronome(&range_score, metronome)
}
