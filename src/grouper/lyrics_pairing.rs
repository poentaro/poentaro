use crate::ast::grouped::{GroupedMeasure, NoteEvent, Notes};
use crate::ast::parsed::{ParsedLyrics, Syllable};
use crate::chord_lyric_placement::chord_lyric_tally_error;
use crate::error::{RecoverableError, Span, Warning};
use crate::lyric_slots::{chord_attacks, measure_lyric_slots};

/// The target measure one lyric measure pairs with: its paired part's (see
/// `lyric_paired_target`), and whether that part's previous measure ties into it.
pub(in crate::grouper) struct PairedMeasure<'a> {
    pub(in crate::grouper) measure: &'a GroupedMeasure,
    pub(in crate::grouper) entering_tied: bool,
    /// The paired part is a `chords` part.
    pub(in crate::grouper) chords: bool,
}

/// Builds a lyric part's grouped measures by pairing each measure's raw
/// syllables to the lyric slots of its paired target measure's notes
/// (tie-aware). Underflow is recovered by padding empty syllables and
/// recording a warning. Tie state depends only on the paired part's own
/// notes, never on the lyrics.
pub(super) fn pair_lyrics_measures(
    paired_measures: &[PairedMeasure],
    lyrics: ParsedLyrics,
    lyric_errors: &[Option<RecoverableError>],
    part_name: &str,
) -> Vec<GroupedMeasure> {
    let lyric_line_count = lyrics.measure_syllables.len();
    lyrics
        .measure_syllables
        .into_iter()
        .zip(lyrics.measure_starts.into_iter().zip(lyrics.measure_ends))
        .enumerate()
        .map(|(index, (raw_syllables, (start, end)))| {
            let lyrics_span = Span::new(start, end);
            let paired = paired_measures.get(index);
            let target = paired.map(|paired| paired.measure);
            let events = target.map_or(&[][..], |measure| measure.notes.events.as_slice());
            let line_error = lyric_errors.get(index).cloned().flatten();
            let (paired, warning, tally_error) = if paired.is_some_and(|paired| paired.chords) {
                // Chords hold no ties, and a line with a bracket mistake is
                // already an error: tallying it would only repeat the problem.
                let tally_error = if line_error.is_some() {
                    None
                } else {
                    tally_chord_lyrics(events, &raw_syllables, lyrics_span)
                };
                (raw_syllables, None, tally_error)
            } else {
                let (paired_syllables, warning) = pair_lyrics_to_notes(
                    events,
                    &raw_syllables,
                    &lyrics_span,
                    paired.is_some_and(|paired| paired.entering_tied),
                    part_name,
                );
                (paired_syllables, warning, None)
            };
            let count_mismatch = target.is_none().then(|| {
                Warning::new(
                    Span::new(0, 0),
                    format!(
                        "[{part_name}] internal invariant: {lyric_line_count} lyric lines but {} target measures",
                        paired_measures.len()
                    ),
                )
            });
            GroupedMeasure {
                notes: Notes { events: Vec::new() },
                source_span: lyrics_span,
                paired_lyrics: paired,
                lyrics_error: warning.into_iter().chain(count_mismatch).collect(),
                beat_overflow_error: None,
                dotted_eighth_errors: Vec::new(),
                chord_errors: Vec::new(),
                lex_error: None,
                lyrics_parse_error: line_error.or(tally_error),
                extension_no_preceding_event_error: None,
                resolution_multiplier: target.map_or(1, |measure| measure.resolution_multiplier),
                beat_group_size: target.map_or(4, |measure| measure.beat_group_size),
            }
        })
        .collect()
}

fn pair_lyrics_to_notes(
    events: &[NoteEvent],
    raw_syllables: &[Syllable],
    source_span: &Span,
    entering_tied: bool,
    part_name: &str,
) -> (Vec<Syllable>, Option<Warning>) {
    let no_lyrics = raw_syllables.is_empty();
    let lyric_slots = measure_lyric_slots(events, entering_tied);
    let mut syllable_idx = 0;
    let mut paired = Vec::new();
    let mut underflow_detected = false;

    for _ in &lyric_slots.slots {
        if let Some(syllable) = raw_syllables.get(syllable_idx) {
            paired.push(syllable.clone());
            syllable_idx += 1;
        } else if !no_lyrics {
            paired.push(Syllable {
                text: String::new(),
                held: false,
                bracketed: false,
                span: Span::new(source_span.start, source_span.start),
            });
            underflow_detected = true;
        }
    }
    let overflow_count = raw_syllables.len().saturating_sub(syllable_idx);
    let error = if underflow_detected {
        Some(Warning::new(
            *source_span,
            format!(
                "[{part_name}] lyrics underflow: ran out of syllables at syllable {syllable_idx} (fewer syllables than notes)"
            ),
        ))
    } else if overflow_count > 0 {
        Some(Warning::new(
            *source_span,
            format!(
                "[{part_name}] lyrics overflow: {overflow_count} extra syllable{} after all notes are consumed",
                if overflow_count == 1 { "" } else { "s" }
            ),
        ))
    } else {
        None
    };

    (paired, error)
}

/// Under a chord part the line's syllables are kept whole (bracketed and
/// unbracketed, in order); this only validates them (see
/// `chord_lyric_placement::chord_lyric_tally_error`).
fn tally_chord_lyrics(
    events: &[NoteEvent],
    syllables: &[Syllable],
    source_span: Span,
) -> Option<RecoverableError> {
    let attacks = chord_attacks(events);
    chord_lyric_tally_error(
        syllables,
        attacks.len(),
        attacks.first().is_some_and(|attack| attack.column == 0),
        source_span,
    )
}
