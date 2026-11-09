use super::*;

#[test]
fn span_covers_each_syllables_own_source_text() {
    let source = r#"metadata { title "t" },

parts ["Melody" [M]: notes, "Melody lyrics" [Mv1]: lyrics [M]],

score """
    [M] 1 2 3 4
    [Mv1] a b c d
    """,
"#;
    let spans = list_lyric_spans_from_source(source, "test.poentaro", None)
        .unwrap()
        .spans;

    assert_eq!(spans.len(), 4);
    let texts: Vec<&str> = spans.iter().map(|s| &source[s.start..s.end]).collect();
    assert_eq!(texts, vec!["a", "b", "c", "d"]);
    assert_eq!(
        spans.iter().map(|s| s.note_id).collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert!(spans.iter().all(|s| s.measure_index == 0));
    // The lyric part's own index (after its target's), not its target's.
    assert!(spans.iter().all(|s| s.source_part_index == 1));
    assert!(spans
        .iter()
        .all(|s| s.part_abbreviation.as_deref() == Some("Mv1")));
}

#[test]
fn tie_continuation_note_consumes_no_extra_syllable() {
    let source = r#"metadata { title "t" },

parts ["Melody" [M]: notes, "Melody lyrics" [Mv1]: lyrics [M]],

score """
    [M] 4~4 3 2
    [Mv1] la di dum
    """,
"#;
    let spans = list_lyric_spans_from_source(source, "test.poentaro", None)
        .unwrap()
        .spans;

    // 3 notes, but the tied continuation isn't its own note event needing a
    // syllable of its own: "4~4" is a single tied note followed by "3 2", so
    // only 3 syllable-taking note attacks exist, matching the 3 syllables.
    // Note ids still count the tie-continuation event itself (it just reuses
    // its attack's own id rather than getting a fresh one), so the
    // syllable-bearing ids are 0 (the tied attack), then 2 and 3 (the
    // continuation event consumed id 1 without taking a syllable).
    let texts: Vec<&str> = spans.iter().map(|s| &source[s.start..s.end]).collect();
    assert_eq!(texts, vec!["la", "di", "dum"]);
    assert_eq!(
        spans.iter().map(|s| s.note_id).collect::<Vec<_>>(),
        vec![0, 2, 3]
    );
}

#[test]
fn each_lyric_part_produces_its_own_spans_sharing_note_ids() {
    let source = r#"metadata { title "t" },

parts ["Melody" [M]: notes, "Melody verse 1" [Mv1]: lyrics [M], "Melody verse 2" [Mv2]: lyrics [M]],

score """
    [M] 1 2
    [Mv1] a b
    [Mv2] one two
    """,
"#;
    let spans = list_lyric_spans_from_source(source, "test.poentaro", None)
        .unwrap()
        .spans;

    assert_eq!(spans.len(), 4);
    let verse0: Vec<_> = spans.iter().filter(|s| s.source_part_index == 1).collect();
    let verse1: Vec<_> = spans.iter().filter(|s| s.source_part_index == 2).collect();
    assert_eq!(verse0.len(), 2);
    assert_eq!(verse1.len(), 2);
    assert_eq!(
        verse0.iter().map(|s| s.note_id).collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(
        verse1.iter().map(|s| s.note_id).collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(&source[verse0[0].start..verse0[0].end], "a");
    assert_eq!(&source[verse1[0].start..verse1[0].end], "one");
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
) -> LyricSourceSpan {
    LyricSourceSpan {
        source_part_index,
        part_abbreviation: Some(test_part_abbreviation(source_part_index)),
        note_id,
        measure_index,
        start,
        end,
        timing: MeasureTiming::default(),
    }
}

#[test]
fn multiple_selected_cells_merge_into_one_run_per_lyric_part_measure() {
    let spans = vec![
        span(0, 0, 0, 10, 11),
        span(0, 1, 0, 12, 13),
        span(0, 2, 0, 14, 15),
    ];
    let cells = vec![
        LyricCell {
            source_part_index: 0,
            note_id: 0,
        },
        LyricCell {
            source_part_index: 0,
            note_id: 1,
        },
        LyricCell {
            source_part_index: 0,
            note_id: 2,
        },
    ];

    let runs = group_selected_lyrics_into_contiguous_runs(&cells, &spans);

    assert_eq!(
        runs,
        vec![LyricSelectionRun {
            source_part_index: 0,
            part_abbreviation: Some(test_part_abbreviation(0)),
            measure_index: 0,
            start_byte: 10,
            end_byte: 15,
        }]
    );
}

#[test]
fn different_lyric_parts_of_the_same_note_produce_separate_runs() {
    let spans = vec![span(1, 0, 0, 10, 11), span(2, 0, 0, 40, 43)];
    let cells = vec![
        LyricCell {
            source_part_index: 1,
            note_id: 0,
        },
        LyricCell {
            source_part_index: 2,
            note_id: 0,
        },
    ];

    let runs = group_selected_lyrics_into_contiguous_runs(&cells, &spans);

    assert_eq!(
        runs,
        vec![
            LyricSelectionRun {
                source_part_index: 1,
                part_abbreviation: Some(test_part_abbreviation(1)),
                measure_index: 0,
                start_byte: 10,
                end_byte: 11,
            },
            LyricSelectionRun {
                source_part_index: 2,
                part_abbreviation: Some(test_part_abbreviation(2)),
                measure_index: 0,
                start_byte: 40,
                end_byte: 43,
            },
        ]
    );
}

#[test]
fn empty_selection_produces_no_runs() {
    let spans = vec![span(0, 0, 0, 10, 11)];

    let runs = group_selected_lyrics_into_contiguous_runs(&[], &spans);

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
        LyricCell {
            source_part_index: 1,
            note_id: 0,
        },
        LyricCell {
            source_part_index: 0,
            note_id: 0,
        },
        LyricCell {
            source_part_index: 1,
            note_id: 1,
        },
        LyricCell {
            source_part_index: 0,
            note_id: 1,
        },
    ];

    let runs = group_selected_lyrics_into_contiguous_runs(&cells, &spans);

    let keys: Vec<(usize, usize)> = runs
        .iter()
        .map(|r| (r.source_part_index, r.measure_index))
        .collect();
    assert_eq!(keys, vec![(0, 0), (0, 1), (1, 0), (1, 2)]);
}

#[test]
fn note_ids_skip_the_notes_of_a_measure_without_lyrics() {
    let source = r#"parts ["M": notes, "L": lyrics [M]],

score """
    time=4/4
    [M] 6,~---
    [L] a

    [M] 6, 5 5 5

    [M] 6, 3_2_ 1 6,
    [L] A B C D E
    """,
"#;
    let spans = list_lyric_spans_from_source(source, "test.poentaro", None)
        .unwrap()
        .spans;

    assert_eq!(
        spans.iter().map(|s| s.note_id).collect::<Vec<_>>(),
        vec![0, 5, 6, 7, 8, 9]
    );
}
