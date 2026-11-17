//! Pixel resolution for the two `GridContent` shapes that stand in for a
//! *run* of rests rather than a single written one: a collapsed
//! `MultiMeasureRest` bar, and a consolidated `implicit_fill` whole-rest
//! (see `expand_elements::push_implicit_fill_rest`). Split out of
//! `resolve.rs` to keep that file under the line-count cap.

use crate::compositor::types::{AbsoluteContent, AbsoluteElement};
use crate::grid_layout::types::ColumnGeometry;

/// The collapsed multi-measure-rest bar spans its custom column_span width
/// starting at the column's left edge, rather than the generic per-column
/// halign/valign math above — but inset by `padding` (the configured
/// `notes_horizontal_padding_pt`) on both ends, mirroring the same clearance
/// every other note-ish column keeps before whatever follows it, so the
/// bar's own end ticks don't render flush against the enclosing measure
/// dividers. `layout_spacing::multi_measure_rest_weight` reserves this same
/// padding on both ends when sizing the block's column span, so the two
/// can't drift apart.
pub(super) fn resolve_multi_measure_rest(
    count: u32,
    x_start: f32,
    width: f32,
    y: f32,
    padding: f32,
) -> AbsoluteElement {
    AbsoluteElement {
        x: x_start + padding,
        y,
        content: AbsoluteContent::MultiMeasureRest {
            count,
            width: (width - padding * 2.0).max(0.0),
        },
    }
}

/// An `implicit_fill` rest's `column` / `column_span` (see
/// `expand_elements::push_implicit_fill_rest`) run exactly from the bar line
/// opening the measure to the one closing it — but a bar line's own
/// rendered position sits at the *center* of its column, not its edge (see
/// `GridContent::BarLine`'s `HAlign::Center`), so the run's true left/right
/// bounds are each half a column further out than `column` /
/// `column + column_span`. `ColumnGeometry::x_start` accepts a fractional
/// column for exactly this (see its doc comment), so `± 0.5` reaches those
/// bar line centers directly. Centering the glyph between them (rather than
/// flush-left-anchoring it at the run's first column, like an ordinary
/// written rest) puts it at the center of the whole measure, matching the
/// conventional Western whole-rest engraving, regardless of how many beats
/// the run collapsed. Slightly off (by half a column) at a system's very
/// first or last measure, whose opening/closing bar line is `Start`/`End`
/// aligned instead of `Center` — not worth threading that distinction
/// through just for this cosmetic centering.
pub(super) fn resolve_implicit_fill_rest(
    dotted: bool,
    double_dotted: bool,
    column: u32,
    column_span: u32,
    geometry: &ColumnGeometry,
    y: f32,
) -> AbsoluteElement {
    let left = geometry.x_start(column as f32 - 0.5);
    let right = geometry.x_start(column as f32 + column_span as f32 + 0.5);
    AbsoluteElement {
        x: (left + right) * 0.5,
        y,
        content: AbsoluteContent::Rest {
            dotted,
            double_dotted,
            implicit_fill: true,
        },
    }
}
