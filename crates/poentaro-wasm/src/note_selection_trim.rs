use std::collections::HashSet;

use itertools::Itertools;
use poentaro::wav::TrimWindow;

use crate::note_selection_types::NoteCellIn;
use crate::types::NoteTimingOut;

/// Narrows a measure-range clip down to the stretch that exactly covers the
/// selected notes, so "play selection" starts and stops at their real
/// boundaries instead of playing the whole boundary measures.
/// `next_note_start_s` caps the release tail at the earliest unselected note
/// starting at or after the window's end, so that note isn't heard.
///
/// `None` when no selected note has a timing, or the window is empty.
pub(crate) fn note_selection_trim_window(
    cells: &[NoteCellIn],
    note_timings: &[NoteTimingOut],
) -> Option<TrimWindow> {
    let selected: HashSet<NoteCellIn> = cells.iter().copied().collect();
    let is_selected = |timing: &&NoteTimingOut| {
        selected.contains(&NoteCellIn {
            source_part_index: timing.source_part_index,
            note_id: timing.note_id,
        })
    };
    let selected_timings = note_timings.iter().filter(is_selected).collect_vec();
    let start_s = selected_timings
        .iter()
        .map(|timing| timing.start_s)
        .min_by(f64::total_cmp)?;
    let end_s = selected_timings
        .iter()
        .map(|timing| timing.end_s)
        .max_by(f64::total_cmp)?;
    if end_s <= start_s {
        return None;
    }
    let next_note_start_s = note_timings
        .iter()
        .filter(|timing| !is_selected(timing))
        .map(|timing| timing.start_s)
        .filter(|&start| start >= end_s)
        .min_by(f64::total_cmp);
    Some(TrimWindow {
        start_s,
        end_s,
        next_note_start_s,
    })
}

#[cfg(test)]
#[path = "note_selection_trim_tests.rs"]
mod tests;
