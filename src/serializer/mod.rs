use base64::Engine;

use crate::renderer::new_types::{SvgDocument, SvgElement, SvgKind, SvgVariant};

mod group;
mod rect;
mod text;

use group::serialize_group;
use rect::serialize_rect_element;
use text::{serialize_text, serialize_text_with_tspans, TextWithTspansStyle};

pub fn serialize(documents: &[SvgDocument], source: Option<&str>) -> Vec<String> {
    documents
        .iter()
        .map(|doc| serialize_doc(doc, source))
        .collect()
}

fn serialize_doc(doc: &SvgDocument, source: Option<&str>) -> String {
    let mut body = String::new();
    if let Some(source) = source {
        body.push_str(&format!(
            r#"<metadata id="poentaro-source">{}</metadata>"#,
            base64::engine::general_purpose::STANDARD.encode(source)
        ));
    }
    for el in &doc.elements {
        serialize_element(el, &mut body);
    }
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="210mm" height="297mm" viewBox="0 0 {:.0} {:.0}">{}</svg>"#,
        doc.width_pt, doc.height_pt, body
    )
}

pub(super) fn variant_attr(variant: Option<SvgVariant>) -> String {
    variant
        .map(|variant| format!(r#" data-variant="{}""#, variant.as_str()))
        .unwrap_or_default()
}

fn serialize_element(el: &SvgElement, out: &mut String) {
    match &el.kind {
        SvgKind::Text { .. } => serialize_text(el, out, &el.kind),
        SvgKind::Line {
            x2,
            y2,
            stroke_width,
        } => {
            out.push_str(&format!(
                r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}"{} stroke="black" stroke-width="{:.1}"/>"#,
                el.x,
                el.y,
                x2,
                y2,
                variant_attr(el.variant),
                stroke_width
            ));
        }
        SvgKind::Circle { r } => {
            out.push_str(&format!(
                r#"<circle cx="{:.1}" cy="{:.1}"{} r="{:.1}" fill="black"/>"#,
                el.x,
                el.y,
                variant_attr(el.variant),
                r
            ));
        }
        SvgKind::RingCircle { .. } | SvgKind::RoundedBar { .. } => {
            serialize_guitar_diagram_shape(el, out);
        }
        SvgKind::Path {
            control_x,
            control_y,
            end_x,
            end_y,
            stroke_width,
        } => {
            out.push_str(&format!(
                r#"<path d="M {:.1} {:.1} Q {:.1} {:.1} {:.1} {:.1}"{} fill="none" stroke="black" stroke-width="{:.1}"/>"#,
                el.x,
                el.y,
                control_x,
                control_y,
                end_x,
                end_y,
                variant_attr(el.variant),
                stroke_width
            ));
        }
        SvgKind::Rect { .. }
        | SvgKind::ErrorRect { .. }
        | SvgKind::PlaybackCursorRect { .. }
        | SvgKind::TransparentRect { .. } => serialize_rect_element(el, out, &el.kind),
        SvgKind::TextWithTspans {
            font_size,
            anchor,
            baseline,
            font,
            spans,
        } => serialize_text_with_tspans(
            el,
            out,
            TextWithTspansStyle {
                font_size: *font_size,
                anchor,
                baseline,
                font: *font,
            },
            spans,
        ),
        SvgKind::Group { children, tag } => serialize_group(out, children, tag),
    }
}

/// The two shapes only guitar chord diagrams use (an open string's ring and
/// a barre), split out of `serialize_element` to keep it short.
fn serialize_guitar_diagram_shape(el: &SvgElement, out: &mut String) {
    match &el.kind {
        SvgKind::RingCircle { r, stroke_width } => {
            out.push_str(&format!(
                r#"<circle cx="{:.2}" cy="{:.2}"{} r="{:.2}" fill="none" stroke="black" stroke-width="{:.2}"/>"#,
                el.x,
                el.y,
                variant_attr(el.variant),
                r,
                stroke_width
            ));
        }
        SvgKind::RoundedBar { width, height } => {
            out.push_str(&format!(
                r#"<rect x="{:.2}" y="{:.2}"{} width="{:.2}" height="{:.2}" rx="{:.2}" fill="black"/>"#,
                el.x,
                el.y,
                variant_attr(el.variant),
                width,
                height,
                height * 0.5
            ));
        }
        _ => {}
    }
}

pub(super) fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests;
