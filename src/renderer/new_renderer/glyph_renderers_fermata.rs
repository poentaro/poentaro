use super::dot_glyph;
use crate::compositor::types::AbsoluteElement;
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

pub(in crate::renderer::new_renderer) fn render_fermata(
    elem: &AbsoluteElement,
    octave: i8,
    size: f32,
    width: f32,
) -> [SvgElement; 2] {
    let center = elem.x + width * 0.5;
    let y = elem.y - size * (0.68 + 0.3 * f32::from(octave.max(0)));
    let half_width = size * 0.30;
    [
        SvgElement {
            x: center - half_width,
            y,
            variant: Some(SvgVariant::Fermata),
            kind: SvgKind::Path {
                control_x: center,
                control_y: y - size * 0.45,
                end_x: center + half_width,
                end_y: y,
                stroke_width: size * 0.055,
            },
        },
        dot_glyph(center, y, size * 0.07, SvgVariant::Fermata),
    ]
}
