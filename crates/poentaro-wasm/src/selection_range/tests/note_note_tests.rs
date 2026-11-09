use crate::responses::list_note_spans_response;
use crate::selection_range::resolve_selection_range_response;
use crate::selection_range::types::{
    ClickableElementId, NoteCellOut, ResolveSelectionRangeResponse,
};
use crate::types::{ListNoteSpansResponse, NoteSpanOut};

use super::test_helpers::{
    assert_range, fixture, lyric_cell, note, note_cell, note_span, timed_note_span,
};

/// Table-driven case: given `(anchor, current)`, resolving a same-part
/// `Note ↔ Note` range must produce exactly the expected note cells (never
/// any lyric cells — see `resolve_selection_range_response`'s doc comment
/// on why an index range has no notion of "row"), regardless of
/// anchor/current order.
fn assert_note_range(
    anchor: &ClickableElementId,
    current: &ClickableElementId,
    expected_note_cells: &[NoteCellOut],
) {
    let (note_spans, lyric_spans) = fixture();
    let response = resolve_selection_range_response(&note_spans, &lyric_spans, anchor, current);
    match response {
        ResolveSelectionRangeResponse::Ok {
            note_cells,
            lyric_cells,
        } => {
            assert_eq!(note_cells, expected_note_cells);
            assert_eq!(lyric_cells, Vec::new());
        }
        ResolveSelectionRangeResponse::Err => panic!("expected Ok, got Err"),
    }
}

#[test]
fn same_note_anchor_equals_current() {
    assert_note_range(&note(0, 1), &note(0, 1), &[note_cell(0, 1)]);
}

#[test]
fn multi_note_range_anchor_before_current() {
    assert_note_range(
        &note(0, 0),
        &note(0, 2),
        &[note_cell(0, 0), note_cell(0, 1), note_cell(0, 2)],
    );
}

#[test]
fn multi_note_range_current_before_anchor() {
    assert_note_range(
        &note(0, 2),
        &note(0, 0),
        &[note_cell(0, 0), note_cell(0, 1), note_cell(0, 2)],
    );
}

#[test]
fn cross_part_note_range_anchor_before_current() {
    // Parts 0-3, measures 0-1: the lyric rows between the two notes'
    // parts are selected too, and part 0's measure-2 note isn't.
    assert_range(
        &note(0, 0),
        &note(3, 3),
        &[note_cell(0, 0), note_cell(0, 1), note_cell(3, 3)],
        &[
            lyric_cell(1, 0),
            lyric_cell(1, 1),
            lyric_cell(2, 0),
            lyric_cell(2, 1),
        ],
    );
}

#[test]
fn cross_part_note_range_measure_with_no_notes_in_one_part_contributes_nothing() {
    // Measures 1-2 — part 3 has no note in measure 2, so it contributes
    // nothing there; no error.
    assert_range(
        &note(0, 2),
        &note(3, 3),
        &[note_cell(0, 1), note_cell(0, 2), note_cell(3, 3)],
        &[lyric_cell(1, 1), lyric_cell(1, 2), lyric_cell(2, 1)],
    );
}

