#![allow(clippy::disallowed_macros)]
use poentaro::{note_timings_from_source, render_svgs_from_source};
use std::collections::HashSet;

/// Two parts (one with a cross-barline tie and a chord part), plus a rest,
/// so the compiler's `note_id` stamping and `note_timings_seconds`'s walk
/// both have ties, rests, and chords to agree on.
pub(crate) fn multi_part_source_with_ties_rests_and_chord() -> &'static str {
    r#"metadata { title "note highlight identity" },

parts ["Soprano" [S]: notes, "Alto" [A]: notes, "Chords" [C]: chords],

score """
    time=4/4 key=C4 bpm=120
    [S] 1~ 1 0 2
    [A] 3 3 3 3
    [C] 1 - - -

    [S] 3 3~ 3 0
    [A] 5 5 5 5
    [C] 0 0 4 -
    """,
"#
}

/// Extracts every `(source_part_index, note_id)` pair rendered as a
/// `data-tag="note"` group in the SVG output — the identity the grid-layout
/// highlight-target pass assigns to each note/rest's on-screen position(s).
pub(crate) fn note_ids_in_svg(svgs: &[String]) -> HashSet<(usize, usize)> {
    let mut ids = HashSet::new();
    for svg in svgs {
        let mut rest = svg.as_str();
        while let Some(tag_start) = rest.find(r#"data-tag="note""#) {
            rest = &rest[tag_start..];
            if let (Some(part_index), Some(note_id)) = (
                extract_usize_attr(rest, "data-part-index"),
                extract_usize_attr(rest, "data-note-id"),
            ) {
                ids.insert((part_index, note_id));
            }
            rest = &rest[1..];
        }
    }
    ids
}

fn extract_usize_attr(s: &str, attr: &str) -> Option<usize> {
    let needle = format!(r#"{attr}="#);
    let start = s.find(&needle)? + needle.len() + 1;
    let end = start + s[start..].find('"')?;
    s[start..end].parse().ok()
}

#[test]
fn note_ids_match_between_grid_layout_and_midi_timing() {
    let source = multi_part_source_with_ties_rests_and_chord();

    let render_output =
        render_svgs_from_source(source, "note_highlight_identity.poentaro", &[]).unwrap();
    let svg_ids = note_ids_in_svg(&render_output.svgs);

    let note_timings =
        note_timings_from_source(source, "note_highlight_identity.poentaro", None, None, &[])
            .unwrap();
    let timing_ids: HashSet<(usize, usize)> = note_timings
        .iter()
        .map(|t| (t.source_part_index, t.note_id))
        .collect();

    assert!(
        !svg_ids.is_empty(),
        "expected at least one note group in the rendered SVG"
    );
    assert_eq!(
        svg_ids, timing_ids,
        "grid-layout note highlight targets and MIDI note timings must agree on every \
         (source_part_index, note_id) pair"
    );
}

/// A `sequence` that replays labeled span `A` twice before playing `B`
/// once, so playback order (`A`, `A`, `B`) differs from written order
/// (`A`, `B`).
fn sequence_source_repeating_a_span() -> &'static str {
    r#"metadata { title "note highlight identity sequence" },

parts ["Soprano" [S]: notes],

sequence ["A", "A", "B"],

score """
    time=4/4 key=C4 bpm=120 label="A"
    [S] 1 2 3 4

    label="B"
    [S] 5 6 7 1
    """,
"#
}

#[test]
fn note_ids_match_between_grid_layout_and_midi_timing_with_sequence() {
    let source = sequence_source_repeating_a_span();

    let render_output =
        render_svgs_from_source(source, "note_highlight_identity_sequence.poentaro", &[]).unwrap();
    let svg_ids = note_ids_in_svg(&render_output.svgs);

    let note_timings = note_timings_from_source(
        source,
        "note_highlight_identity_sequence.poentaro",
        None,
        None,
        &[],
    )
    .unwrap();
    let timing_ids: HashSet<(usize, usize)> = note_timings
        .iter()
        .map(|t| (t.source_part_index, t.note_id))
        .collect();

    assert!(
        !svg_ids.is_empty(),
        "expected at least one note group in the rendered SVG"
    );
    assert_eq!(
        svg_ids, timing_ids,
        "grid-layout note highlight targets and MIDI note timings must agree on every \
         (source_part_index, note_id) pair, even when `sequence` replays a span"
    );

    // Span `A` (written measure 0, 4 notes) plays twice, `B` (written
    // measure 1, 4 notes) plays once: 12 sounding occurrences total, even
    // though only 8 distinct (source_part_index, note_id) pairs exist.
    assert_eq!(
        note_timings.len(),
        12,
        "expected one NoteTiming per playback occurrence, not per written note"
    );

    // The 4 notes of span `A` should each appear as two occurrences with
    // distinct, non-overlapping start times (the first pass then the
    // second), proving timing follows playback order rather than being
    // computed once from written order.
    for note_id in 0..4 {
        let mut starts: Vec<f64> = note_timings
            .iter()
            .filter(|t| t.source_part_index == 0 && t.note_id == note_id)
            .map(|t| t.start_s)
            .collect();
        starts.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            starts.len(),
            2,
            "note_id {note_id} in span A should sound twice"
        );
        assert!(
            starts[1] > starts[0],
            "second occurrence of note_id {note_id} should start after the first"
        );
    }
}
