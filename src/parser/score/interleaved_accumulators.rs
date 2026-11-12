use super::errors::invariant;
use super::{SlotAction, TrackAccumulator};
use crate::ast::parsed::{
    ParsedLyrics, ParsedTimedTrack, ParsedTrack, PartDecl, PartKind, ScoreLineRole, ScoreLineSlot,
};
use crate::error::{IrrecoverableError, Span};
use crate::parser::score::token_parser::GroupStack;

pub(super) fn build_slot_actions(slots: &[ScoreLineSlot]) -> Vec<SlotAction> {
    slots
        .iter()
        .map(|slot| match slot.role {
            ScoreLineRole::Chord => SlotAction::Chord {
                track_index: slot.track_index,
            },
            ScoreLineRole::Notes => SlotAction::Notes {
                track_index: slot.track_index,
            },
            ScoreLineRole::Lyrics => SlotAction::Lyrics {
                track_index: slot.track_index,
            },
        })
        .collect()
}

pub(super) fn init_accumulators(declarations: &[PartDecl]) -> Vec<TrackAccumulator> {
    declarations
        .iter()
        .map(|decl| TrackAccumulator::Timed {
            measure_slots: Vec::new(),
            pending_events: Vec::new(),
            syllables: if matches!(decl.kind, PartKind::Lyrics { .. }) {
                Some(Vec::new())
            } else {
                None
            },
            lyrics_line_starts: Vec::new(),
            lyrics_line_ends: Vec::new(),
            per_measure_beat_errors: Vec::new(),
            per_measure_dotted_eighth_errors: Vec::new(),
            per_measure_chord_errors: Vec::new(),
            per_measure_lex_errors: Vec::new(),
            per_measure_lyrics_errors: Vec::new(),
        })
        .collect()
}

pub(super) fn build_parse_result(
    declarations: &[PartDecl],
    accumulators: Vec<TrackAccumulator>,
    group_states: Vec<GroupStack>,
) -> Result<Vec<ParsedTrack>, IrrecoverableError> {
    if declarations.len() != accumulators.len() || declarations.len() != group_states.len() {
        return Err(invariant(
            Span::new(0, 0),
            "internal error: declaration/accumulator count mismatch",
        ));
    }

    declarations
        .iter()
        .zip(accumulators)
        .zip(group_states)
        .map(|((decl, acc), group_state)| {
            let TrackAccumulator::Timed {
                measure_slots,
                syllables,
                lyrics_line_starts,
                lyrics_line_ends,
                per_measure_beat_errors,
                per_measure_dotted_eighth_errors,
                per_measure_chord_errors,
                per_measure_lex_errors,
                per_measure_lyrics_errors,
                ..
            } = acc;
            Ok(ParsedTrack::Timed(ParsedTimedTrack {
                abbreviation: decl.abbreviation.clone(),
                display_name: decl.display_name.clone(),
                kind: decl.kind,
                soundfont: decl.soundfont,
                volume: decl.volume,
                octave_offset: decl.octave_offset,
                range_instrument: decl.range_instrument,
                singing_voice: decl.singing_voice,
                lyric_targets: decl.lyric_targets.clone(),
                measure_slots,
                lyrics: syllables.map(|measure_syllables| ParsedLyrics {
                    measure_syllables,
                    measure_starts: lyrics_line_starts,
                    measure_ends: lyrics_line_ends,
                }),
                per_measure_beat_errors,
                per_measure_dotted_eighth_errors,
                per_measure_chord_errors,
                per_measure_lex_errors,
                per_measure_lyrics_errors,
                slur_groups: group_state.closed_groups,
            }))
        })
        .collect()
}
