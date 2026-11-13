//! Lyric-side click-target computation, split out of `click_targets.rs` to
//! keep that file under the max file-length cap. Every `Lyric*` click target
//! (syllable and verse-label alike) is computed here.

use crate::compiler::types::{ElementContent, LyricLink, MeasureBlock, MeasureRow};
use crate::grid_layout::layout::{
    block_column_width, is_chord_only_row, is_lyric_row, sings_along_to, SystemBands, LABEL_COLS,
    MUSIC_START_COL,
};
use crate::grid_layout::playback_cursor::{block_has_bar_line, group_elements_by_note_id};
use crate::grid_layout::system_walk::for_each_system;
use crate::grid_layout::types::{GridElement, Header, LyricClickTarget, LyricLabelClickTarget};
use std::collections::HashMap;

/// One absolute row index per entry in `first.rows`, `Some` only for entries
/// that are a lyric row (`is_lyric_row`) — `None` for note rows. Mirrors
/// `note_row_spans`'s cursor walk exactly (same sub-row counts, same
/// verse-absorption order), but records each verse row's own single row
/// index instead of merging it into its note row's combined span, since a
/// lyric syllable's click target needs to sit on exactly the one row its
/// text is drawn on.
fn lyric_row_absolute_indices(
    lyric_links: &[LyricLink],
    system: &[MeasureBlock],
    row_offset: usize,
    bands: &SystemBands,
) -> Vec<Option<usize>> {
    let Some(first) = system.first() else {
        return Vec::new();
    };
    let mut cursor = row_offset;
    let mut result: Vec<Option<usize>> = Vec::with_capacity(first.rows.len());
    let mut idx = 0;
    while let Some(part_template) = first.rows.get(idx) {
        if is_lyric_row(part_template) {
            result.push(Some(cursor));
            cursor += 1;
            idx += 1;
            continue;
        }
        let sub_count = bands.sub_row_count(idx, part_template);
        result.push(None);
        cursor += sub_count;
        let mut verse_end = idx + 1;
        while first
            .rows
            .get(verse_end)
            .is_some_and(|verse_row| sings_along_to(lyric_links, verse_row, part_template))
        {
            result.push(Some(cursor));
            cursor += 1;
            verse_end += 1;
        }
        idx = verse_end;
    }
    result
}

/// One entry per `first.rows` index, `Some(note_part_idx)` for a lyric row
/// naming the note row it's paired with (the row it sings along to),
/// `None` for a note row (or a lyric row whose target row is absent). Mirrors
/// the same lyric-row absorption walk `note_row_spans`/`lyric_row_absolute_indices` use,
/// so a lyric syllable's click target can be widened to match its note's own
/// written column span.
fn lyric_owner_note_row_indices(
    lyric_links: &[LyricLink],
    system: &[MeasureBlock],
) -> Vec<Option<usize>> {
    let Some(first) = system.first() else {
        return Vec::new();
    };
    let mut result: Vec<Option<usize>> = vec![None; first.rows.len()];
    let mut idx = 0;
    while let Some(part_template) = first.rows.get(idx) {
        if is_lyric_row(part_template) {
            idx += 1;
            continue;
        }
        let note_idx = idx;
        let mut verse_idx = idx + 1;
        while first
            .rows
            .get(verse_idx)
            .is_some_and(|verse_row| sings_along_to(lyric_links, verse_row, part_template))
        {
            if let Some(slot) = result.get_mut(verse_idx) {
                *slot = Some(note_idx);
            }
            verse_idx += 1;
        }
        idx = verse_idx;
    }
    result
}

/// Per-`(block, part_row)` position shared by every syllable target emitted
/// from that row — bundled to keep [`push_lyric_click_targets_for_row`]
/// under the max-arguments lint.
struct LyricRowPosition<'a> {
    part_row: &'a MeasureRow,
    row_idx: usize,
    block_idx: usize,
    last_block_idx: usize,
    col_offset: u32,
    col_w: u32,
    has_bar_line: bool,
}

