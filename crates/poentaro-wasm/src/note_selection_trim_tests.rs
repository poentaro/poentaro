use poentaro::wav::TrimWindow;

use super::note_selection_trim_window;
use crate::note_selection_types::NoteCellIn;
use crate::types::NoteTimingOut;

fn timing(source_part_index: usize, note_id: usize, start_s: f64, end_s: f64) -> NoteTimingOut {
    NoteTimingOut {
        source_part_index,
        note_id,
        start_s,
        end_s,
    }
}

fn cell(source_part_index: usize, note_id: usize) -> NoteCellIn {
    NoteCellIn {
        source_part_index,
        note_id,
    }
}

fn window(start_s: f64, end_s: f64, next_note_start_s: Option<f64>) -> Option<TrimWindow> {
    Some(TrimWindow {
        start_s,
        end_s,
        next_note_start_s,
    })
}

#[test]
fn narrows_to_the_selected_notes_only() {
    let timings = [
        timing(0, 0, 0.0, 1.0),
        timing(0, 1, 1.0, 2.0),
        timing(0, 2, 2.0, 3.0),
        timing(0, 3, 3.0, 4.0),
    ];
    assert_eq!(
        note_selection_trim_window(&[cell(0, 1), cell(0, 2)], &timings),
        window(1.0, 3.0, Some(3.0))
    );
}

#[test]
fn caps_at_the_earliest_unselected_note_starting_at_or_after_the_end() {
    let timings = [
        timing(0, 0, 0.0, 1.0),
        timing(0, 1, 1.0, 1.5),
        timing(0, 2, 1.5, 4.0),
        timing(0, 3, 4.0, 5.0),
    ];
    assert_eq!(
        note_selection_trim_window(&[cell(0, 1)], &timings),
        window(1.0, 1.5, Some(1.5))
    );
}

#[test]
fn has_no_cap_when_the_selection_reaches_the_last_note() {
    let timings = [timing(0, 0, 0.0, 1.0), timing(0, 1, 1.0, 2.0)];
    assert_eq!(
        note_selection_trim_window(&[cell(0, 1)], &timings),
        window(1.0, 2.0, None)
    );
}

#[test]
fn ignores_notes_outside_the_selection() {
    let timings = [timing(0, 0, 0.0, 1.0), timing(1, 0, 0.0, 5.0)];
    assert_eq!(
        note_selection_trim_window(&[cell(0, 0)], &timings),
        window(0.0, 1.0, None)
    );
}

#[test]
fn is_none_when_no_selected_note_has_a_timing() {
    let timings = [timing(0, 0, 0.0, 1.0)];
    assert_eq!(note_selection_trim_window(&[cell(5, 9)], &timings), None);
}

#[test]
fn is_none_for_an_empty_selection() {
    let timings = [timing(0, 0, 0.0, 1.0)];
    assert_eq!(note_selection_trim_window(&[], &timings), None);
}
