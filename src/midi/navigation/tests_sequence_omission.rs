use super::{expand_for_measure_range, expand_navigation_with_origins};
use crate::ast::grouped::{
    Metadata, MultiPartMeasure, Notes, PartRow, PartSlice, Score, SequenceSpan,
};
use crate::ast::parsed::{Offset, PartKind, Soundfont};
use crate::error::Span;

fn text_style(
    font_size: u32,
    horizontal_padding_pt: u32,
    vertical_padding_pt: u32,
) -> crate::ast::grouped::TextStyle {
    crate::ast::grouped::TextStyle {
        font_size,
        horizontal_padding_pt,
        vertical_padding_pt,
        ..Default::default()
    }
}

fn metadata() -> Metadata {
    Metadata {
        title: None,
        subtitle: None,
        author: None,
        row_height: 24,
        max_measures_per_system: 28,
        note_number_width: 8,
        parts_list_columns: 3,
        part_label_width_pt: 40,
        page_margin_pt: 25,
        lyrics: text_style(14, 4, 12),
        notes: text_style(14, 4, 0),
        chords: text_style(14, 4, 0),
        note_dash: text_style(14, 4, 0),
        title_style: text_style(36, 0, 0),
        subtitle_style: text_style(19, 0, 0),
        author_style: text_style(14, 0, 0),
        sequence: text_style(12, 0, 0),
        part_legend: text_style(12, 0, 0),
        measure_number: text_style(10, 0, 0),
        section_label: text_style(12, 0, 0),
        part_label: text_style(12, 0, 0),
        page_number: text_style(14, 0, 0),
        running_title: text_style(14, 0, 0),
        merge_duplicate_measures_across_parts: true,
        hide_resting_parts: true,
        hide_system_dividers: false,
        directive_row_offset: Offset::default(),
    }
}

fn bare_measure(index: usize) -> MultiPartMeasure {
    MultiPartMeasure {
        time_signature: None,
        bpm: None,
        key: None,
        label: None,
        merge_duplicate_measures_across_parts: true,
        hide_resting_parts: true,
        system_break: false,
        parts: vec![],
        source_span: Span::new(index, index + 1),
        diagnostics: vec![],
    }
}

fn score_with_sequence(measures: Vec<MultiPartMeasure>, sequence: Vec<SequenceSpan>) -> Score {
    Score {
        metadata: metadata(),
        measures,
        document_diagnostics: vec![],
        sequence: Some(sequence),
    }
}

fn measure_with_parts(index: usize, part_names: &[&str]) -> MultiPartMeasure {
    MultiPartMeasure {
        parts: part_names
            .iter()
            .map(|name| {
                PartRow::Timed(PartSlice {
                    name: Some(name.to_string()),
                    kind: PartKind::Notes,
                    soundfont: Soundfont::CHOIR_AAHS,
                    volume: 100,
                    octave_offset: 0,
                    range_instrument: None,
                    notes: Notes { events: vec![] },
                    lyrics: None,
                    has_error: false,
                    resolution_multiplier: 1,
                    beat_group_size: 4,
                })
            })
            .collect(),
        ..bare_measure(index)
    }
}

#[test]
fn sequence_omit_parts_drops_named_parts_per_occurrence() {
    // A single measure "Chorus" with three parts (S, A2, T). Replaying it
    // twice with different `(-abbrev ...)` omissions should drop only the
    // named parts on each occurrence, leaving the written measure untouched.
    let measures = vec![measure_with_parts(0, &["S", "A2", "T"])];
    let score = score_with_sequence(
        measures,
        vec![
            SequenceSpan {
                label: "Chorus".to_string(),
                start: 0,
                end: 0,
                omit_parts: vec!["S".to_string(), "A2".to_string()],
                part_filter_display: None,
            },
            SequenceSpan {
                label: "Chorus".to_string(),
                start: 0,
                end: 0,
                omit_parts: vec!["A2".to_string()],
                part_filter_display: None,
            },
        ],
    );
    let (expanded, _) = expand_navigation_with_origins(&score).unwrap();
    assert_eq!(expanded.measures.len(), 2);
    fn names(measure: &MultiPartMeasure) -> Vec<&str> {
        measure
            .parts
            .iter()
            .filter_map(|p| p.name().map(String::as_str))
            .collect()
    }
    assert_eq!(names(&expanded.measures[0]), vec!["T"]);
    assert_eq!(names(&expanded.measures[1]), vec!["S", "T"]);
    // The original written score is untouched.
    assert_eq!(score.measures[0].parts.len(), 3);
}

