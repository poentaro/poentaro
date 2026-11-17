use crate::ast::grouped::{default_font_size, resolve_text_style, TextStyle, TextStyleDefaults};
use crate::ast::parsed::{ParsedMetadata, TextStyleKind};
use crate::compositor::types::FontFamily;

/// `resolve_text_style` for the common case of a kind with no non-zero
/// horizontal/vertical padding default (every kind except `lyrics`, `notes`,
/// `chords`, `note_dash` — see `resolve_text_styles`).
pub(super) fn simple_text_style(
    parsed: crate::ast::parsed::TextStyle,
    default_font_size: u32,
    defaults: TextStyleDefaults,
) -> TextStyle {
    resolve_text_style(parsed, default_font_size, 0, 0, defaults)
}

/// The running title's `TextStyle`: italic serif, `font_size` 60% of
/// `row_height` unless overridden.
pub(super) fn resolve_running_title(metadata: &ParsedMetadata, row_height: u32) -> TextStyle {
    simple_text_style(
        metadata.running_title_style,
        default_font_size(metadata, TextStyleKind::RunningTitle, row_height),
        TextStyleDefaults {
            bold: false,
            italic: true,
            font_family: FontFamily::Serif,
        },
    )
}