/// The note cells a cross-part range selects in `note_spans`, which must
/// hold no lyric cells.
fn resolved_note_cells(
    note_spans: &[NoteSpanOut],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> Vec<NoteCellOut> {
    match resolve_selection_range_response(note_spans, &[], anchor, current) {
        ResolveSelectionRangeResponse::Ok {
            note_cells,
            lyric_cells,
        } => {
            assert_eq!(lyric_cells, Vec::new());
            note_cells
        }
        ResolveSelectionRangeResponse::Err => panic!("expected Ok, got Err"),
    }
}

#[test]
fn cross_part_note_range_within_same_measure_is_bounded_by_time() {
    // `1 2 3` / `4 5 6` on two parts: note 1 of the first and note 2 of the
    // second select `1 2` / `4 5`, not the whole shared measure.
    let note_spans = vec![
        timed_note_span(0, 0, 0, 0, 4),
        timed_note_span(0, 1, 0, 4, 4),
        timed_note_span(0, 2, 0, 8, 4),
        timed_note_span(1, 3, 0, 0, 4),
        timed_note_span(1, 4, 0, 4, 4),
        timed_note_span(1, 5, 0, 8, 4),
    ];

    assert_eq!(
        resolved_note_cells(&note_spans, &note(0, 0), &note(1, 4)),
        vec![
            note_cell(0, 0),
            note_cell(0, 1),
            note_cell(1, 3),
            note_cell(1, 4),
        ]
    );
}

#[test]
fn cross_part_note_range_across_measures_runs_from_start_to_end() {
    // Like multi-line text selection: from the anchor to the end of its
    // measure, then from the start of the current's measure up to it, in
    // both parts.
    let quarters = |part: usize, first_id: usize, measure: usize| {
        (0..4)
            .map(move |index| timed_note_span(part, first_id + index, measure, index as u32 * 4, 4))
    };
    let note_spans = quarters(0, 0, 0)
        .chain(quarters(1, 4, 0))
        .chain(quarters(0, 8, 1))
        .chain(quarters(1, 12, 1))
        .collect::<Vec<_>>();

    assert_eq!(
        resolved_note_cells(&note_spans, &note(0, 1), &note(1, 13)),
        vec![
            note_cell(0, 1),
            note_cell(0, 2),
            note_cell(0, 3),
            note_cell(1, 5),
            note_cell(1, 6),
            note_cell(1, 7),
            note_cell(0, 8),
            note_cell(0, 9),
            note_cell(1, 12),
            note_cell(1, 13),
        ]
    );
}

/// Resolves a cross-part range over the spans of real `.poentaro` source.
fn resolved_note_cells_in_source(
    source: &str,
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> Vec<NoteCellOut> {
    let source = format!(
        "metadata {{ hide resting parts : no, merge duplicate measures across parts : no }},\n\n{source}"
    );
    let ListNoteSpansResponse::Ok { spans } = list_note_spans_response(&source, None) else {
        panic!("failed to compile:\n{source}");
    };
    resolved_note_cells(&spans, anchor, current)
}

#[test]
fn cross_part_note_range_selects_a_held_note_in_a_middle_part() {
    let source = r#"parts ["A": notes, "B": notes, "C": notes],

score """
    [A] 1 2 3 4
    [B] 5---
    [C] 1 2 3 4
    """,
"#;

    assert_eq!(
        resolved_note_cells_in_source(source, &note(0, 2), &note(2, 3)),
        vec![
            note_cell(0, 2),
            note_cell(0, 3),
            note_cell(1, 0),
            note_cell(2, 2),
            note_cell(2, 3),
        ]
    );
}

#[test]
fn cross_part_note_range_bounds_a_different_rhythm_by_time() {
    // A's 2nd note spans eighths 3-4 of B; B's 5th note ends at A's 3rd.
    let source = r#"parts ["A": notes, "B": notes],

score """
    [A] 1 2 3 4
    [B] 5_ 6_ 7_ 1_ 2_ 3_ 4_ 5_
    """,
"#;

    assert_eq!(
        resolved_note_cells_in_source(source, &note(0, 1), &note(1, 4)),
        vec![
            note_cell(0, 1),
            note_cell(0, 2),
            note_cell(1, 2),
            note_cell(1, 3),
            note_cell(1, 4),
        ]
    );
}

#[test]
fn cross_part_note_range_covers_every_measure_of_a_tied_endpoint() {
    // The shrunk proptest case: A's `1~` is tied into the next measure, so
    // the range runs through B's last eighth, `6_`.
    let source = r#"parts ["A": notes, "B": notes, "C": notes],

score """
    [A] 0 0 0 0
    [B] 0---
    [C] 0 3_ 3~- 3_

    [A] 0- 0_ 1~ 1_
    [B] 1 2_ 4- 6_
    [C] 4-- 1
    """,
"#;

    let selected = resolved_note_cells_in_source(source, &note(0, 6), &note(2, 0));
    assert!(selected.contains(&note_cell(1, 4)), "{selected:?}");
}

#[test]
fn same_part_note_range_uses_note_id_not_measure_index() {
    // Distinguishes the guarded same-part arm (note_id-based) from the
    // cross-part arm below it (measure_index-based): note_id 5 shares
    // measure_index 1 with note_id 1, but sits outside the anchor/current
    // note_id range [0, 1]. If the cross-part arm ever fired for a
    // same-part pair, it would wrongly include note_id 5 via its measure
    // match — the guard on the arm above ensures it never does.
    let note_spans = vec![
        note_span(0, 0, 0),
        note_span(0, 1, 1),
        note_span(0, 5, 1),
        note_span(0, 2, 2),
    ];
    let lyric_spans = Vec::new();
    let anchor = note(0, 0);
    let current = note(0, 1);

    let response = resolve_selection_range_response(&note_spans, &lyric_spans, &anchor, &current);

    match response {
        ResolveSelectionRangeResponse::Ok {
            note_cells,
            lyric_cells,
        } => {
            assert_eq!(note_cells, vec![note_cell(0, 0), note_cell(0, 1)]);
            assert_eq!(lyric_cells, Vec::new());
        }
        ResolveSelectionRangeResponse::Err => panic!("expected Ok, got Err"),
    }
}
