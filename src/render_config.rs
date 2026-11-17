use crate::ast::grouped::Metadata;
use crate::ast::parsed::Offset;
use crate::compositor::types::{FontFamily, GlyphFontFamilies};
use crate::coordinate_resolver::{ElementPaddings, LyricFontSizes};
use crate::grid_layout::layout::LyricSizing;

#[derive(Debug, Clone, Default)]
pub struct RenderConfig {
    pub row_height: u32,
    pub note_number_width: u32,
    pub part_label_width_pt: u32,
    /// Blank margin in points on every side of the page (see
    /// `Metadata::page_margin_pt`).
    pub page_margin_pt: u32,
    pub max_measures_per_system: u32,
    pub lyrics_font_size: u32,
    pub notes_font_size: u32,
    pub chords_font_size: u32,
    /// Font size in points of a note dash (see `Metadata::note_dash_font_size`).
    /// Previously note dashes always rendered at `notes_font_size`, ignoring
    /// this field entirely — see `note_dash_font_size()`.
    pub note_dash_font_size: u32,
    pub hide_system_dividers: bool,
    pub directive_row_offset: Offset,
    /// Font size in points of each measure's bar number (see
    /// `Metadata::measure_number_font_size`).
    pub measure_number_font_size: u32,
    /// Font size in points of an inline section label (see
    /// `Metadata::section_label_font_size`).
    pub section_label_font_size: u32,
    /// Font size in points of a part's row label (see
    /// `Metadata::part_label_font_size`).
    pub part_label_font_size: u32,
    /// Font size in points of the footer page number (see
    /// `Metadata::page_number_font_size`).
    pub page_number_font_size: u32,
    /// See `Metadata::notes_style`.
    pub notes_bold: bool,
    pub notes_italic: bool,
    pub notes_underline: bool,
    /// See `Metadata::chords_style`.
    pub chords_bold: bool,
    pub chords_italic: bool,
    pub chords_underline: bool,
    /// See `Metadata::lyrics_style`.
    pub lyrics_bold: bool,
    pub lyrics_italic: bool,
    pub lyrics_underline: bool,
    pub lyrics_font_family: FontFamily,
    /// See `Metadata::note_dash_style`.
    pub note_dash_bold: bool,
    pub note_dash_italic: bool,
    pub note_dash_underline: bool,
    /// Which `FontFamily` each of `notes`/`chords`/`note_dash` renders in
    /// (see `Metadata::notes_style`/`chords_style`/`note_dash_style`'s own
    /// `font_family`) — bundled since `font_metrics`'s glyph-width
    /// measurement and the renderer both need all three together to keep a
    /// measure's computed layout width in sync with what actually renders.
    pub glyph_font_families: GlyphFontFamilies,
    /// See `Metadata::sequence`.
    pub sequence_bold: bool,
    pub sequence_italic: bool,
    pub sequence_underline: bool,
    pub sequence_font_family: FontFamily,
    /// See `Metadata::measure_number_style`.
    pub measure_number_bold: bool,
    pub measure_number_italic: bool,
    pub measure_number_underline: bool,
    pub measure_number_font_family: FontFamily,
    /// See `Metadata::section_label_style`.
    pub section_label_bold: bool,
    pub section_label_italic: bool,
    pub section_label_underline: bool,
    pub section_label_font_family: FontFamily,
    /// See `Metadata::part_label_style`.
    pub part_label_bold: bool,
    pub part_label_italic: bool,
    pub part_label_underline: bool,
    pub part_label_font_family: FontFamily,
    /// See `Metadata::page_number_style`.
    pub page_number_bold: bool,
    pub page_number_italic: bool,
    pub page_number_underline: bool,
    pub page_number_font_family: FontFamily,
    /// Extra vertical padding in points around a lyric syllable's
    /// click-target box (see `Metadata::lyric_click_target_padding_pt`).
    pub lyric_click_target_padding_pt: u32,
    /// Extra vertical padding in points added to a note/chord part's
    /// note-head sub-row (see `Metadata::notes.vertical_padding_pt`).
    pub notes_vertical_padding_pt: u32,
    /// Extra vertical padding in points added to an inline section label's
    /// rendered box (see `Metadata::section_label.vertical_padding_pt`).
    pub section_label_vertical_padding_pt: u32,
    /// Extra vertical padding in points, offsetting the footer page number
    /// upward from the page's bottom edge (see
    /// `Metadata::page_number.vertical_padding_pt`).
    pub page_number_vertical_padding_pt: u32,
    /// Horizontal padding in points reserved before a note head/rest/percussion-hit
    /// glyph (see `Metadata::notes_horizontal_padding_pt`).
    pub notes_horizontal_padding_pt: u32,
    /// Horizontal padding in points reserved before a chord symbol (see
    /// `Metadata::chords_horizontal_padding_pt`).
    pub chords_horizontal_padding_pt: u32,
    /// Horizontal padding in points reserved before a lyric syllable (see
    /// `Metadata::lyrics_horizontal_padding_pt`).
    pub lyrics_horizontal_padding_pt: u32,
    /// Horizontal padding in points reserved before a note dash (see
    /// `Metadata::note_dash_horizontal_padding_pt`).
    pub note_dash_horizontal_padding_pt: u32,
}

