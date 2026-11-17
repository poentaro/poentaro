// ── Text-style defaults and the fully-resolved `TextStyle` type ────────────

use crate::ast::parsed::TextStyleKind;

/// Default `row_height` in points, used when unset in `metadata`.
pub const DEFAULT_ROW_HEIGHT: u32 = 24;
/// Default `max_measures_per_system`, used when unset in `metadata`.
pub const DEFAULT_MAX_MEASURES_PER_SYSTEM: u32 = 4;
/// Default `note_number_width` in points, used when unset in `metadata`.
pub const DEFAULT_NOTE_NUMBER_WIDTH: u32 = 8;
/// Default `part_label_width_pt`, used when unset in `metadata`.
pub const DEFAULT_PART_LABEL_WIDTH_PT: u32 = 40;

/// Default `page_margin_pt`, used when unset in `metadata`.
pub const DEFAULT_PAGE_MARGIN_PT: u32 = 25;
/// Default `parts_list_columns`, used when unset in `metadata`.
pub const DEFAULT_PARTS_LIST_COLUMNS: u32 = 4;
/// Default `merge_duplicate_measures_across_parts`, used when unset in `metadata`.
pub const DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS: bool = true;
/// Default `hide_resting_parts`, used when unset in `metadata`.
pub const DEFAULT_HIDE_RESTING_PARTS: bool = true;
/// Default `hide_system_dividers`, used when unset in `metadata`.
pub const DEFAULT_HIDE_SYSTEM_DIVIDERS: bool = false;
/// Default `directive_row_offset`, used when unset in `metadata`.
pub const DEFAULT_DIRECTIVE_ROW_OFFSET: crate::ast::parsed::Offset =
    crate::ast::parsed::Offset { x: 0, y: 0 };

/// How a text kind's `font_size` defaults when its `<kind> = { ... }` style
/// object leaves it unset. The single source of truth for every kind's
/// default font size: the grouper resolves from it, and the Edit Metadata
/// modal's help text is generated from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSizeDefault {
    /// A fixed size in points.
    Fixed { points: u32 },
    /// `percent`% of `row_height`, rounded to the nearest point.
    RowHeightPercent { percent: u32 },
    /// Whatever `kind`'s `font_size` resolves to (its own override, else its
    /// own default).
    SameAs { kind: TextStyleKind },
}

impl TextStyleKind {
    pub fn font_size_default(self) -> FontSizeDefault {
        use FontSizeDefault::{Fixed, RowHeightPercent, SameAs};
        match self {
            Self::Title => RowHeightPercent { percent: 150 },
            Self::Subtitle => RowHeightPercent { percent: 80 },
            Self::Author
            | Self::PartLegend
            | Self::PageNumber
            | Self::RunningTitle
            | Self::Lyrics => RowHeightPercent { percent: 60 },
            Self::Sequence | Self::SectionLabel | Self::PartLabel => Fixed { points: 12 },
            Self::MeasureNumber => Fixed { points: 10 },
            Self::Notes | Self::Chords => SameAs { kind: Self::Lyrics },
            Self::NoteDash => SameAs { kind: Self::Notes },
        }
    }
}

/// `kind`'s default `font_size` in points under `metadata` (whose other
/// kinds' overrides a [`FontSizeDefault::SameAs`] rule follows), with
/// `row_height` already resolved.
pub(crate) fn default_font_size(
    metadata: &crate::ast::parsed::ParsedMetadata,
    kind: TextStyleKind,
    row_height: u32,
) -> u32 {
    match kind.font_size_default() {
        FontSizeDefault::Fixed { points } => points,
        FontSizeDefault::RowHeightPercent { percent } => (row_height * percent + 50) / 100,
        FontSizeDefault::SameAs { kind } => metadata
            .style(kind)
            .font_size
            .unwrap_or_else(|| default_font_size(metadata, kind, row_height)),
    }
}

/// Default `vertical_padding_pt` of `title`, `subtitle`, `author`
/// and `part_legend`, used when unset in `metadata`. A header row is one
/// font size tall plus this padding above and below, so `0` is the tightest.
pub const DEFAULT_HEADER_VERTICAL_PADDING_PT: u32 = 4;

/// Default `lyrics.vertical_padding_pt` (formerly `lyric_click_target_padding_pt`),
/// used when unset in `metadata`.
pub const DEFAULT_LYRIC_CLICK_TARGET_PADDING_PT: u32 = 12;

/// Default `notes.horizontal_padding_pt`, used when unset in `metadata`.
pub const DEFAULT_NOTES_HORIZONTAL_PADDING_PT: u32 = 4;
/// Default `chords.horizontal_padding_pt`, used when unset in `metadata`.
pub const DEFAULT_CHORDS_HORIZONTAL_PADDING_PT: u32 = 4;
/// Default `lyrics.horizontal_padding_pt`, used when unset in `metadata`.
pub const DEFAULT_LYRICS_HORIZONTAL_PADDING_PT: u32 = 4;
/// Default `note_dash.horizontal_padding_pt`, used when unset in `metadata`.
pub const DEFAULT_NOTE_DASH_HORIZONTAL_PADDING_PT: u32 = 4;

