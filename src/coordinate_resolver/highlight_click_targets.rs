use super::page_geometry::PageGeometry;
use crate::compositor::types::{AbsoluteContent, AbsoluteElement};
use crate::grid_layout::types::GridRow;

use super::highlights::{resolve_row_range_geometry, RowLayoutContext};

/// Uses `target.click_row_end` rather than `target.row_end` — unlike the
/// playback cursor rect in `highlights`, a note's own click/selection target
/// never extends down into a following lyric verse row (a lyric syllable has
/// its own independent [`crate::grid_layout::types::LyricClickTarget`]); see
/// `PlaybackCursorTarget::click_row_end`'s doc comment.
pub(super) fn resolve_note_click_target(
    target: &crate::grid_layout::types::PlaybackCursorTarget,
    rows: &[GridRow],
    row_tops: &[f32],
    page_geometry: PageGeometry,
) -> Option<AbsoluteElement> {
    let ctx = RowLayoutContext {
        rows,
        row_tops,
        page_geometry,
    };
    let geometry = resolve_row_range_geometry(
        target.row_start,
        target.click_row_end,
        target.column_start,
        target.column_end,
        ctx,
    )?;
    Some(AbsoluteElement {
        x: geometry.x,
        y: geometry.y,
        content: AbsoluteContent::NoteClickTarget {
            width: geometry.width,
            height: geometry.height,
            source_part_index: target.source_part_index,
            note_id: target.note_id,
        },
    })
}

pub(super) fn resolve_measure_click_target(
    target: &crate::grid_layout::types::MeasureClickTarget,
    rows: &[GridRow],
    row_tops: &[f32],
    page_geometry: PageGeometry,
) -> Option<AbsoluteElement> {
    let ctx = RowLayoutContext {
        rows,
        row_tops,
        page_geometry,
    };
    let geometry = resolve_row_range_geometry(
        target.row_start,
        target.row_end,
        target.column_start,
        target.column_end,
        ctx,
    )?;
    Some(AbsoluteElement {
        x: geometry.x,
        y: geometry.y,
        content: AbsoluteContent::MeasureClickTarget {
            width: geometry.width,
            height: geometry.height,
            measure_index: target.measure_index,
            measure_index_end: target.measure_index_end,
        },
    })
}

/// A digit's own glyph metrics (e.g. a single "1" at font-size 10) make for
/// an unusably thin hit box — a few points wide is smaller than a cursor can
/// reliably land on. This pads [`resolve_bar_number_click_target`]'s rect
/// out on both sides so the box stays comfortably hoverable/clickable even
/// for a one- or two-digit bar number.
const BAR_NUMBER_CLICK_TARGET_HORIZONTAL_PADDING: f32 = 6.0;

/// Sibling to [`resolve_measure_click_target`] for one measure's own bar
/// number (see [`crate::grid_layout::types::BarNumberClickTarget`]): unlike
/// every other row-range target above, its width isn't a `column_start`/
/// `column_end` bound — the bar number is a small, precisely-positioned text
/// element within its (single, exact) grid column, not a whole measure body
/// — so its width is measured here from the identical `TextSpan`
/// `AbsoluteContent::DirectiveLine::bar_number` renders (see
/// `content_conversion::bar_number_text_span`), the same way the renderer
/// itself lays that text out, then padded on both sides (see
/// `BAR_NUMBER_CLICK_TARGET_HORIZONTAL_PADDING`) so the digits' own glyph
/// width alone doesn't leave an unusably thin hit box.
pub(super) fn resolve_bar_number_click_target(
    target: &crate::grid_layout::types::BarNumberClickTarget,
    ctx: &RowLayoutContext,
    measure_number_font_size: f32,
    measure_number_style: crate::grid_layout::types::TextStyleFlags,
) -> Option<AbsoluteElement> {
    let row = ctx.rows.get(target.row)?;
    let y = *ctx.row_tops.get(target.row)?;
    let geometry = ctx.page_geometry.row_geometry(row);
    let x = geometry.x_start(target.column as f32);
    let digits_width =
        crate::font_metrics::span_width(&super::content_conversion::bar_number_text_span(
            target.measure_index as u32 + 1,
            measure_number_font_size,
            measure_number_style.bold,
            measure_number_style.italic,
            measure_number_style.underline,
        ));
    Some(AbsoluteElement {
        x: x - BAR_NUMBER_CLICK_TARGET_HORIZONTAL_PADDING,
        y,
        content: AbsoluteContent::BarNumberClickTarget {
            width: digits_width + BAR_NUMBER_CLICK_TARGET_HORIZONTAL_PADDING * 2.0,
            height: row.height_pt,
            measure_index: target.measure_index,
            measure_index_end: target.measure_index_end,
        },
    })
}

/// Fixed hit width (in points) for a bar line's click target — the Rust-side
/// replacement for the old TS-only `BAR_LINE_HIT_WIDTH` pixel constant, now
/// that the bar line's own identity and geometry both live here.
const BAR_LINE_CLICK_TARGET_WIDTH_PT: f32 = 6.0;