impl RenderConfig {
    /// The page margin and part-label column every page is laid out in (see
    /// `grid_layout::PageFrame`).
    pub fn page_frame(&self) -> crate::grid_layout::PageFrame {
        crate::grid_layout::PageFrame {
            margin_pt: self.page_margin_pt as f32,
            part_label_width_pt: self.part_label_width_pt as f32,
        }
    }

    pub fn from_metadata(meta: &Metadata) -> Self {
        RenderConfig {
            row_height: meta.row_height,
            note_number_width: meta.note_number_width,
            part_label_width_pt: meta.part_label_width_pt,
            page_margin_pt: meta.page_margin_pt,
            max_measures_per_system: meta.max_measures_per_system,
            lyrics_font_size: meta.lyrics.font_size,
            notes_font_size: meta.notes.font_size,
            chords_font_size: meta.chords.font_size,
            note_dash_font_size: meta.note_dash.font_size,
            hide_system_dividers: meta.hide_system_dividers,
            directive_row_offset: meta.directive_row_offset,
            measure_number_font_size: meta.measure_number.font_size,
            section_label_font_size: meta.section_label.font_size,
            part_label_font_size: meta.part_label.font_size,
            page_number_font_size: meta.page_number.font_size,
            notes_bold: meta.notes.bold,
            notes_italic: meta.notes.italic,
            notes_underline: meta.notes.underline,
            chords_bold: meta.chords.bold,
            chords_italic: meta.chords.italic,
            chords_underline: meta.chords.underline,
            lyrics_bold: meta.lyrics.bold,
            lyrics_italic: meta.lyrics.italic,
            lyrics_underline: meta.lyrics.underline,
            lyrics_font_family: meta.lyrics.font_family,
            note_dash_bold: meta.note_dash.bold,
            note_dash_italic: meta.note_dash.italic,
            note_dash_underline: meta.note_dash.underline,
            glyph_font_families: GlyphFontFamilies {
                notes: meta.notes.font_family,
                chords: meta.chords.font_family,
                note_dash: meta.note_dash.font_family,
            },
            sequence_bold: meta.sequence.bold,
            sequence_italic: meta.sequence.italic,
            sequence_underline: meta.sequence.underline,
            sequence_font_family: meta.sequence.font_family,
            measure_number_bold: meta.measure_number.bold,
            measure_number_italic: meta.measure_number.italic,
            measure_number_underline: meta.measure_number.underline,
            measure_number_font_family: meta.measure_number.font_family,
            section_label_bold: meta.section_label.bold,
            section_label_italic: meta.section_label.italic,
            section_label_underline: meta.section_label.underline,
            section_label_font_family: meta.section_label.font_family,
            part_label_bold: meta.part_label.bold,
            part_label_italic: meta.part_label.italic,
            part_label_underline: meta.part_label.underline,
            part_label_font_family: meta.part_label.font_family,
            page_number_bold: meta.page_number.bold,
            page_number_italic: meta.page_number.italic,
            page_number_underline: meta.page_number.underline,
            page_number_font_family: meta.page_number.font_family,
            lyric_click_target_padding_pt: meta.lyrics.vertical_padding_pt,
            notes_vertical_padding_pt: meta.notes.vertical_padding_pt,
            section_label_vertical_padding_pt: meta.section_label.vertical_padding_pt,
            page_number_vertical_padding_pt: meta.page_number.vertical_padding_pt,
            notes_horizontal_padding_pt: meta.notes.horizontal_padding_pt,
            chords_horizontal_padding_pt: meta.chords.horizontal_padding_pt,
            lyrics_horizontal_padding_pt: meta.lyrics.horizontal_padding_pt,
            note_dash_horizontal_padding_pt: meta.note_dash.horizontal_padding_pt,
        }
    }

