use crate::ast::grouped::Score;
use crate::ast::parsed::KeyChange;
use crate::error::IrrecoverableError;

/// Rebuilds `score.measures` to reflect a `sequence` section's resolved
/// playback order, so downstream MIDI/WAV generation replays measures in
/// actual playback order instead of written order.
///
/// - No `sequence` section present: returns the score unchanged.
/// - Otherwise: returns a score whose measures are the expanded playback
///   sequence, with each occurrence's `(-abbrev ...)` part omissions applied.
pub fn expand_navigation(score: &Score) -> Result<Score, IrrecoverableError> {
    expand_navigation_with_origins(score).map(|(score, _)| score)
}

/// Same as [`expand_navigation`], but also returns a same-length `Vec<usize>`
/// mapping each measure in the expanded score back to its index in the
/// original written `score.measures`.
pub fn expand_navigation_with_origins(
    score: &Score,
) -> Result<(Score, Vec<usize>), IrrecoverableError> {
    let (expanded, origins) = expand_navigation_with_note_positions(score)?;
    Ok((
        expanded,
        origins
            .into_iter()
            .map(|o| o.written_measure_index)
            .collect(),
    ))
}

/// One measure of the expanded (playback-order) score, tracing back to where
/// it came from in the written score.
pub struct ExpandedMeasureOrigin {
    /// Index into the original written `score.measures`.
    pub written_measure_index: usize,
    /// `part_written_indices[i]` is the index this expanded measure's
    /// `parts[i]` had in the written measure's `parts`, before any
    /// `(-abbrev ...)` omission dropped some entries and shifted the rest —
    /// so callers keying data by the written `(part_index, note_id)` (e.g.
    /// `ColumnElement::note_id`) can still look it up after expansion.
    pub part_written_indices: Vec<usize>,
}

/// Same as [`expand_navigation_with_origins`], but the origins additionally
/// carry, per expanded measure, the written index of each surviving part —
/// needed to key note-level playback timing (see
/// [`super::timing::note_timings_seconds`]) back to the written
/// `(source_part_index, note_id)` identity after `(-abbrev ...)` omissions
/// have shifted part positions.
pub fn expand_navigation_with_note_positions(
    score: &Score,
) -> Result<(Score, Vec<ExpandedMeasureOrigin>), IrrecoverableError> {
    if score.measures.is_empty() {
        return Ok((score.clone(), Vec::new()));
    }

    if let Some(spans) = &score.sequence {
        let idx: Vec<(usize, &[String])> = spans
            .iter()
            .flat_map(|span| (span.start..=span.end).map(move |i| (i, span.omit_parts.as_slice())))
            .collect();
        return Ok(build_expanded(score, &idx));
    }

    let origins = score
        .measures
        .iter()
        .enumerate()
        .map(|(i, measure)| ExpandedMeasureOrigin {
            written_measure_index: i,
            part_written_indices: (0..measure.parts.len()).collect(),
        })
        .collect();
    Ok((score.clone(), origins))
}

/// For every written measure index, the BPM/key that was already in effect
/// immediately *before* that measure (i.e. accumulated from every earlier
/// written measure's own `bpm=`/`key=`, not including this measure's own).
/// `None` until the first measure that sets one.
///
/// [`build_expanded`] uses this to backfill a played measure's missing
/// `bpm`/`key` when `sequence` navigation skips over the written measures
/// between it and the previous played one (e.g. an intro before the
/// sequence's first listed section, or a gap between two labeled sections):
/// without a written measure actually being processed there, nothing would
/// otherwise carry an earlier `bpm=`/`key=` change forward into what does
/// get played. Mirrors [`super::timing_range::build_measure_range_score`]'s
/// same accumulate-then-backfill treatment for a measure-range selection.
fn accumulated_context_before(score: &Score) -> (Vec<Option<u32>>, Vec<Option<KeyChange>>) {
    let mut bpm_before = Vec::with_capacity(score.measures.len());
    let mut key_before = Vec::with_capacity(score.measures.len());
    let mut current_bpm = None;
    let mut current_key: Option<KeyChange> = None;
    for measure in &score.measures {
        bpm_before.push(current_bpm);
        key_before.push(current_key.clone());
        if let Some(bpm) = measure.bpm {
            current_bpm = Some(bpm);
        }
        if let Some(key) = &measure.key {
            current_key = Some(key.clone());
        }
    }
    (bpm_before, key_before)
}