/// Emits one [`LyricClickTarget`] per real (non-empty-text) syllable in
/// `pos.part_row`, skipping a padded/synthetic blank-verse row's `Lyric`
/// element (see `make_padding_row` in `layout_systems.rs`) — it carries
/// empty text so `is_lyric_row` still classifies the row correctly, but has
/// no real syllable to click. Split out of `compute_all_lyric_click_targets`
/// to keep that function under the max-function-length lint.
fn push_lyric_click_targets_for_row(
    pos: &LyricRowPosition<'_>,
    note_max_cols: &HashMap<u32, u32>,
    page_idx: usize,
    results: &mut Vec<(usize, LyricClickTarget)>,
) {
    // The paired note row's own rightmost/leftmost written column, used
    // below to detect the measure's last/first note — mirrors
    // `compute_all_playback_cursor_targets`'s `row_max_col`/`row_min_col`.
    let row_max_col = note_max_cols.values().copied().max();
    let row_min_col = pos
        .part_row
        .elements
        .iter()
        .filter(|e| matches!(e.content, ElementContent::Lyric { .. }))
        .map(|e| e.column)
        .min();
    for el in &pos.part_row.elements {
        let ElementContent::Lyric { text, note_id, .. } = &el.content else {
            continue;
        };
        // Can't gate on `ColumnElement::note_id` the way the note/rest case
        // does: that field is documented to always be `None` for `Lyric`
        // content, real or padded — see `ColumnElement::note_id`'s doc
        // comment. Empty text is the padding signal instead.
        if text.is_empty() {
            continue;
        }
        // Snap the left edge to the rendered x of the system's leading bar
        // line for the measure's first syllable, exactly like
        // `compute_all_playback_cursor_targets` snaps the first note's own
        // click target — otherwise the box starts a whole column short of
        // (or past) where the bar line is actually drawn.
        let column_start = if pos.block_idx == 0 && Some(el.column) == row_min_col {
            LABEL_COLS as f32
        } else {
            (pos.col_offset + el.column) as f32
        };
        // Snap the right edge to the rendered x of this block's own trailing
        // bar line for the measure's last syllable, mirroring
        // `compute_all_playback_cursor_targets`'s own `column_end` snap:
        // flush right (`bar_line_col + 1.0`) for the system's last measure,
        // otherwise centered on the bar line (`bar_line_col + 0.5`) like an
        // inter-measure one is drawn.
        let column_end = match note_max_cols.get(&el.column) {
            Some(&max_col) if pos.has_bar_line && Some(max_col) == row_max_col => {
                let bar_line_col = (pos.col_offset + pos.col_w - 1) as f32;
                if pos.block_idx == pos.last_block_idx {
                    bar_line_col + 1.0
                } else {
                    bar_line_col + 0.5
                }
            }
            Some(&max_col) => (pos.col_offset + max_col + 1) as f32,
            None => column_start + 1.0,
        };
        results.push((
            page_idx,
            LyricClickTarget {
                row: pos.row_idx,
                column_start,
                column_end,
                source_part_index: pos.part_row.source_part_index,
                note_id: *note_id,
            },
        ));
    }
}