    /// Font size used for Latin-script lyric syllables (and other body text).
    pub fn lyric_font_size(&self) -> f32 {
        self.lyrics_font_size as f32
    }

    /// Font size used for CJK lyric syllables, which render larger than Latin
    /// glyphs at the same visual weight.
    pub fn lyric_cjk_font_size(&self) -> f32 {
        self.lyric_font_size() * 1.2
    }

    pub fn lyric_font_sizes(&self) -> LyricFontSizes {
        LyricFontSizes {
            base: self.lyric_font_size(),
            cjk: self.lyric_cjk_font_size(),
        }
    }

    /// Font size used for note heads, rests, percussion hits, and tuplet brackets.
    pub fn notes_font_size(&self) -> f32 {
        self.notes_font_size as f32
    }

    /// Font size used for chord symbols.
    pub fn chords_font_size(&self) -> f32 {
        self.chords_font_size as f32
    }

    /// Font size used for note dashes (see `Metadata::note_dash_font_size`).
    pub fn note_dash_font_size(&self) -> f32 {
        self.note_dash_font_size as f32
    }

    /// Extra vertical padding around a lyric syllable's click-target box
    /// (see `lyric_row_height`).
    pub fn lyric_click_target_padding_pt(&self) -> f32 {
        self.lyric_click_target_padding_pt as f32
    }

    /// Extra vertical padding around a note/chord part's note-head sub-row
    /// (see `note_part_sub_row_heights`).
    pub fn notes_vertical_padding_pt(&self) -> f32 {
        self.notes_vertical_padding_pt as f32
    }

    /// Extra vertical padding around an inline section label's rendered box
    /// (see `font_metrics::section_label_box_height`).
    pub fn section_label_vertical_padding_pt(&self) -> f32 {
        self.section_label_vertical_padding_pt as f32
    }

    /// Extra vertical padding offsetting the footer page number upward from
    /// the page's bottom edge (see `coordinate_resolver::resolve`).
    pub fn page_number_vertical_padding_pt(&self) -> f32 {
        self.page_number_vertical_padding_pt as f32
    }

    /// `lyric_font_sizes()` plus `lyric_click_target_padding_pt()` plus
    /// `notes_vertical_padding_pt()`, bundled for the grid_layout functions
    /// that need all three together (see `LyricSizing`).
    pub(crate) fn lyric_sizing(&self) -> LyricSizing {
        LyricSizing {
            font_sizes: self.lyric_font_sizes(),
            click_target_padding_pt: self.lyric_click_target_padding_pt(),
            notes_vertical_padding_pt: self.notes_vertical_padding_pt(),
        }
    }

    /// Horizontal padding in points reserved before a note head/rest/percussion-hit
    /// glyph (see `Metadata::notes_horizontal_padding_pt`). Also backs the
    /// multi-measure-rest bar's end insets and the tie/slur/underline/tuplet-bracket
    /// span anchors, all of which key off a note column.
    pub fn notes_horizontal_padding_pt(&self) -> f32 {
        self.notes_horizontal_padding_pt as f32
    }

    /// Horizontal padding in points reserved before a chord symbol (see
    /// `Metadata::chords_horizontal_padding_pt`).
    pub fn chords_horizontal_padding_pt(&self) -> f32 {
        self.chords_horizontal_padding_pt as f32
    }

