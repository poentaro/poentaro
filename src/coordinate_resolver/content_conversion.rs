use crate::compositor::types::{
    AbsoluteContent, DominantBaseline, FontFamily, FontWeight, TextAnchor, TextSpan,
};
use crate::error::IrrecoverableError;
use crate::grid_layout::types::{HAlign, PostArcGridContent};

use super::directive_line_conversion::{
    directive_line_absolute, sequence_line_content, DirectiveLineFontSizes,
};

fn text_anchor(halign: HAlign) -> TextAnchor {
    match halign {
        HAlign::Start => TextAnchor::Start,
        HAlign::Center => TextAnchor::Middle,
        HAlign::End => TextAnchor::End,
    }
}

/// Bundles `sans_serif_text`'s style params — split out once a 6th field
/// pushed the plain argument list over clippy's `too_many_arguments` limit.
#[derive(Clone, Copy)]
struct SansSerifTextStyle {
    font_size: f32,
    anchor: TextAnchor,
    weight: FontWeight,
    italic: bool,
    underline: bool,
    font: FontFamily,
}

fn sans_serif_text(content: String, style: SansSerifTextStyle) -> AbsoluteContent {
    AbsoluteContent::Text {
        content,
        font_size: style.font_size,
        anchor: style.anchor,
        baseline: DominantBaseline::Middle,
        font: style.font,
        weight: style.weight,
        italic: style.italic,
        underline: style.underline,
    }
}

/// Builds the text span for a section label (bold + italic), matching how a
/// `label="..."` directive is rendered inline on a measure. Used by the
/// `sequence` header line at `font_size` (see `Metadata::sequence_font_size`);
/// the directive line's own label is rendered as an independent text element
/// (see [`AbsoluteContent::DirectiveLine`], `label`/`label_x_offset`) rather
/// than through this span.
pub(super) fn section_label_span(
    label_text: &str,
    font_size: f32,
    bold: bool,
    italic: bool,
    underline: bool,
) -> TextSpan {
    TextSpan {
        content: label_text.to_string(),
        bold,
        italic,
        underline,
        font_size,
    }
}

/// The bar-number `TextSpan` a `DirectiveLine`'s `bar_number` renders as —
/// shared with `highlight_click_targets::resolve_bar_number_click_target`, which needs
/// the identical span (content and `font_size`) to measure the same click
/// target's width via `font_metrics::span_width`.
pub(super) fn bar_number_text_span(
    n: u32,
    font_size: f32,
    bold: bool,
    italic: bool,
    underline: bool,
) -> TextSpan {
    TextSpan {
        content: n.to_string(),
        bold,
        italic,
        underline,
        font_size,
    }
}

/// Bundles [`grid_text_to_absolute`]/[`grid_to_absolute`]'s part-label style
/// params — split out once `RowLabel`'s bold/italic/underline joined
/// `part_label_font_size`, pushing the plain argument list over clippy's
/// `too_many_arguments` limit.
#[derive(Clone, Copy)]
pub(super) struct PartLabelStyle {
    pub(super) font_size: f32,
    pub(super) bold: bool,
    pub(super) italic: bool,
    pub(super) underline: bool,
    pub(super) font_family: FontFamily,
}

