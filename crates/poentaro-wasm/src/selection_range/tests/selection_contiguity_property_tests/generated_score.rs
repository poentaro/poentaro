//! Random scores for the contiguity property test: several `notes` parts in
//! 4/4 with mixed durations, rests and ties, and lyric parts of CJK and
//! Latin syllables.

use itertools::Itertools;
use proptest::prelude::*;
use proptest::sample::Index;

/// Quarter-beats in one 4/4 measure (see `syntax.md`'s "Duration suffixes").
pub(super) const MEASURE_LENGTH: usize = 16;

pub(super) const NOTES_PART_NAMES: [&str; 4] = ["A", "B", "C", "D"];

/// CJK characters are one syllable each; Latin syllables are space-separated,
/// with a trailing `-` for a word split across notes. A standalone `-` holds
/// the previous syllable (see `syntax.md`'s "Lyrics syntax").
pub(super) const SYLLABLES: [&str; 9] = ["你", "好", "愛", "拼", "la", "twin-", "kle", "do", "-"];
pub(super) const HELD_SYLLABLE: &str = "-";

#[derive(Debug, Clone)]
pub(super) struct GeneratedEvent {
    /// `None` for a rest.
    pub(super) pitch: Option<u8>,
    /// In quarter-beats.
    pub(super) duration: usize,
    pub(super) tied_to_next: bool,
}

#[derive(Debug, Clone)]
pub(super) struct GeneratedMeasure {
    pub(super) events: Vec<GeneratedEvent>,
}

#[derive(Debug, Clone)]
pub(super) struct GeneratedNotesPart {
    pub(super) measures: Vec<GeneratedMeasure>,
}

#[derive(Debug, Clone)]
pub(super) struct GeneratedLyricPart {
    /// Index into `GeneratedScore::notes_parts`.
    pub(super) target: usize,
    /// Declared after every notes part instead of right after its target,
    /// so the parser has to move its row back under the target.
    pub(super) declared_last: bool,
    /// Picks from `SYLLABLES`, consumed one per syllable in score order.
    pub(super) syllable_picks: Vec<Index>,
}

#[derive(Debug, Clone)]
pub(super) struct GeneratedScore {
    pub(super) notes_parts: Vec<GeneratedNotesPart>,
    pub(super) lyric_parts: Vec<GeneratedLyricPart>,
    pub(super) merge_duplicate_measures: bool,
    pub(super) hide_resting_parts: bool,
    /// Writes each measure's lines bottom-up, which the score allows.
    pub(super) reverse_lines: bool,
}

/// When something sounds, in quarter-beats from the start of the piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Timing {
    pub(super) onset: usize,
    pub(super) end: usize,
}

impl Timing {
    pub(super) fn overlaps(&self, other: &Timing) -> bool {
        self.onset < other.end && other.onset < self.end
    }
}

/// Durations filling exactly one measure: picks from `choices` in order,
/// skipping any that would overflow, then pads the rest with quarters (and a
/// final eighth if needed).
fn fill_measure(choices: &[usize]) -> Vec<usize> {
    let mut durations: Vec<usize> = Vec::new();
    for &duration in choices {
        if durations.iter().sum::<usize>() + duration <= MEASURE_LENGTH {
            durations.push(duration);
        }
    }
    let remaining = MEASURE_LENGTH - durations.iter().sum::<usize>();
    durations.extend(std::iter::repeat_n(4, remaining / 4));
    if remaining % 4 != 0 {
        durations.push(remaining % 4);
    }
    durations
}

fn measure_strategy() -> impl Strategy<Value = GeneratedMeasure> {
    (
        prop::collection::vec(prop::sample::select(vec![2, 4, 8, 12, 16]), 1..=8),
        prop::collection::vec((0u8..=7, prop::bool::weighted(0.2)), MEASURE_LENGTH),
    )
        .prop_map(|(choices, pitches_and_ties)| GeneratedMeasure {
            events: fill_measure(&choices)
                .into_iter()
                .zip(pitches_and_ties)
                .map(|(duration, (pitch, tied_to_next))| GeneratedEvent {
                    pitch: (pitch != 0).then_some(pitch),
                    duration,
                    tied_to_next,
                })
                .collect(),
        })
}

/// Keeps only the ties the syntax allows — a note tied to a following note —
/// and gives that following note the same pitch, as a tie requires.
fn fix_up_ties(part: GeneratedNotesPart) -> GeneratedNotesPart {
    let measure_lengths = part
        .measures
        .iter()
        .map(|measure| measure.events.len())
        .collect_vec();
    let mut events = part
        .measures
        .into_iter()
        .flat_map(|measure| measure.events)
        .collect_vec();
    for index in 0..events.len() {
        let next_pitch = events.get(index + 1).and_then(|next| next.pitch);
        let event = &mut events[index];
        event.tied_to_next = event.tied_to_next && event.pitch.is_some() && next_pitch.is_some();
        if event.tied_to_next {
            let pitch = event.pitch;
            events[index + 1].pitch = pitch;
        }
    }
    let mut events = events.into_iter();
    GeneratedNotesPart {
        measures: measure_lengths
            .into_iter()
            .map(|length| GeneratedMeasure {
                events: events.by_ref().take(length).collect(),
            })
            .collect(),
    }
}

fn lyric_part_strategy(notes_part_count: usize) -> impl Strategy<Value = GeneratedLyricPart> {
    (
        0..notes_part_count,
        any::<bool>(),
        prop::collection::vec(any::<Index>(), 3 * MEASURE_LENGTH),
    )
        .prop_map(
            |(target, declared_last, syllable_picks)| GeneratedLyricPart {
                target,
                declared_last,
                syllable_picks,
            },
        )
}

pub(super) fn score_strategy() -> impl Strategy<Value = GeneratedScore> {
    (1usize..=NOTES_PART_NAMES.len(), 1usize..=3).prop_flat_map(
        |(notes_part_count, measure_count)| {
            (
                prop::collection::vec(
                    prop::collection::vec(measure_strategy(), measure_count)
                        .prop_map(|measures| fix_up_ties(GeneratedNotesPart { measures })),
                    notes_part_count,
                ),
                prop::collection::vec(lyric_part_strategy(notes_part_count), 0..=3),
                any::<bool>(),
                any::<bool>(),
                any::<bool>(),
            )
                .prop_map(
                    |(
                        notes_parts,
                        lyric_parts,
                        merge_duplicate_measures,
                        hide_resting_parts,
                        reverse_lines,
                    )| GeneratedScore {
                        notes_parts,
                        lyric_parts,
                        merge_duplicate_measures,
                        hide_resting_parts,
                        reverse_lines,
                    },
                )
        },
    )
}
