use super::*;

#[test]
fn multi_part_score_assigns_correct_part_and_measure_indices() {
    let source = r#"metadata { title "t" },

parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    [M] 1 2 3 4
    [B] 5 6 7 1'
    """,
"#;
    let spans = list_note_spans_from_source(source, "test.poentaro", None)
        .unwrap()
        .spans;

    let melody: Vec<_> = spans.iter().filter(|s| s.source_part_index == 0).collect();
    let bass: Vec<_> = spans.iter().filter(|s| s.source_part_index == 1).collect();

    assert_eq!(melody.len(), 4);
    assert_eq!(bass.len(), 4);
    assert!(spans.iter().all(|s| s.measure_index == 0));
    assert_eq!(
        melody.iter().map(|s| s.note_id).collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert_eq!(
        bass.iter().map(|s| s.note_id).collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert!(spans.iter().all(|s| s.start.is_some() && s.end.is_some()));
}

#[test]
fn tie_continuation_reuses_one_span_id() {
    let source = r#"metadata { title "t" },

parts ["Melody" [M]: notes],

score """
    [M] 4~4 3 2
    """,
"#;
    let spans = list_note_spans_from_source(source, "test.poentaro", None)
        .unwrap()
        .spans;

    assert_eq!(spans.len(), 4);
    assert_eq!(
        spans.iter().map(|s| s.note_id).collect::<Vec<_>>(),
        vec![0, 0, 2, 3],
        "the tied note should reuse the id of the note it continues from"
    );
}

#[test]
fn chord_and_percussion_parts_get_spans() {
    let source = r#"metadata { title "t" },

parts ["Melody" [M]: notes, "Chords" [C]: chords, "Perc" [P]: percussion { sound 38 }],

score """
    [M] 1 2 3 4
    [C] 1 1m 1 1m
    [P] x x x x
    """,
"#;
    let spans = list_note_spans_from_source(source, "test.poentaro", None)
        .unwrap()
        .spans;

    for part_index in 0..3 {
        let part_spans: Vec<_> = spans
            .iter()
            .filter(|s| s.source_part_index == part_index)
            .collect();
        assert_eq!(part_spans.len(), 4, "part {part_index} should have 4 spans");
        assert!(
            part_spans.iter().all(|s| s.start.is_some()),
            "part {part_index} spans should all have a source span"
        );
    }
}

#[test]
fn rest_yields_a_span_over_its_own_token() {
    let source = r#"metadata { title "t" },

parts ["Melody" [M]: notes],

score """
    [M] 1 0 3 4
    """,
"#;
    let spans = list_note_spans_from_source(source, "test.poentaro", None)
        .unwrap()
        .spans;

    assert_eq!(spans.len(), 4);
    assert!(spans.iter().all(|s| s.start.is_some() && s.end.is_some()));
    let rest_start = spans[1].start.unwrap();
    let rest_end = spans[1].end.unwrap();
    assert_eq!(&source[rest_start..rest_end], "0");
}

/// Stand-in abbreviation for a hand-built span's part, so each test part
/// carries a distinct name its grouped runs can be checked against.
fn test_part_abbreviation(source_part_index: usize) -> String {
    format!("P{source_part_index}")
}

fn span(
    source_part_index: usize,
    note_id: usize,
    measure_index: usize,
    start: usize,
    end: usize,
) -> NoteSourceSpan {
    NoteSourceSpan {
        source_part_index,
        part_abbreviation: Some(test_part_abbreviation(source_part_index)),
        note_id,
        measure_index,
        start: Some(start),
        end: Some(end),
        timing: MeasureTiming::default(),
    }
}

/// A cell with no mappable source span. No current event kind produces
/// this (every `NoteEvent` variant, including `Rest`, carries a byte span),
/// but `group_selected_notes_into_contiguous_runs` still guards against it
/// for a future event kind that might.
fn unmappable_span(
    source_part_index: usize,
    note_id: usize,
    measure_index: usize,
) -> NoteSourceSpan {
    NoteSourceSpan {
        source_part_index,
        part_abbreviation: Some(test_part_abbreviation(source_part_index)),
        note_id,
        measure_index,
        start: None,
        end: None,
        timing: MeasureTiming::default(),
    }
}

#[test]
fn multiple_selected_cells_merge_into_one_run_per_part_measure() {
    let spans = vec![
        span(0, 0, 0, 10, 11),
        span(0, 1, 0, 12, 13),
        span(0, 2, 0, 14, 15),
    ];
    let cells = vec![
        NoteCell {
            source_part_index: 0,
            note_id: 0,
        },
        NoteCell {
            source_part_index: 0,
            note_id: 1,
        },
        NoteCell {
            source_part_index: 0,
            note_id: 2,
        },
    ];

    let runs = group_selected_notes_into_contiguous_runs(&cells, &spans);

    assert_eq!(
        runs,
        vec![NoteSelectionRun {
            source_part_index: 0,
            part_abbreviation: Some(test_part_abbreviation(0)),
            measure_index: 0,
            start_byte: 10,
            end_byte: 15,
        }]
    );
}

#[test]
fn selected_cell_with_no_mappable_span_does_not_break_contiguity() {
    let spans = vec![
        span(0, 0, 0, 10, 11),
        unmappable_span(0, 1, 0),
        span(0, 2, 0, 14, 15),
    ];
    let cells = vec![
        NoteCell {
            source_part_index: 0,
            note_id: 0,
        },
        NoteCell {
            source_part_index: 0,
            note_id: 1,
        },
        NoteCell {
            source_part_index: 0,
            note_id: 2,
        },
    ];

    let runs = group_selected_notes_into_contiguous_runs(&cells, &spans);

    assert_eq!(
        runs,
        vec![NoteSelectionRun {
            source_part_index: 0,
            part_abbreviation: Some(test_part_abbreviation(0)),
            measure_index: 0,
            start_byte: 10,
            end_byte: 15,
        }]
    );
}

#[test]
fn selecting_only_a_rest_selects_its_source_token() {
    // Clicking the "0" rest in the SVG preview should select the "0" in
    // the Monaco editor: a selection containing only a rest cell must
    // still produce a run, not be silently dropped.
    let source = r#"metadata { title "t" },

parts ["Melody" [M]: notes],

score """
    [M] 1 0 3 4
    """,
"#;
    let spans = list_note_spans_from_source(source, "test.poentaro", None)
        .unwrap()
        .spans;
    let rest_cell = NoteCell {
        source_part_index: 0,
        note_id: spans[1].note_id,
    };

    let runs = group_selected_notes_into_contiguous_runs(&[rest_cell], &spans);

    assert_eq!(runs.len(), 1);
    assert_eq!(&source[runs[0].start_byte..runs[0].end_byte], "0");
}

#[test]
fn different_parts_and_measures_produce_separate_runs() {
    let spans = vec![
        span(0, 0, 0, 10, 11),
        span(1, 0, 0, 20, 21),
        span(0, 1, 1, 30, 31),
    ];
    let cells = vec![
        NoteCell {
            source_part_index: 0,
            note_id: 0,
        },
        NoteCell {
            source_part_index: 1,
            note_id: 0,
        },
        NoteCell {
            source_part_index: 0,
            note_id: 1,
        },
    ];

    let runs = group_selected_notes_into_contiguous_runs(&cells, &spans);

    assert_eq!(
        runs,
        vec![
            NoteSelectionRun {
                source_part_index: 0,
                part_abbreviation: Some(test_part_abbreviation(0)),
                measure_index: 0,
                start_byte: 10,
                end_byte: 11,
            },
            NoteSelectionRun {
                source_part_index: 0,
                part_abbreviation: Some(test_part_abbreviation(0)),
                measure_index: 1,
                start_byte: 30,
                end_byte: 31,
            },
            NoteSelectionRun {
                source_part_index: 1,
                part_abbreviation: Some(test_part_abbreviation(1)),
                measure_index: 0,
                start_byte: 20,
                end_byte: 21,
            },
        ]
    );
}

#[test]
fn empty_selection_produces_no_runs() {
    let spans = vec![span(0, 0, 0, 10, 11)];

    let runs = group_selected_notes_into_contiguous_runs(&[], &spans);

    assert!(runs.is_empty());
}

#[test]
fn output_is_sorted_by_part_then_measure() {
    let spans = vec![
        span(1, 0, 2, 50, 51),
        span(0, 0, 1, 30, 31),
        span(1, 1, 0, 10, 11),
        span(0, 1, 0, 20, 21),
    ];
    let cells = vec![
        NoteCell {
            source_part_index: 1,
            note_id: 0,
        },
        NoteCell {
            source_part_index: 0,
            note_id: 0,
        },
        NoteCell {
            source_part_index: 1,
            note_id: 1,
        },
        NoteCell {
            source_part_index: 0,
            note_id: 1,
        },
    ];

    let runs = group_selected_notes_into_contiguous_runs(&cells, &spans);

    let keys: Vec<(usize, usize)> = runs
        .iter()
        .map(|r| (r.source_part_index, r.measure_index))
        .collect();
    assert_eq!(keys, vec![(0, 0), (0, 1), (1, 0), (1, 2)]);
}
