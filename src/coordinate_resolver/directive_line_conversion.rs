use crate::compositor::types::{AbsoluteContent, FontFamily, TextSpan};
use crate::grid_layout::types::PostArcGridContent;

use super::content_conversion::{bar_number_text_span, section_label_span};

/// Font sizes for the small overlay labels drawn around the score body —
/// measure numbers and inline section labels — threaded from
/// `RenderConfig` down through `resolve`/`grid_to_absolute` (see
/// `Metadata::measure_number_font_size`/`Metadata::section_label_font_size`).
#[derive(Clone, Copy)]
pub(super) struct DirectiveLineFontSizes {
    pub(super) measure_number: f32,
    pub(super) section_label: f32,
    /// See `Metadata::section_label.vertical_padding_pt` /
    /// `AbsoluteContent::DirectiveLine::label_box_height`.
    pub(super) section_label_vertical_padding_pt: f32,
    /// See `Metadata::measure_number_style`.
    pub(super) measure_number_bold: bool,
    pub(super) measure_number_italic: bool,
    pub(super) measure_number_underline: bool,
    pub(super) measure_number_font_family: FontFamily,
    /// See `Metadata::section_label_style`.
    pub(super) section_label_bold: bool,
    pub(super) section_label_italic: bool,
    pub(super) section_label_underline: bool,
    pub(super) section_label_font_family: FontFamily,
    /// See `Metadata::sequence`.
    pub(super) sequence_bold: bool,
    pub(super) sequence_italic: bool,
    pub(super) sequence_underline: bool,
    pub(super) sequence_font_family: FontFamily,
}

/// Result of [`directive_line_content`]: the line's text spans plus layout
/// hints for elements positioned separately from the monolithic `<text>`
/// (the bar number, the section-label bounding box).
struct DirectiveLineContent {
    /// Bar-number span, rendered as its own text element at the line's
    /// start (offset 0).
    bar_number: Option<TextSpan>,
    spans: Vec<TextSpan>,
    /// X offset (in points, from the line's start) where `spans` begins:
    /// see the field of the same name on
    /// [`crate::compositor::types::AbsoluteContent::DirectiveLine`].
    spans_x_offset: f32,
    /// X offset (in points, from the line's start) where the label's own,
    /// independently-positioned text element begins: past `bar_number`'s
    /// measured width (plus a gap) when one is present, zero otherwise.
    label_x_offset: f32,
}

/// Builds the text spans for a directive line (excluding the bar number
/// and section label, which are rendered as their own independent text
/// elements — see `bar_number`/`label_x_offset`), plus layout hints for the
/// section-label box (see [`DirectiveLineContent`]).
///
/// Two explicit passes, per Task 4 of
/// `PLAN-section-label-engraving-quality.md`: pass 1
/// ([`build_directive_line_spans`]) builds the line's logical elements
/// (content/style only, no positions); pass 2 below walks that list once,
/// measuring each element with real font-metrics glyph advances (see
/// [`crate::font_metrics`]), since actual text layout happens in the
/// browser and isn't otherwise available here. Elements are laid out left
/// to right in a fixed order — bar number, then section label, then the
/// rest of the directives — so a label never intersects the directives
/// that follow it, regardless of how short or long the bar number is (this
/// ordering is a follow-up fix on top of the 5 tasks in
/// `PLAN-section-label-engraving-quality.md`, not one of the tasks itself).
fn directive_line_content(
    content: &PostArcGridContent,
    font_sizes: DirectiveLineFontSizes,
) -> DirectiveLineContent {
    let PostArcGridContent::DirectiveLine { label, .. } = content else {
        return DirectiveLineContent {
            bar_number: None,
            spans: Vec::new(),
            spans_x_offset: 0.0,
            label_x_offset: 0.0,
        };
    };

    let (bar_number_span, spans) = build_directive_line_spans(content, font_sizes);
    let bar_number_width = bar_number_span
        .as_ref()
        .map(crate::font_metrics::span_width)
        .unwrap_or(0.0);

    let label_x_offset = if label.is_some() && bar_number_span.is_some() {
        bar_number_width + crate::font_metrics::DIRECTIVE_LINE_ELEMENT_GAP
    } else {
        0.0
    };
    let spans_x_offset = match label {
        Some(label_str) => {
            label_x_offset
                + crate::font_metrics::section_label_box_width(
                    label_str,
                    font_sizes.section_label,
                    font_sizes.section_label_bold,
                )
                + crate::font_metrics::DIRECTIVE_LINE_ELEMENT_GAP
        }
        // The spans' own leading spaces don't render (SVG drops leading
        // whitespace in a `<text>` without `xml:space="preserve"`), so the
        // gap after the bar number must be measured, as in the label case.
        None if bar_number_span.is_some() && !spans.is_empty() => {
            bar_number_width + crate::font_metrics::DIRECTIVE_LINE_ELEMENT_GAP
        }
        None => bar_number_width,
    };

    DirectiveLineContent {
        bar_number: bar_number_span,
        spans,
        spans_x_offset,
        label_x_offset,
    }
}