/// Clones `score.measures` at each `(index, omit_parts)` pair (playback
/// order) into a new `Score`, dropping any part whose abbreviation appears
/// in that occurrence's `omit_parts`, alongside origin info for each
/// resulting measure (its written index, and the written index of each
/// surviving part).
fn build_expanded(
    score: &Score,
    idx: &[(usize, &[String])],
) -> (Score, Vec<ExpandedMeasureOrigin>) {
    let (bpm_before, key_before) = accumulated_context_before(score);
    let mut origins = Vec::with_capacity(idx.len());
    let measures = idx
        .iter()
        .filter_map(|&(i, omit_parts)| {
            let mut measure = score.measures.get(i).cloned()?;
            if measure.bpm.is_none() {
                measure.bpm = bpm_before.get(i).copied().flatten();
            }
            if measure.key.is_none() {
                measure.key = key_before.get(i).cloned().flatten();
            }
            let mut part_written_indices = Vec::with_capacity(measure.parts.len());
            let mut written_idx = 0usize;
            measure.parts.retain(|part| {
                let omitted = !omit_parts.is_empty()
                    && part.name().is_some_and(|name| omit_parts.contains(name));
                if !omitted {
                    part_written_indices.push(written_idx);
                }
                written_idx += 1;
                !omitted
            });
            origins.push(ExpandedMeasureOrigin {
                written_measure_index: i,
                part_written_indices,
            });
            Some(measure)
        })
        .collect();

    (
        Score {
            metadata: score.metadata.clone(),
            measures,
            document_diagnostics: score.document_diagnostics.clone(),
            sequence: None,
        },
        origins,
    )
}

/// Filters `expanded.measures`/`origins` in lockstep to keep only parts whose
/// name is in `enabled_tracks`, preserving each surviving part's true
/// written index in `origins[i].part_written_indices` — the counterpart to
/// [`crate::filters::apply_track_filter`]'s physical removal, but applied
/// *after* [`expand_navigation_with_note_positions`] so track-filtered
/// playback timing (`note_timings_seconds`) still reports the original
/// written `source_part_index` rather than a part's position among the
/// surviving parts. Mirrors [`build_expanded`]'s `omit_parts` retain, but
/// keyed by inclusion (`enabled_tracks`) instead of exclusion, and applied
/// uniformly to every measure rather than per-`sequence`-entry.
///
/// `None` keeps every part (no-op). Every `origin.part_written_indices` must
/// already be the same length as its measure's `parts` (true for both
/// [`expand_navigation_with_note_positions`]'s branches).
pub(super) fn filter_expanded_tracks(
    expanded: &mut Score,
    origins: &mut [ExpandedMeasureOrigin],
    enabled_tracks: Option<&[String]>,
) {
    let Some(tracks) = enabled_tracks else {
        return;
    };
    for (measure, origin) in expanded.measures.iter_mut().zip(origins.iter_mut()) {
        let mut kept_parts = Vec::with_capacity(measure.parts.len());
        let mut kept_indices = Vec::with_capacity(measure.parts.len());
        for (part, &written_index) in measure
            .parts
            .drain(..)
            .zip(origin.part_written_indices.iter())
        {
            if part.name().is_some_and(|name| tracks.contains(name)) {
                kept_indices.push(written_index);
                kept_parts.push(part);
            }
        }
        measure.parts = kept_parts;
        origin.part_written_indices = kept_indices;
    }
}

mod range;
pub use range::{earliest_playback_position, expand_for_measure_range};

#[cfg(test)]
mod tests_sequence_omission;