/// One [`LyricClickTarget`] per lyric syllable in every system, keyed by page
/// index like `compute_all_measure_click_targets`. A syllable's target
/// starts at the one grid column `expand_lyric_part` placed it in, but its
/// `column_end` is widened to match its own note's full written column span
/// (attack plus any dash-continuation columns, via `group_elements_by_note_id`
/// on the paired note row) so a multi-beat note's hover/click box covers its
/// whole duration rather than stopping after the first beat. Both edges are
/// then snapped to a system's leading/trailing bar line where relevant
/// (`column_start` for the first syllable, `column_end` for the last),
/// exactly like `compute_all_playback_cursor_targets` snaps a note's own
/// click target.
pub(crate) fn compute_all_lyric_click_targets(
    lyric_links: &[LyricLink],
    page_systems: &[Vec<Vec<MeasureBlock>>],
    tuplet_bracket_map: &HashMap<(usize, usize), Vec<GridElement>>,
    header: &Header,
    base: f32,
    hide_system_dividers: bool,
) -> Vec<(usize, LyricClickTarget)> {
    let mut results: Vec<(usize, LyricClickTarget)> = Vec::new();

    for_each_system(
        page_systems,
        tuplet_bracket_map,
        header,
        base,
        hide_system_dividers,
        |page_idx, system, row_offset, bands, _musical_row_count| {
            let row_indices = lyric_row_absolute_indices(lyric_links, system, row_offset, bands);
            let owner_note_row_indices = lyric_owner_note_row_indices(lyric_links, system);

            let last_block_idx = system.len().saturating_sub(1);
            let mut col_offset: u32 = MUSIC_START_COL;
            for (block_idx, block) in system.iter().enumerate() {
                let col_w = block_column_width(block);
                let has_bar_line = block_has_bar_line(block);
                for (part_idx, row_idx) in row_indices.iter().enumerate() {
                    let Some(row_idx) = row_idx else {
                        continue;
                    };
                    let Some(part_row) = block.rows.get(part_idx) else {
                        continue;
                    };
                    // The paired note row's own `(start column -> max_col)`
                    // map, used to widen each syllable's `column_end` to the
                    // full written span of the note starting under it rather
                    // than the single column the syllable text itself sits
                    // in. Keyed by column, not note id: a lyric part sung on
                    // several parts may pair with one other than the row it
                    // is drawn under, whose note ids then differ.
                    //
                    // Not for a chord-targeting lyric row: the lyric line
                    // carries its own timing there, so a bracketed syllable
                    // widened to its chord's whole span would overlap the
                    // unbracketed syllables spread inside that span. Every
                    // syllable keeps its own single-column box instead.
                    let note_max_cols: HashMap<u32, u32> = owner_note_row_indices
                        .get(part_idx)
                        .copied()
                        .flatten()
                        .and_then(|note_idx| block.rows.get(note_idx))
                        .filter(|note_row| !is_chord_only_row(note_row))
                        .map(|note_row| {
                            group_elements_by_note_id(&note_row.elements)
                                .into_iter()
                                .map(|(_note_id, min_col, max_col)| (min_col, max_col))
                                .collect()
                        })
                        .unwrap_or_default();
                    push_lyric_click_targets_for_row(
                        &LyricRowPosition {
                            part_row,
                            row_idx: *row_idx,
                            block_idx,
                            last_block_idx,
                            col_offset,
                            col_w,
                            has_bar_line,
                        },
                        &note_max_cols,
                        page_idx,
                        &mut results,
                    );
                }
                col_offset += col_w;
            }
        },
    );
    results
}

/// One [`LyricLabelClickTarget`] per verse row's `RowLabel` in every system,
/// keyed by page index like `compute_all_part_label_click_targets` — the
/// lyric-side mirror of that function, reusing `lyric_row_absolute_indices`
/// (the same per-verse-row index walk `compute_all_lyric_click_targets`
/// uses) instead of `note_row_spans`.
pub(crate) fn compute_all_lyric_label_click_targets(
    lyric_links: &[LyricLink],
    page_systems: &[Vec<Vec<MeasureBlock>>],
    tuplet_bracket_map: &HashMap<(usize, usize), Vec<GridElement>>,
    header: &Header,
    base: f32,
    hide_system_dividers: bool,
) -> Vec<(usize, LyricLabelClickTarget)> {
    let mut global_measure_index: usize = 0;
    let mut results: Vec<(usize, LyricLabelClickTarget)> = Vec::new();

    for_each_system(
        page_systems,
        tuplet_bracket_map,
        header,
        base,
        hide_system_dividers,
        |page_idx, system, row_offset, bands, _musical_row_count| {
            let row_indices = lyric_row_absolute_indices(lyric_links, system, row_offset, bands);

            let measure_index_start = global_measure_index;
            for block in system {
                global_measure_index += block.represents_measures;
            }
            let measure_index_end = global_measure_index.saturating_sub(1);

            let Some(first) = system.first() else {
                return;
            };
            for (part_idx, row_idx) in row_indices.iter().enumerate() {
                let Some(row_idx) = row_idx else {
                    continue;
                };
                let Some(part_template) = first.rows.get(part_idx) else {
                    continue;
                };
                if part_template.label.is_empty() {
                    continue;
                }
                results.push((
                    page_idx,
                    LyricLabelClickTarget {
                        row: *row_idx,
                        source_part_index: part_template.source_part_index,
                        measure_index_start,
                        measure_index_end,
                    },
                ));
            }
        },
    );
    results
}