/// Pass 1 of [`directive_line_content`]: builds the directive line's
/// ordered logical elements — the bar number, plus key/bpm/time signature —
/// with their content/style, but no positions — positions are assigned in
/// pass 2.
fn build_directive_line_spans(
    content: &PostArcGridContent,
    font_sizes: DirectiveLineFontSizes,
) -> (Option<TextSpan>, Vec<TextSpan>) {
    let PostArcGridContent::DirectiveLine {
        bar_number,
        key,
        bpm,
        time_signature,
        ..
    } = content
    else {
        return (None, Vec::new());
    };
    let bar_number_span = bar_number.map(|n| {
        bar_number_text_span(
            n,
            font_sizes.measure_number,
            font_sizes.measure_number_bold,
            font_sizes.measure_number_italic,
            font_sizes.measure_number_underline,
        )
    });
    let mut spans: Vec<TextSpan> = Vec::new();
    if let Some(key_str) = key {
        spans.push(TextSpan {
            content: format!("  {key_str}"),
            bold: false,
            italic: false,
            underline: false,
            font_size: 12.0,
        });
    }
    if let Some(b) = bpm {
        spans.push(TextSpan {
            content: format!("  \u{2669}={b}"),
            bold: false,
            italic: false,
            underline: false,
            font_size: 12.0,
        });
    }
    if let Some((n, d)) = time_signature {
        spans.push(TextSpan {
            content: format!("  {n}/{d}"),
            bold: false,
            italic: false,
            underline: false,
            font_size: 12.0,
        });
    }
    (bar_number_span, spans)
}

/// Builds the text spans for the `sequence` header line: a plain
/// "Sequence: " prefix, each label styled per `Metadata::sequence` (built via
/// the shared [`section_label_span`] helper) — followed by a plain, non-bold/italic
/// `(-abbrev -abbrev ...)` (omit) or `(abbrev abbrev ...)` (only) span when
/// that entry's suffix restricts that occurrence's MIDI/WAV playback —
/// joined by a plain " › ".
pub(super) fn sequence_line_content(
    entries: &[crate::grid_layout::types::SequenceEntryInfo],
    font_size: f32,
    directive_font_sizes: DirectiveLineFontSizes,
) -> Vec<TextSpan> {
    use crate::parser::sequence_parser::PartFilterKind;

    let mut spans = vec![TextSpan {
        content: "Sequence: ".to_string(),
        bold: false,
        italic: false,
        underline: false,
        font_size,
    }];
    for (index, entry) in entries.iter().enumerate() {
        if index > 0 {
            spans.push(TextSpan {
                content: " \u{203a} ".to_string(),
                bold: false,
                italic: false,
                underline: false,
                font_size,
            });
        }
        spans.push(section_label_span(
            &entry.label,
            font_size,
            directive_font_sizes.sequence_bold,
            directive_font_sizes.sequence_italic,
            directive_font_sizes.sequence_underline,
        ));
        if let Some(filter) = &entry.part_filter {
            let content = match filter.kind {
                PartFilterKind::Omit => format!(" (-{})", filter.parts.join(" -")),
                PartFilterKind::Only => format!(" ({})", filter.parts.join(" ")),
            };
            spans.push(TextSpan {
                content,
                bold: false,
                italic: false,
                underline: false,
                font_size,
            });
        }
    }
    spans
}

/// Builds the `AbsoluteContent::DirectiveLine` variant for an actual
/// directive line (as opposed to the `sequence` summary header — see
/// `grid_text_to_absolute`'s `PostArcGridContent::SequenceLine` arm, which
/// builds its own directly since it has no label/box to compute). Split out
/// of `grid_text_to_absolute` to keep that function under the repo's
/// max-lines lint.
pub(super) fn directive_line_absolute(
    content: &PostArcGridContent,
    label: &Option<String>,
    directive_font_sizes: DirectiveLineFontSizes,
) -> AbsoluteContent {
    let directive_line = directive_line_content(content, directive_font_sizes);
    AbsoluteContent::DirectiveLine {
        bar_number: directive_line.bar_number,
        bar_number_font_family: directive_font_sizes.measure_number_font_family,
        label: label.clone(),
        label_font_size: directive_font_sizes.section_label,
        label_bold: directive_font_sizes.section_label_bold,
        label_italic: directive_font_sizes.section_label_italic,
        label_underline: directive_font_sizes.section_label_underline,
        label_font_family: directive_font_sizes.section_label_font_family,
        label_box_height: crate::font_metrics::section_label_box_height(
            directive_font_sizes.section_label,
        ) + directive_font_sizes.section_label_vertical_padding_pt,
        spans: directive_line.spans,
        spans_font_family: FontFamily::SansSerif,
        spans_x_offset: directive_line.spans_x_offset,
        label_x_offset: directive_line.label_x_offset,
        apply_row_offset: true,
    }
}