/// Sibling to [`resolve_measure_click_target`] for one bar line (see
/// [`crate::grid_layout::types::BarLineClickTarget`]): unlike every row-range
/// target above, its width isn't derived from `column_start`/`column_end` —
/// a bar line is a single grid column, not a span — so its rect is centered
/// on `target.column`'s own x position and padded out to a fixed width
/// (`BAR_LINE_CLICK_TARGET_WIDTH_PT`) so a real cursor doesn't have to land
/// on the exact boundary pixel.
pub(super) fn resolve_bar_line_click_target(
    target: &crate::grid_layout::types::BarLineClickTarget,
    rows: &[GridRow],
    row_tops: &[f32],
    page_geometry: PageGeometry,
) -> Option<AbsoluteElement> {
    let start_row = rows.get(target.row_start)?;
    let y = *row_tops.get(target.row_start)?;
    if target.row_end >= rows.len() {
        return None;
    }
    let geometry = page_geometry.row_geometry(start_row);
    let x_center = geometry.x_start(target.column);
    let height = rows
        .get(target.row_start..=target.row_end)
        .map(|slice| slice.iter().map(|row| row.height_pt).sum())
        .unwrap_or(0.0);
    Some(AbsoluteElement {
        x: x_center - BAR_LINE_CLICK_TARGET_WIDTH_PT / 2.0,
        y,
        content: AbsoluteContent::BarLineClickTarget {
            width: BAR_LINE_CLICK_TARGET_WIDTH_PT,
            height,
            measure_index_next: target.measure_index_next,
            measure_index_prev: target.measure_index_prev,
        },
    })
}

/// Same row-bounds math as the click targets above, but fixed to the
/// label region (columns `0..LABEL_COLS`) rather than a `column_start`/
/// `column_end` the target itself carries — see `AbsoluteContent::PartLabelClickTarget`.
pub(super) fn resolve_part_label_click_target(
    target: &crate::grid_layout::types::PartLabelClickTarget,
    rows: &[GridRow],
    row_tops: &[f32],
    page_geometry: PageGeometry,
) -> Option<AbsoluteElement> {
    let ctx = RowLayoutContext {
        rows,
        row_tops,
        page_geometry,
    };
    let geometry = resolve_row_range_geometry(
        target.row_start,
        target.row_end,
        0.0,
        crate::grid_layout::layout::LABEL_COLS as f32,
        ctx,
    )?;
    Some(AbsoluteElement {
        x: geometry.x,
        y: geometry.y,
        content: AbsoluteContent::PartLabelClickTarget {
            width: geometry.width,
            height: geometry.height,
            source_part_index: target.source_part_index,
            measure_index_start: target.measure_index_start,
            measure_index_end: target.measure_index_end,
        },
    })
}

/// Same fixed label-region geometry as `resolve_part_label_click_target`,
/// but for exactly one verse row (`target.row..=target.row`) rather than a
/// part's whole span of sub-rows — the lyric-side mirror of that function.
pub(super) fn resolve_lyric_label_click_target(
    target: &crate::grid_layout::types::LyricLabelClickTarget,
    rows: &[GridRow],
    row_tops: &[f32],
    page_geometry: PageGeometry,
) -> Option<AbsoluteElement> {
    let ctx = RowLayoutContext {
        rows,
        row_tops,
        page_geometry,
    };
    let geometry = resolve_row_range_geometry(
        target.row,
        target.row,
        0.0,
        crate::grid_layout::layout::LABEL_COLS as f32,
        ctx,
    )?;
    Some(AbsoluteElement {
        x: geometry.x,
        y: geometry.y,
        content: AbsoluteContent::LyricLabelClickTarget {
            width: geometry.width,
            height: geometry.height,
            source_part_index: target.source_part_index,
            measure_index_start: target.measure_index_start,
            measure_index_end: target.measure_index_end,
        },
    })
}

/// A lyric syllable's click target is simpler than the targets above: it
/// always sits on exactly one row (`target.row`) and one grid column
/// (`target.column_start..target.column_end`), with no bar-line-snapping —
/// a syllable never touches a bar line — and no bounds check against
/// `rows.len()`, so it doesn't go through [`resolve_row_range_geometry`].
pub(super) fn resolve_lyric_click_target(
    target: &crate::grid_layout::types::LyricClickTarget,
    rows: &[GridRow],
    row_tops: &[f32],
    page_geometry: PageGeometry,
) -> Option<AbsoluteElement> {
    let row = rows.get(target.row)?;
    let target_y = row_tops.get(target.row)?;
    let geometry = page_geometry.row_geometry(row);
    let target_x = geometry.x_start(target.column_start);
    let target_width = geometry.x_start(target.column_end) - geometry.x_start(target.column_start);
    Some(AbsoluteElement {
        x: target_x,
        y: *target_y,
        content: AbsoluteContent::LyricClickTarget {
            width: target_width,
            height: row.height_pt,
            source_part_index: target.source_part_index,
            note_id: target.note_id,
        },
    })
}
