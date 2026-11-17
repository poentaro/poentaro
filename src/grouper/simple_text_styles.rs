use crate::ast::grouped::{
    default_font_size, resolve_text_style, TextStyle, TextStyleDefaults,
    DEFAULT_HEADER_VERTICAL_PADDING_PT,
};
use crate::ast::parsed::{ParsedMetadata, TextStyleKind};

/// `resolve_text_style` for the common case of a kind with no non-zero
/// horizontal padding default (every kind except `lyrics`, `notes`,
/// `chords`, `note_dash` — see `resolve_text_styles`).
fn simple_text_style(
    parsed: crate::ast::parsed::TextStyle,
    default_font_size: u32,
    default_vertical_padding_pt: u32,
    defaults: TextStyleDefaults,
) -> TextStyle {
    resolve_text_style(
        parsed,
        default_font_size,
        0,
        default_vertical_padding_pt,
        defaults,
    )
}

/// Resolves the `TextStyle` fields whose only non-zero default is `font_size`
/// (see `simple_text_style`). Split out from `resolve_text_styles` to stay
/// under clippy's line-count limit.
pub(super) fn resolve_simple_text_styles(
    metadata: &ParsedMetadata,
    row_height: u32,
) -> (
    TextStyle,
    TextStyle,
    TextStyle,
    TextStyle,
    TextStyle,
    TextStyle,
    TextStyle,
    TextStyle,
    TextStyle,
) {
    use crate::compositor::types::FontFamily;
    let font_size = |kind| default_font_size(metadata, kind, row_height);
    let defaults = |bold: bool, italic: bool, font_family: FontFamily| TextStyleDefaults {
        bold,
        italic,
        font_family,
    };
    (
        simple_text_style(
            metadata.title_style,
            font_size(TextStyleKind::Title),
            DEFAULT_HEADER_VERTICAL_PADDING_PT,
            defaults(false, false, FontFamily::Serif),
        ),
        simple_text_style(
            metadata.subtitle_style,
            font_size(TextStyleKind::Subtitle),
            DEFAULT_HEADER_VERTICAL_PADDING_PT,
            defaults(false, true, FontFamily::Serif),
        ),
        simple_text_style(
            metadata.author_style,
            font_size(TextStyleKind::Author),
            DEFAULT_HEADER_VERTICAL_PADDING_PT,
            defaults(false, false, FontFamily::Serif),
        ),
        simple_text_style(
            metadata.sequence_style,
            font_size(TextStyleKind::Sequence),
            0,
            defaults(false, false, FontFamily::SansSerif),
        ),
        simple_text_style(
            metadata.part_legend_style,
            font_size(TextStyleKind::PartLegend),
            DEFAULT_HEADER_VERTICAL_PADDING_PT,
            defaults(false, false, FontFamily::Serif),
        ),
        simple_text_style(
            metadata.measure_number_style,
            font_size(TextStyleKind::MeasureNumber),
            0,
            defaults(false, false, FontFamily::SansSerif),
        ),
        simple_text_style(
            metadata.section_label_style,
            font_size(TextStyleKind::SectionLabel),
            0,
            defaults(true, true, FontFamily::SansSerif),
        ),
        simple_text_style(
            metadata.page_number_style,
            font_size(TextStyleKind::PageNumber),
            0,
            defaults(false, false, FontFamily::SansSerif),
        ),
        simple_text_style(
            metadata.part_label_style,
            font_size(TextStyleKind::PartLabel),
            0,
            defaults(false, false, FontFamily::SansSerif),
        ),
    )
}
