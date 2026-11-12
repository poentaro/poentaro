use super::{
    align_empty_note_measures, pair_lyrics_measures, GroupedPart, IrrecoverableError,
    ParsedMeasureSlot, ParsedTimedTrack, PartGrouper, PerMeasureErrors,
};
use crate::ast::grouped::GroupedTrack;
use crate::ast::parsed::PartKind;
use crate::grouper::lyrics_pairing::PairedMeasure;
use crate::lyric_paired_target::{enters_tied, paired_target_index, target_part};
use crate::tuplet::apply_resolution_multiplier;

/// `global_resolution_multipliers[i]` is the tuplet-rescale multiplier every part must
/// use for measure index `i`, already accounting for tuplets in *any* part at that
/// measure (see `compute_global_resolution_multipliers` in `grouper::mod`) — so that
/// sibling parts sharing a measure stay on the same rescaled grid and their notes line
/// up column-for-column at matching beats.
pub(in crate::grouper) fn group_timed_track(
    part: ParsedTimedTrack,
    global_resolution_multipliers: &[u32],
) -> Result<GroupedPart, IrrecoverableError> {
    let per_measure_beat_errors = part.per_measure_beat_errors.clone();
    let per_measure_dotted_eighth_errors = part.per_measure_dotted_eighth_errors.clone();
    let per_measure_chord_errors = part.per_measure_chord_errors.clone();
    let per_measure_lex_errors = part.per_measure_lex_errors.clone();
    let per_measure_lyrics_errors = part.per_measure_lyrics_errors.clone();
    let part_volume = part.volume;
    let part_octave_offset = part.octave_offset;
    let part_range_instrument = part.range_instrument;
    let part_singing_voice = part.singing_voice;
    let mut grouper = PartGrouper::new(&part);
    for (slot_index, slot) in part.measure_slots.into_iter().enumerate() {
        match slot {
            ParsedMeasureSlot::EmptyNote { span } => grouper.push_empty_note_slot(span),
            ParsedMeasureSlot::Real { events } => {
                let resolution_multiplier = global_resolution_multipliers
                    .get(slot_index)
                    .copied()
                    .unwrap_or(1);
                let events = apply_resolution_multiplier(events, resolution_multiplier);
                grouper.begin_measure_slot(resolution_multiplier);
                for spanned in events {
                    grouper.process_event(spanned)?;
                }
                // `validate_and_pad_beats` (parse time) already guarantees this slot's
                // events sum to exactly one measure's nominal capacity, but tuplet
                // rescaling can make the *actual* (rescaled) total miss the rescaled
                // capacity by a beat or two — see the **Tuplet** glossary entry in
                // ARCHITECTURE.md. `process_event` only flushes on an exact match, so
                // without this, such a measure would never close and its notes would
                // bleed into the next measure slot. Force the boundary here instead.
                grouper.flush_measure();
            }
        }
    }
    let (slots, name, kind, soundfont) = grouper.finish();
    let measures = align_empty_note_measures(
        slots,
        &PerMeasureErrors {
            beat_errors: &per_measure_beat_errors,
            dotted_eighth_errors: &per_measure_dotted_eighth_errors,
            chord_errors: &per_measure_chord_errors,
            lex_errors: &per_measure_lex_errors,
            lyrics_errors: &per_measure_lyrics_errors,
        },
    )?;
    Ok(GroupedPart {
        name,
        kind,
        soundfont,
        volume: part_volume,
        octave_offset: part_octave_offset,
        range_instrument: part_range_instrument,
        singing_voice: part_singing_voice,
        lyric_targets: Vec::new(),
        measures,
    })
}

/// Groups a lyric part: it has no notes of its own, so each of its measures
/// is the lyric line's syllables paired to the notes of the target it pairs
/// with there (see `pair_lyrics_measures`, `lyric_paired_target`). Its
/// targets are declared before it, so `grouped_tracks` holds them already.
/// `None` when the bottommost target is missing.
pub(in crate::grouper) fn group_lyrics_track(
    part: ParsedTimedTrack,
    grouped_tracks: &[GroupedTrack],
    target_part_index: usize,
) -> Option<GroupedPart> {
    let measure_count = target_part(grouped_tracks, target_part_index)?
        .measures
        .len();
    let paired_measures = (0..measure_count)
        .filter_map(|measure_index| {
            let paired = target_part(
                grouped_tracks,
                paired_target_index(
                    grouped_tracks,
                    &part.lyric_targets,
                    target_part_index,
                    measure_index,
                ),
            )?;
            Some(PairedMeasure {
                measure: paired.measures.get(measure_index)?,
                entering_tied: enters_tied(paired, measure_index),
                chords: paired.kind == PartKind::Chords,
            })
        })
        .collect::<Vec<_>>();
    let measures = match part.lyrics {
        Some(lyrics) => pair_lyrics_measures(
            &paired_measures,
            lyrics,
            &part.per_measure_lyrics_errors,
            &part.abbreviation,
        ),
        None => Vec::new(),
    };
    Some(GroupedPart {
        name: Some(part.abbreviation),
        kind: part.kind,
        soundfont: part.soundfont,
        volume: part.volume,
        octave_offset: part.octave_offset,
        range_instrument: part.range_instrument,
        singing_voice: None,
        lyric_targets: part.lyric_targets,
        measures,
    })
}