    /// Horizontal padding in points reserved before a lyric syllable (see
    /// `Metadata::lyrics_horizontal_padding_pt`).
    pub fn lyrics_horizontal_padding_pt(&self) -> f32 {
        self.lyrics_horizontal_padding_pt as f32
    }

    /// Horizontal padding in points reserved before a note dash (see
    /// `Metadata::note_dash_horizontal_padding_pt`).
    pub fn note_dash_horizontal_padding_pt(&self) -> f32 {
        self.note_dash_horizontal_padding_pt as f32
    }

    /// Every `*_horizontal_padding_pt()` accessor bundled together, for the
    /// `coordinate_resolver` functions that need all four (see
    /// `ElementPaddings`).
    pub(crate) fn element_paddings(&self) -> ElementPaddings {
        ElementPaddings {
            notes: self.notes_horizontal_padding_pt(),
            chords: self.chords_horizontal_padding_pt(),
            lyrics: self.lyrics_horizontal_padding_pt(),
            note_dash: self.note_dash_horizontal_padding_pt(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::grouped::Metadata;

    fn text_style(font_size: u32) -> crate::ast::grouped::TextStyle {
        crate::ast::grouped::TextStyle {
            font_size,
            horizontal_padding_pt: 4,
            vertical_padding_pt: 0,
            ..Default::default()
        }
    }

    #[test]
    fn from_metadata_copies_fields() {
        let meta = Metadata {
            title: None,
            subtitle: None,
            author: None,
            row_height: 30,
            note_number_width: 12,
            max_measures_per_system: 6,
            parts_list_columns: 3,
            part_label_width_pt: 40,
            page_margin_pt: 25,
            lyrics: crate::ast::grouped::TextStyle {
                vertical_padding_pt: 12,
                ..text_style(18)
            },
            notes: crate::ast::grouped::TextStyle {
                vertical_padding_pt: 5,
                ..text_style(18)
            },
            chords: text_style(18),
            note_dash: text_style(18),
            title_style: text_style(45),
            subtitle_style: text_style(24),
            author_style: text_style(18),
            sequence: text_style(12),
            part_legend: text_style(12),
            merge_duplicate_measures_across_parts: true,
            hide_resting_parts: true,
            hide_system_dividers: false,
            directive_row_offset: Offset::default(),
            measure_number: text_style(10),
            section_label: crate::ast::grouped::TextStyle {
                vertical_padding_pt: 8,
                ..text_style(12)
            },
            part_label: text_style(12),
            page_number: crate::ast::grouped::TextStyle {
                vertical_padding_pt: 4,
                ..text_style(18)
            },
            running_title: text_style(12),
        };
        let cfg = RenderConfig::from_metadata(&meta);
        assert_eq!(cfg.row_height, 30);
        assert_eq!(cfg.note_number_width, 12);
        assert_eq!(cfg.part_label_width_pt, 40);
        assert_eq!(cfg.max_measures_per_system, 6);
        assert_eq!(cfg.lyrics_font_size, 18);
        assert_eq!(cfg.lyric_font_size(), 18.0);
        assert_eq!(cfg.note_dash_font_size(), 18.0);
        assert_eq!(cfg.lyric_click_target_padding_pt(), 12.0);
        assert_eq!(cfg.measure_number_font_size, 10);
        assert_eq!(cfg.section_label_font_size, 12);
        assert_eq!(cfg.part_label_font_size, 12);
        assert_eq!(cfg.page_number_font_size, 18);
        assert_eq!(cfg.notes_horizontal_padding_pt(), 4.0);
        assert_eq!(cfg.chords_horizontal_padding_pt(), 4.0);
        assert_eq!(cfg.lyrics_horizontal_padding_pt(), 4.0);
        assert_eq!(cfg.note_dash_horizontal_padding_pt(), 4.0);
        assert_eq!(cfg.notes_vertical_padding_pt(), 5.0);
        assert_eq!(cfg.section_label_vertical_padding_pt(), 8.0);
        assert_eq!(cfg.page_number_vertical_padding_pt(), 4.0);
    }
}