/// Fully-resolved (all components defaulted) per-kind text style: font size plus
/// the two layout components every text kind now shares. See
/// `crate::ast::parsed::TextStyle` for the parsed (`Option`-wrapped) counterpart
/// and `resolve_text_style` for how a kind's default `font_size` is derived.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TextStyle {
    pub font_size: u32,
    /// Horizontal padding in points reserved before this kind's glyph, widening
    /// its column's spacing rod by the same amount (see
    /// `grid_layout::layout_spacing::column_rod`). Default: 0, except `notes`,
    /// `chords`, `lyrics`, `note_dash` (4).
    pub horizontal_padding_pt: u32,
    /// Extra vertical padding in points added above/below this kind's element.
    /// Default: 0, except `lyrics` (12, formerly `lyric_click_target_padding_pt`).
    pub vertical_padding_pt: u32,
    /// Whether this kind renders bold. Default: `false`, except `section_label` (`true`).
    pub bold: bool,
    /// Whether this kind renders italic. Default: `false`, except `subtitle`
    /// and `section_label` (`true`).
    pub italic: bool,
    /// Whether this kind renders underlined. Default: `false` for every kind.
    pub underline: bool,
    /// Which embedded font role this kind's glyphs render in. Default:
    /// `Serif` for `title`/`subtitle`/`author`/`lyrics`, `SansSerif` for
    /// every other kind, `Monospace` for `notes`/`chords`/`note_dash` — all
    /// overridable, including the three `Monospace`-default kinds (see
    /// `RenderConfig::glyph_font_families`, `font_metrics::
    /// advance_width_for_family`, which re-measures their glyph widths
    /// against whichever font this resolves to, so an override can't desync
    /// layout from what actually renders).
    pub font_family: crate::compositor::types::FontFamily,
}

/// Bundles [`resolve_text_style`]'s `default_bold`/`default_italic`/
/// `default_font_family` — split out once `font_family` pushed the plain
/// argument list over clippy's `too_many_arguments` limit. `default_bold`/
/// `default_italic` are `false` for every kind except where documented
/// otherwise on `Metadata`'s per-kind fields.
#[derive(Clone, Copy)]
pub(crate) struct TextStyleDefaults {
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) font_family: crate::compositor::types::FontFamily,
}

/// Fills in each unset component of a parsed `<kind> = { ... }` style object
/// with its default, producing the fully-resolved `TextStyle` a kind's
/// `Metadata` field holds. `default_horizontal_padding_pt`/
/// `default_vertical_padding_pt` are `0` for every kind except where
/// documented otherwise on `Metadata`'s per-kind fields; `underline` has no
/// per-kind default and always falls back to `false`.
pub(crate) fn resolve_text_style(
    parsed: crate::ast::parsed::TextStyle,
    default_font_size: u32,
    default_horizontal_padding_pt: u32,
    default_vertical_padding_pt: u32,
    defaults: TextStyleDefaults,
) -> TextStyle {
    TextStyle {
        font_size: parsed.font_size.unwrap_or(default_font_size),
        horizontal_padding_pt: parsed
            .horizontal_padding_pt
            .unwrap_or(default_horizontal_padding_pt),
        vertical_padding_pt: parsed
            .vertical_padding_pt
            .unwrap_or(default_vertical_padding_pt),
        bold: parsed.bold.unwrap_or(defaults.bold),
        italic: parsed.italic.unwrap_or(defaults.italic),
        underline: parsed.underline.unwrap_or(false),
        font_family: parsed.font_family.map_or(defaults.font_family, |choice| {
            use crate::ast::parsed::FontFamilyChoice;
            use crate::compositor::types::FontFamily;
            match choice {
                FontFamilyChoice::Serif => FontFamily::Serif,
                FontFamilyChoice::SansSerif => FontFamily::SansSerif,
                FontFamilyChoice::Monospace => FontFamily::Monospace,
            }
        }),
    }
}

impl super::Metadata {
    pub fn style(&self, kind: TextStyleKind) -> &TextStyle {
        match kind {
            TextStyleKind::Title => &self.title_style,
            TextStyleKind::Subtitle => &self.subtitle_style,
            TextStyleKind::Author => &self.author_style,
            TextStyleKind::Sequence => &self.sequence,
            TextStyleKind::PartLegend => &self.part_legend,
            TextStyleKind::MeasureNumber => &self.measure_number,
            TextStyleKind::SectionLabel => &self.section_label,
            TextStyleKind::PartLabel => &self.part_label,
            TextStyleKind::PageNumber => &self.page_number,
            TextStyleKind::RunningTitle => &self.running_title,
            TextStyleKind::Lyrics => &self.lyrics,
            TextStyleKind::Notes => &self.notes,
            TextStyleKind::Chords => &self.chords,
            TextStyleKind::NoteDash => &self.note_dash,
        }
    }

    pub fn style_mut(&mut self, kind: TextStyleKind) -> &mut TextStyle {
        match kind {
            TextStyleKind::Title => &mut self.title_style,
            TextStyleKind::Subtitle => &mut self.subtitle_style,
            TextStyleKind::Author => &mut self.author_style,
            TextStyleKind::Sequence => &mut self.sequence,
            TextStyleKind::PartLegend => &mut self.part_legend,
            TextStyleKind::MeasureNumber => &mut self.measure_number,
            TextStyleKind::SectionLabel => &mut self.section_label,
            TextStyleKind::PartLabel => &mut self.part_label,
            TextStyleKind::PageNumber => &mut self.page_number,
            TextStyleKind::RunningTitle => &mut self.running_title,
            TextStyleKind::Lyrics => &mut self.lyrics,
            TextStyleKind::Notes => &mut self.notes,
            TextStyleKind::Chords => &mut self.chords,
            TextStyleKind::NoteDash => &mut self.note_dash,
        }
    }
}
