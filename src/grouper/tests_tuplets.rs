use super::*;

/// `3:{1_1_1_}` — an eighth-note triplet (3 eighth notes, 2 quarter-beats each as
/// written) tagged `TupletInfo { num: 3, den: 2 }` — followed by six more plain eighth
/// notes, groups into one measure with no diagnostics, exercising
/// `crate::tuplet::apply_resolution_multiplier` end to end through `PartGrouper`'s
/// multiplier-scaled capacity check (`PartGrouper::effective_capacity`).
///
/// The triplet rescales to exactly one beat (its 3 eighth notes fill the time normally
/// taken by 2), so the remaining 6 plain eighth notes (3 beats' worth) are needed to
/// exactly fill the rest of the 4/4 bar — the parser's own capacity check
/// (`interleaved_beat_padding::validate_and_pad_beats`, tuplet-aware — see
/// `eighth_note_triplet_fills_exactly_one_beat` in `tuplet_tests.rs` for the rescale math
/// tested directly) confirms this measure is exactly full before it ever reaches the
/// grouper.
#[test]
fn measure_with_a_triplet_groups_with_correctly_rescaled_durations() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 3:{1_1_1_} 2_ 3_ 4_ 5_ 6_ 7_
    """,
"#,
    );
    assert_eq!(score.measures.len(), 1);
    assert!(
        score.measures[0].diagnostics.is_empty(),
        "expected no diagnostics, got: {:?}",
        score.measures[0]
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
    let events = first_part_notes(&score, 0);
    assert_eq!(events.len(), 9, "3 triplet notes + 6 plain eighth notes");

    // resolution_multiplier = lcm(3) = 3. Triplet notes (written duration 2) rescale to
    // 2 * 3 * 2 / 3 = 4 each; plain eighth notes (written duration 2) rescale to 2 * 3 =
    // 6 each. Together the whole rescaled measure sums to exactly `16 * 3` (the 4/4 bar's
    // capacity scaled by the same multiplier), landing exactly on the flush boundary.
    let durations: Vec<u32> = events
        .iter()
        .map(|event| match event {
            NoteEvent::Note(n) => n.duration,
            NoteEvent::Rest(_) | NoteEvent::Chord(_) | NoteEvent::Percussion(_) => {
                panic!("expected only Note events")
            }
        })
        .collect();
    assert_eq!(durations, vec![4, 4, 4, 6, 6, 6, 6, 6, 6]);
    assert_eq!(durations.iter().sum::<u32>(), 16 * 3);
}
