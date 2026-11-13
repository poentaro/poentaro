use super::glyph_weight;
use crate::compositor::types::{AbsoluteContent, AbsoluteElement};
use crate::compositor::types::{DominantBaseline, FontFamily, TextAnchor};
use crate::renderer::new_renderer::GlyphStyle;
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

pub(in crate::renderer::new_renderer) fn render_lyric(
    elem: &AbsoluteElement,
    s: &str,
    base_font_size: &f32,
    cjk_font_size: &f32,
    style: GlyphStyle,
    font_family: FontFamily,
) -> Vec<SvgElement> {
    let font_size = crate::font_metrics::lyric_font_size(s, *base_font_size, *cjk_font_size);
    let ruby = match &elem.content {
        AbsoluteContent::Lyric { ruby, .. } => ruby.as_deref(),
        _ => None,
    };
    let ruby = ruby.map(|ruby| {
        let ruby_font_size = crate::font_metrics::lyric_ruby_font_size(*cjk_font_size);
        SvgElement {
            // Centred over the syllable, just above its top.
            x: elem.x + crate::font_metrics::text_width_for_family(font_family, s, font_size) * 0.5,
            y: elem.y
                - font_size * 0.5
                - crate::font_metrics::lyric_ruby_gap_pt(*cjk_font_size)
                - ruby_font_size * 0.5,
            variant: Some(SvgVariant::Lyric),
            kind: SvgKind::Text {
                content: ruby.to_string(),
                font_size: ruby_font_size,
                anchor: TextAnchor::Middle,
                baseline: DominantBaseline::Middle,
                font: font_family,
                weight: glyph_weight(style.bold),
                italic: style.italic,
                underline: false,
            },
        }
    });
    let syllable = SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Lyric),
        kind: SvgKind::Text {
            content: s.to_string(),
            font_size,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            // Defaults to `FontFamily::Serif` — despite the name, shared with
            // the song title's font, not exclusive to it (see its doc comment
            // in `src/compositor/types.rs`) — but overridable via
            // `Metadata::lyrics.font_family`.
            font: font_family,
            weight: glyph_weight(style.bold),
            italic: style.italic,
            underline: style.underline,
        },
    };
    std::iter::once(syllable).chain(ruby).collect()
}

/// Renders a standalone `lyrics` part's whole verse line, left-aligned at
/// `elem.x` (the measure's left edge), since it spans the full measure width
/// instead of being positioned per note like [`render_lyric`].
pub(in crate::renderer::new_renderer) fn render_lyric_line(
    elem: &AbsoluteElement,
    s: &str,
    base_font_size: &f32,
    cjk_font_size: &f32,
    style: GlyphStyle,
    font_family: FontFamily,
) -> Vec<SvgElement> {
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Lyric),
        kind: SvgKind::Text {
            content: s.to_string(),
            font_size: crate::font_metrics::lyric_font_size(s, *base_font_size, *cjk_font_size),
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            font: font_family,
            weight: glyph_weight(style.bold),
            italic: style.italic,
            underline: style.underline,
        },
    }]
}
