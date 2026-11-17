use crate::compositor::types::{AbsoluteContent, AbsoluteElement};
use crate::grid_layout::types::{ColumnGeometry, GridContent, GridElement};

use super::resolve::RowResolveConfig;

/// The glyph anchor of a span's last column, mirroring `start_center`'s
/// `geometry.glyph_left_anchor_x(el.column as f32, ...)` but for `el.column +
/// el.column_span - 1`.
fn span_end_center(geometry: &ColumnGeometry, el: &GridElement, padding: f32) -> f32 {
    geometry.glyph_left_anchor_x(el.column as f32 + el.column_span as f32 - 1.0, padding)
}

/// Handles the underline/tie/slur variants, whose x-extent is defined in
/// terms of column centers/edges rather than the halign/valign math above.
/// Returns `None` for every other `GridContent` variant.
pub(super) fn resolve_span_marking(
    el: &GridElement,
    y: f32,
    geometry: &ColumnGeometry,
    config: RowResolveConfig,
) -> Option<AbsoluteElement> {
    // A span marking's own glyph anchor keys off the note's *center*, unlike
    // the flush-left glyphs above (which draw `TextAnchor::Start` at exactly
    // `padding - bearing`, see `flush_left_padding`). A span can cover notes of differing
    // pitches/widths, so per-glyph bearing tracking isn't practical here;
    // this approximates the note's center with the same flat
    // `note_number_width` nominal box the renderer itself uses (see
    // `center` in `glyph_renderers.rs::render_note_head`).
    let padding = config.paddings.notes + config.note_number_width * 0.5;
    match &el.content {
        GridContent::Underline { level } => {
            let start_center = geometry.glyph_left_anchor_x(el.column as f32, padding);
            let end_center = span_end_center(geometry, el, padding);
            // The half-`note_number_width` pad on each end assumes there's a
            // neighboring note column to bleed into, same as any other note
            // glyph. That's not true at a measure boundary — the column just
            // past the span may belong to a `BarLine`, whose rod is far
            // narrower than a note's — so clamp each end to the span's own
            // column edges (`geometry.x_start`) rather than let the pad
            // overshoot into whatever sits next door.
            let span_left = geometry.x_start(el.column as f32);
            let span_right = geometry.x_start(el.column as f32 + el.column_span as f32);
            let ul_x = (start_center - config.note_number_width * 0.5).max(span_left);
            let ul_right = (end_center + config.note_number_width * 0.5).min(span_right);
            Some(AbsoluteElement {
                x: ul_x,
                y,
                content: AbsoluteContent::Underline {
                    width: ul_right - ul_x,
                    level: *level,
                },
            })
        }
        GridContent::TieOrSlur { kind } => {
            let start_center = geometry.glyph_left_anchor_x(el.column as f32, padding);
            let end_center = span_end_center(geometry, el, padding);
            Some(AbsoluteElement {
                x: start_center,
                y,
                content: AbsoluteContent::TieOrSlur {
                    kind: kind.clone(),
                    width: end_center - start_center,
                },
            })
        }
        GridContent::TieOrSlurTail { kind } => {
            let start_center = geometry.glyph_left_anchor_x(el.column as f32, padding);
            let system_right_edge = geometry.x_start(el.column as f32 + el.column_span as f32);
            Some(AbsoluteElement {
                x: start_center,
                y,
                content: AbsoluteContent::TieOrSlur {
                    kind: kind.clone(),
                    width: system_right_edge - start_center,
                },
            })
        }
        GridContent::TieOrSlurHead { kind } => {
            let system_left_edge = geometry.x_start(el.column as f32);
            let end_center = span_end_center(geometry, el, padding);
            Some(AbsoluteElement {
                x: system_left_edge,
                y,
                content: AbsoluteContent::TieOrSlur {
                    kind: kind.clone(),
                    width: end_center - system_left_edge,
                },
            })
        }
        GridContent::TupletBracket { label } => {
            let start_center = geometry.glyph_left_anchor_x(el.column as f32, padding);
            let end_center = span_end_center(geometry, el, padding);
            Some(AbsoluteElement {
                x: start_center,
                y,
                content: AbsoluteContent::TupletBracket {
                    label: label.clone(),
                    width: end_center - start_center,
                },
            })
        }
        _ => None,
    }
}