#[test]
fn expand_for_measure_range_respect_sequence_false_ignores_sequence_omission() {
    // A single measure "Chorus" with three parts, played twice via
    // `sequence` with the first occurrence omitting S and A2. Selecting
    // the written measure with `respect_sequence: false` (as "play current
    // measure" does) must play it exactly as written, with all three parts,
    // regardless of what any `sequence` occurrence would have omitted.
    let measures = vec![measure_with_parts(0, &["S", "A2", "T"])];
    let score = score_with_sequence(
        measures,
        vec![SequenceSpan {
            label: "Chorus".to_string(),
            start: 0,
            end: 0,
            omit_parts: vec!["S".to_string(), "A2".to_string()],
            part_filter_display: None,
        }],
    );

    let (literal, start, end) = expand_for_measure_range(&score, 0, 0, false, false, None).unwrap();
    assert_eq!((start, end), (0, 0));
    assert_eq!(literal.measures[0].parts.len(), 3);

    let (respected, start, end) =
        expand_for_measure_range(&score, 0, 0, false, true, None).unwrap();
    assert_eq!((start, end), (0, 0));
    assert_eq!(respected.measures[0].parts.len(), 1);
}

#[test]
fn expand_for_measure_range_with_sequence_entry_index_selects_exact_occurrence() {
    // "Chorus" replayed three times via `sequence`: unmarked, then with S
    // and A2 omitted, then with only A2 omitted. Every occurrence shares the
    // same written measure range (0..=0), so selecting by written index
    // alone (`entry_index: None`) can only ever reach the *first* one.
    // Passing the entry's own `sequence` index must reach any of them.
    let measures = vec![measure_with_parts(0, &["S", "A2", "T"])];
    let score = score_with_sequence(
        measures,
        vec![
            SequenceSpan {
                label: "Chorus".to_string(),
                start: 0,
                end: 0,
                omit_parts: Vec::new(),
                part_filter_display: None,
            },
            SequenceSpan {
                label: "Chorus".to_string(),
                start: 0,
                end: 0,
                omit_parts: vec!["S".to_string(), "A2".to_string()],
                part_filter_display: None,
            },
            SequenceSpan {
                label: "Chorus".to_string(),
                start: 0,
                end: 0,
                omit_parts: vec!["A2".to_string()],
                part_filter_display: None,
            },
        ],
    );

    fn part_count(score: &Score, entry_index: usize) -> usize {
        let (expanded, start, end) =
            expand_for_measure_range(score, 0, 0, false, true, Some(entry_index..=entry_index))
                .unwrap();
        assert_eq!((start, end), (entry_index, entry_index));
        expanded.measures[start].parts.len()
    }

    assert_eq!(part_count(&score, 0), 3, "unmarked occurrence: all parts");
    assert_eq!(
        part_count(&score, 1),
        1,
        "second occurrence omits S and A2, leaving only T"
    );
    assert_eq!(
        part_count(&score, 2),
        2,
        "third occurrence omits only A2, leaving S and T"
    );
}

#[test]
fn expand_for_measure_range_with_sequence_entry_index_handles_reversed_written_order() {
    // Reproduces a user report: `sequence` is `X, Y(-b), Y, X` (X is
    // written measure 0, Y is written measure 1). Selecting the third and
    // fourth entries (the unmarked `Y`, then the second `X`) gives a
    // *written* range of `start: 1, end: 0` -- `Y`'s written index is
    // greater than `X`'s, even though `Y` is selected first -- because
    // `sequence` order and written order diverge. `expand_for_measure_range`
    // must still resolve this via `sequence_entry_range` instead of falling
    // into the `start_index > end_index` fallback (which silently swaps the
    // range and replays the literal written score, ignoring both the
    // omission and which occurrence was actually selected).
    let measures = vec![
        measure_with_parts(0, &["a"]),
        measure_with_parts(1, &["a", "b"]),
    ];
    let score = score_with_sequence(
        measures,
        vec![
            SequenceSpan {
                label: "X".to_string(),
                start: 0,
                end: 0,
                omit_parts: Vec::new(),
                part_filter_display: None,
            },
            SequenceSpan {
                label: "Y".to_string(),
                start: 1,
                end: 1,
                omit_parts: vec!["b".to_string()],
                part_filter_display: None,
            },
            SequenceSpan {
                label: "Y".to_string(),
                start: 1,
                end: 1,
                omit_parts: Vec::new(),
                part_filter_display: None,
            },
            SequenceSpan {
                label: "X".to_string(),
                start: 0,
                end: 0,
                omit_parts: Vec::new(),
                part_filter_display: None,
            },
        ],
    );

    fn names(measure: &MultiPartMeasure) -> Vec<&str> {
        measure
            .parts
            .iter()
            .filter_map(|p| p.name().map(String::as_str))
            .collect()
    }

    // Selected entries: the unmarked `Y` (index 2, written measure 1) then
    // the second `X` (index 3, written measure 0) -- written range (1, 0).
    let (expanded, start, end) =
        expand_for_measure_range(&score, 1, 0, false, true, Some(2..=3)).unwrap();
    assert_eq!((start, end), (2, 3));
    assert_eq!(
        names(&expanded.measures[start]),
        vec!["a", "b"],
        "the unmarked `Y` occurrence must keep part b, not the earlier `Y(-b)`'s omission"
    );
    assert_eq!(names(&expanded.measures[end]), vec!["a"]);
}