fn grid_text_to_absolute(
    content: &PostArcGridContent,
    span_width: f32,
    halign: HAlign,
    part_label_style: PartLabelStyle,
    directive_font_sizes: DirectiveLineFontSizes,
) -> Option<AbsoluteContent> {
    match content {
        PostArcGridContent::NoteDash {
            dotted,
            double_dotted,
        } => Some(AbsoluteContent::NoteDash {
            dotted: *dotted,
            double_dotted: *double_dotted,
        }),
        PostArcGridContent::RowLabel(s) => Some(sans_serif_text(
            s.clone(),
            SansSerifTextStyle {
                font_size: part_label_style.font_size,
                anchor: TextAnchor::Middle,
                weight: if part_label_style.bold {
                    FontWeight::Bold
                } else {
                    FontWeight::Normal
                },
                italic: part_label_style.italic,
                underline: part_label_style.underline,
                font: part_label_style.font_family,
            },
        )),
        PostArcGridContent::DirectiveLine { label, .. } => Some(directive_line_absolute(
            content,
            label,
            directive_font_sizes,
        )),
        PostArcGridContent::Text {
            content,
            font_size,
            bold,
            italic,
            underline,
            font_family,
        } => Some(sans_serif_text(
            content.clone(),
            SansSerifTextStyle {
                font_size: *font_size,
                anchor: text_anchor(halign),
                weight: if *bold {
                    FontWeight::Bold
                } else {
                    FontWeight::Normal
                },
                italic: *italic,
                underline: *underline,
                font: *font_family,
            },
        )),
        PostArcGridContent::HorizontalLine => {
            Some(AbsoluteContent::HorizontalLine { width: span_width })
        }
        PostArcGridContent::SequenceLine { entries, font_size } => {
            Some(AbsoluteContent::DirectiveLine {
                bar_number: None,
                bar_number_font_family: FontFamily::SansSerif,
                label: None,
                label_font_size: *font_size,
                label_bold: false,
                label_italic: false,
                label_underline: false,
                label_font_family: FontFamily::SansSerif,
                label_box_height: 0.0,
                spans: sequence_line_content(entries, *font_size, directive_font_sizes),
                spans_font_family: directive_font_sizes.sequence_font_family,
                spans_x_offset: 0.0,
                label_x_offset: 0.0,
                apply_row_offset: false,
            })
        }
        _ => None,
    }
}

pub(super) fn grid_to_absolute(
    content: &PostArcGridContent,
    span_width: f32,
    halign: HAlign,
    part_label_style: PartLabelStyle,
    directive_font_sizes: DirectiveLineFontSizes,
) -> Result<Option<AbsoluteContent>, IrrecoverableError> {
    if let Some(content) = grid_text_to_absolute(
        content,
        span_width,
        halign,
        part_label_style,
        directive_font_sizes,
    ) {
        return Ok(Some(content));
    }

    Ok(match content {
        PostArcGridContent::NoteHead {
            pitch,
            accidental,
            octave,
            dotted,
            double_dotted,
            head_label,
        } => Some(AbsoluteContent::NoteHead {
            pitch: pitch.clone(),
            accidental: accidental.clone(),
            octave: *octave,
            dotted: *dotted,
            double_dotted: *double_dotted,
            head_label: head_label.clone(),
        }),
        PostArcGridContent::Rest {
            dotted,
            double_dotted,
            implicit_fill,
        } => Some(AbsoluteContent::Rest {
            dotted: *dotted,
            double_dotted: *double_dotted,
            implicit_fill: *implicit_fill,
        }),
        PostArcGridContent::MultiMeasureRest { count } => Some(AbsoluteContent::MultiMeasureRest {
            count: *count,
            width: span_width,
        }),
        PostArcGridContent::LowOctaveDots { count } => {
            Some(AbsoluteContent::LowOctaveDots { count: *count })
        }
        PostArcGridContent::ChordSymbol {
            text,
            dotted,
            double_dotted,
        } => Some(AbsoluteContent::ChordSymbol {
            text: text.clone(),
            dotted: *dotted,
            double_dotted: *double_dotted,
        }),
        PostArcGridContent::GuitarDiagram { voicing } => Some(AbsoluteContent::GuitarDiagram {
            voicing: voicing.clone(),
        }),
        PostArcGridContent::PercussionHit => Some(AbsoluteContent::PercussionHit),
        PostArcGridContent::Underline { level } => Some(AbsoluteContent::Underline {
            width: span_width,
            level: *level,
        }),
        PostArcGridContent::BarLine { height_pt, kind } => Some(AbsoluteContent::BarLine {
            height: *height_pt,
            kind: *kind,
        }),
        PostArcGridContent::LyricSyllable {
            text,
            source_part_index,
            note_id,
        } => Some(AbsoluteContent::Lyric {
            text: text.clone(),
            source_part_index: *source_part_index,
            note_id: *note_id,
        }),
        PostArcGridContent::LyricLine(s) => Some(AbsoluteContent::LyricLine(s.clone())),
        _ => None,
    })
}
