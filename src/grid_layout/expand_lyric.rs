//! Lyric verse row expansion — split out of `expand.rs` to keep it under the
//! max-file-lines lint.

use crate::compiler::types::{BarLineKind, ElementContent, MeasureBlock, MULTI_MEASURE_REST_WIDTH};
use crate::coordinate_resolver::LyricFontSizes;
use crate::grid_layout::layout::{
    block_column_width, lyric_row_height, LyricSizing, LABEL_COLS, MUSIC_START_COL,
};
use crate::grid_layout::types::{
    GridContent, GridElement, GridRow, HAlign, MeasureColumnLayout, VAlign,
};

pub(crate) struct LyricPartParams<'a> {
    pub(crate) part_idx: usize,
    pub(crate) base: f32,
    pub(crate) column_count: u32,
    pub(crate) bar_height: f32,
    pub(crate) measure_layout: &'a [MeasureColumnLayout],
    pub(crate) lyric_sizing: LyricSizing,
}

/// The largest resolved lyric font size (see `RenderConfig::lyric_font_size`/
/// `lyric_cjk_font_size`, mirrored by `font_metrics::lyric_font_size`) among
/// this row's own syllables — a CJK syllable resolves larger than a Latin
/// one, so the row must be sized for whichever one actually appears in it,
/// not just the Latin base size. Falls back to `font_sizes.base` when the
/// row has no lyric elements at all (nothing to measure against).
fn lyric_part_max_font_size(
    system: &[MeasureBlock],
    part_idx: usize,
    font_sizes: LyricFontSizes,
) -> f32 {
    system
        .iter()
        .filter_map(|block| block.rows.get(part_idx))
        .flat_map(|row| &row.elements)
        .filter_map(|el| match &el.content {
            ElementContent::Lyric { text, .. } => Some(crate::font_metrics::lyric_font_size(
                text,
                font_sizes.base,
                font_sizes.cjk,
            )),
            _ => None,
        })
        .fold(f32::MIN, f32::max)
        .max(font_sizes.base)
}

/// Every lyric row gets its own label at column 0: the lyric part's own
/// abbreviation from `parts` (e.g. `v1`). `system.first()` always has a
/// `part_idx` entry matching this row's own template (only ever called for
/// an `is_lyric_row` row). Split out of `expand_lyric_part` to keep it under
/// the max function-length lint.
fn push_verse_row_label(row: &mut GridRow, system: &[MeasureBlock], part_idx: usize) {
    let Some(part_template) = system.first().and_then(|b| b.rows.get(part_idx)) else {
        return;
    };
    if part_template.label.is_empty() {
        return;
    }
    row.elements.push(GridElement {
        column: 0,
        column_span: LABEL_COLS,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::RowLabel(part_template.label.clone()),
    });
}

/// Every other row type only draws bar lines when it's the block's first row
/// (`part_idx == 0`) — a lyric row is no exception, since it can end up
/// first when its target's row is hidden (by the part toggle, or by
/// `hide_resting_parts`). Without this, such a measure would render with no
/// bar line at all. Split out of `expand_lyric_part` to keep it under the max
/// function-length lint.
fn push_leading_bar_line(row: &mut GridRow, part_idx: usize, bar_height: f32) {
    if part_idx != 0 {
        return;
    }
    row.elements.push(GridElement {
        column: LABEL_COLS,
        column_span: 1,
        halign: HAlign::Start,
        valign: VAlign::Top,
        content: GridContent::BarLine {
            height_pt: bar_height,
            kind: BarLineKind::Single,
        },
    });
}

pub(crate) fn expand_lyric_part(system: &[MeasureBlock], params: &LyricPartParams<'_>) -> GridRow {
    let part_idx = params.part_idx;
    let column_count = params.column_count;
    let bar_height = params.bar_height;
    let font_size = lyric_part_max_font_size(system, part_idx, params.lyric_sizing.font_sizes);
    let mut row = GridRow {
        height_pt: lyric_row_height(
            params.base,
            font_size,
            params.lyric_sizing.click_target_padding_pt,
        ),
        column_count,
        has_label_region: true,
        measure_layout: params.measure_layout.to_vec(),
        elements: vec![],
    };
    push_leading_bar_line(&mut row, part_idx, bar_height);
    push_verse_row_label(&mut row, system, part_idx);
    let mut measure_col_offset: u32 = 0;
    let last_block_idx = system.len().saturating_sub(1);
    for (block_idx, block) in system.iter().enumerate() {
        let col_w = block_column_width(block);
        if let Some(part_row) = block.rows.get(part_idx) {
            for el in &part_row.elements {
                match &el.content {
                    ElementContent::Lyric { text, note_id } => {
                        row.elements.push(GridElement {
                            column: MUSIC_START_COL + measure_col_offset + el.column,
                            column_span: 1,
                            halign: HAlign::Center,
                            valign: VAlign::Center,
                            content: GridContent::LyricSyllable {
                                text: text.clone(),
                                source_part_index: part_row.source_part_index,
                                note_id: *note_id,
                            },
                        });
                    }
                    ElementContent::MultiMeasureRest { count } => {
                        row.elements.push(GridElement {
                            column: MUSIC_START_COL + measure_col_offset + el.column,
                            column_span: MULTI_MEASURE_REST_WIDTH,
                            halign: HAlign::Center,
                            valign: VAlign::Center,
                            content: GridContent::MultiMeasureRest {
                                count: *count as u32,
                            },
                        });
                    }
                    ElementContent::BarLine if part_idx == 0 => {
                        let halign = if block_idx == last_block_idx {
                            HAlign::End
                        } else {
                            HAlign::Center
                        };
                        row.elements.push(GridElement {
                            column: MUSIC_START_COL + measure_col_offset + el.column,
                            column_span: 1,
                            halign,
                            valign: VAlign::Top,
                            content: GridContent::BarLine {
                                height_pt: bar_height,
                                kind: BarLineKind::Single,
                            },
                        });
                    }
                    _ => {}
                }
            }
        }
        measure_col_offset += col_w;
    }
    row
}
