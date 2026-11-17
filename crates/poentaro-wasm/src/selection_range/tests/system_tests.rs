use poentaro::grid_layout::MeasureRange;

use crate::selection_range::resolve_system_selection_range_response;
use crate::selection_range::types::{ClickableElementId, ResolveSelectionRangeResponse};

use super::test_helpers::{
    fixture, lyric, lyric_cell, lyric_label, measure, note, note_cell, part_label,
};

/// Measure 0 is one system; measures 1-2 are the next.
fn systems() -> Vec<MeasureRange> {
    vec![
        MeasureRange { start: 0, end: 0 },
        MeasureRange { start: 1, end: 2 },
    ]
}

fn resolve(
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> ResolveSelectionRangeResponse {
    let (note_spans, lyric_spans) = fixture();
    resolve_system_selection_range_response(&note_spans, &lyric_spans, &systems(), anchor, current)
}

fn assert_selects_measures_one_and_two(anchor: &ClickableElementId, current: &ClickableElementId) {
    for (first, second) in [(anchor, current), (current, anchor)] {
        let ResolveSelectionRangeResponse::Ok {
            note_cells,
            lyric_cells,
        } = resolve(first, second)
        else {
            panic!("expected Ok, got Err");
        };
        assert_eq!(
            note_cells,
            [note_cell(0, 1), note_cell(0, 2), note_cell(3, 3)]
        );
        assert_eq!(
            lyric_cells,
            [
                lyric_cell(1, 1),
                lyric_cell(1, 2),
                lyric_cell(2, 1),
                lyric_cell(4, 3)
            ]
        );
    }
}

#[test]
fn a_measure_selects_every_part_of_its_whole_system() {
    assert_selects_measures_one_and_two(&measure(1, 1), &measure(1, 1));
}

#[test]
fn a_part_label_selects_every_part_of_its_system() {
    assert_selects_measures_one_and_two(&part_label(0, 1, 2), &part_label(0, 1, 2));
}

#[test]
fn a_note_and_a_syllable_widen_to_their_system() {
    assert_selects_measures_one_and_two(&note(3, 3), &lyric(1, 2));
}

#[test]
fn a_lyric_label_widens_to_its_system() {
    assert_selects_measures_one_and_two(&lyric_label(4, 2, 2), &measure(1, 1));
}

#[test]
fn endpoints_in_different_systems_select_every_system_between() {
    let ResolveSelectionRangeResponse::Ok {
        note_cells,
        lyric_cells,
    } = resolve(&part_label(3, 0, 0), &note(0, 2))
    else {
        panic!("expected Ok, got Err");
    };
    assert_eq!(
        note_cells,
        [
            note_cell(0, 0),
            note_cell(0, 1),
            note_cell(0, 2),
            note_cell(3, 3)
        ]
    );
    assert_eq!(lyric_cells.len(), 6);
}

#[test]
fn a_measure_outside_every_system_selects_just_that_measure() {
    let (note_spans, lyric_spans) = fixture();
    let ResolveSelectionRangeResponse::Ok { note_cells, .. } =
        resolve_system_selection_range_response(
            &note_spans,
            &lyric_spans,
            &[],
            &measure(2, 2),
            &measure(2, 2),
        )
    else {
        panic!("expected Ok, got Err");
    };
    assert_eq!(note_cells, [note_cell(0, 2)]);
}

#[test]
fn a_note_missing_from_the_spans_is_unresolved() {
    assert!(matches!(
        resolve(&note(0, 99), &measure(0, 0)),
        ResolveSelectionRangeResponse::Err
    ));
}
