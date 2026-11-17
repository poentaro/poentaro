use crate::compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor};
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant, TspanData};

use super::{escape_xml, variant_attr};

/// The red an unplayable note head (`SvgVariant::UnplayableNoteHead`) is
/// drawn in; the web preview styles its `data-variant` the same.
const UNPLAYABLE_FILL: &str = "#d32f2f";

pub(super) fn serialize_text(el: &SvgElement, out: &mut String, kind: &SvgKind) {
    let SvgKind::Text {
        content,
        font_size,
        anchor,
        baseline,
        font,
        weight,
        italic,
        underline,
    } = kind
    else {
        return;
    };
    let anchor_str = match anchor {
        TextAnchor::Start => "start",
        TextAnchor::Middle => "middle",
        TextAnchor::End => "end",
    };
    let baseline_str = match baseline {
        DominantBaseline::Middle => "middle",
        DominantBaseline::Hanging => "hanging",
        DominantBaseline::Ideographic => "ideographic",
    };
    let font_str = font_family_css(*font);
    let weight_str = match weight {
        FontWeight::Normal => "normal",
        FontWeight::Bold => "bold",
    };
    let style_str = if *italic {
        "font-style=\"italic\" "
    } else {
        ""
    };
    let decoration_str = if *underline {
        "text-decoration=\"underline\" "
    } else {
        ""
    };
    let fill_str = match el.variant {
        Some(SvgVariant::UnplayableNoteHead) => format!("fill=\"{UNPLAYABLE_FILL}\" "),
        _ => String::new(),
    };
    out.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}"{} font-size="{:.1}" text-anchor="{}" dominant-baseline="{}" font-family='{}' font-weight="{}" {}{}{}>{}</text>"#,
        el.x,
        el.y,
        variant_attr(el.variant),
        font_size,
        anchor_str,
        baseline_str,
        font_str,
        weight_str,
        style_str,
        decoration_str,
        fill_str,
        escape_xml(content)
    ));
}

/// Every `FontFamily::SansSerif` glyph (the directive line's bar number,
/// section label, key/bpm/time signature, navigation markers, and
/// footer) is pinned to this concrete font family — the same one PDF
/// export already resolves `sans-serif` to (see `set_sans_serif_family` in
/// `src/pdf.rs`) — rather than the generic `sans-serif` alias, so glyph
/// widths are consistent between viewers that have this font installed and
/// the PDF export path — see Task 1 of `PLAN-section-label-engraving-quality.md`.
/// Defined in `src/fonts.rs`, the single source of truth for which font
/// backs each `FontFamily` role.
use crate::fonts::SANS_SERIF_FONT_FAMILY_CSS as DIRECTIVE_LINE_FONT_FAMILY;

/// `FontFamily::Serif` — the song title, subtitle, and author (via
/// `make_title_row`/`make_subtitle_author_row`) and lyric syllables/lines
/// (`render_lyric`/`render_lyric_line`) and the part legend — is pinned to whichever font backs
/// the `serif` role in `fonts/fonts.json` (currently Zhuque Fangsong, a
/// calligraphic font kept off the directive line/footer, where
/// its Latin glyphs would read too calligraphic — see `DIRECTIVE_LINE_FONT_FAMILY`'s
/// Source Han Sans SC above). Kept as a separate constant from
/// `DIRECTIVE_LINE_FONT_FAMILY` rather than merged into one, since the two
/// roles are backed by different files. Defined in `src/fonts.rs`.
use crate::fonts::SERIF_FONT_FAMILY_CSS as SERIF_FONT_FAMILY;

/// Every `FontFamily::Monospace` glyph (notehead, rest, chord symbol,
/// percussion, multi-measure-rest count, note dash, Latin lyric) is pinned to
/// this concrete family so raw-SVG viewers render at the same width measured
/// by `font_metrics::monospace_text_width`/`monospace_char_advance_width`,
/// mirroring `DIRECTIVE_LINE_FONT_FAMILY` above. Defined in `src/fonts.rs`.
use crate::fonts::MONOSPACE_FONT_FAMILY_CSS as MONOSPACE_FONT_FAMILY;

/// Maps a resolved [`FontFamily`] role onto the literal CSS `font-family`
/// value that backs it (see the constants above) — shared by [`serialize_text`]
/// and [`serialize_text_with_tspans`].
pub(super) fn font_family_css(font: FontFamily) -> &'static str {
    match font {
        FontFamily::Monospace => MONOSPACE_FONT_FAMILY,
        FontFamily::SansSerif => DIRECTIVE_LINE_FONT_FAMILY,
        FontFamily::Serif => SERIF_FONT_FAMILY,
    }
}

/// Bundles [`serialize_text_with_tspans`]'s per-element style params — split
/// out once `font` pushed the plain argument list over clippy's
/// `too_many_arguments` limit.
#[derive(Clone, Copy)]
pub(super) struct TextWithTspansStyle<'a> {
    pub(super) font_size: f32,
    pub(super) anchor: &'a TextAnchor,
    pub(super) baseline: &'a DominantBaseline,
    pub(super) font: FontFamily,
}

pub(super) fn serialize_text_with_tspans(
    el: &SvgElement,
    out: &mut String,
    style: TextWithTspansStyle,
    spans: &[TspanData],
) {
    let anchor_str = match style.anchor {
        TextAnchor::Start => "start",
        TextAnchor::Middle => "middle",
        TextAnchor::End => "end",
    };
    let baseline_str = match style.baseline {
        DominantBaseline::Middle => "middle",
        DominantBaseline::Hanging => "hanging",
        DominantBaseline::Ideographic => "ideographic",
    };
    out.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}"{} font-size="{:.1}" text-anchor="{}" dominant-baseline="{}" font-family='{}'>"#,
        el.x,
        el.y,
        variant_attr(el.variant),
        style.font_size,
        anchor_str,
        baseline_str,
        font_family_css(style.font)
    ));
    for span in spans {
        let mut attrs = String::new();
        if span.bold {
            attrs.push_str(r#" font-weight="bold""#);
        }
        if span.italic {
            attrs.push_str(r#" font-style="italic""#);
        }
        if span.underline {
            attrs.push_str(r#" text-decoration="underline""#);
        }
        if let Some(fs) = span.font_size {
            attrs.push_str(&format!(r#" font-size="{fs:.1}""#));
        }
        out.push_str(&format!(
            "<tspan{}>{}</tspan>",
            attrs,
            escape_xml(&span.content)
        ));
    }
    out.push_str("</text>");
}
