use super::page_geometry::PageGeometry;
use crate::compositor::types::{AbsoluteElement, AbsolutePage, FontFamily, GlyphFontFamilies};
use crate::error::IrrecoverableError;
use crate::grid_layout::types::{
    ColumnGeometry, GridContent, GridElement, GridPage, GridRow, HAlign, VAlign,
};
use crate::grid_layout::PageFrame;

use super::click_targets::resolve_click_target_elements;
use super::content_conversion::grid_to_absolute;
use super::directive_line_conversion::DirectiveLineFontSizes;
use super::flush_left::{flush_left_padding, is_flush_left_glyph};
use super::highlights::{resolve_error_highlights, resolve_measure_highlights};
use super::post_arc_conversion::to_post_arc_content;
use super::rest_run::{resolve_implicit_fill_rest, resolve_multi_measure_rest};
use super::span_marking::resolve_span_marking;

/// Font sizes used to measure lyric syllable width.
#[derive(Clone, Copy, Default)]
pub struct LyricFontSizes {
    pub base: f32,
    pub cjk: f32,
}

/// Font sizes for the small overlay labels drawn around the score body:
/// measure numbers, inline section labels, and part-name row labels (see
/// `Metadata::measure_number_font_size`/`Metadata::section_label_font_size`/
/// `Metadata::part_label_font_size`).
#[derive(Clone, Copy, Default)]
pub struct LabelFontSizes {
    pub measure_number: f32,
    pub section_label: f32,
    pub part_label: f32,
    /// Extra vertical padding in points added to an inline section label's
    /// rendered background box (see `Metadata::section_label.vertical_padding_pt`
    /// / `font_metrics::section_label_box_height`).
    pub section_label_vertical_padding_pt: f32,
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
    /// See `Metadata::sequence`.
    pub sequence_bold: bool,
    pub sequence_italic: bool,
    pub sequence_underline: bool,
    pub sequence_font_family: FontFamily,
}

impl LabelFontSizes {
    /// This kind's part-label style, bundled for `grid_to_absolute`'s
    /// `RowLabel` arm — split out of `resolve_row_element` to keep it under
    /// clippy's line-count limit.
    pub(super) fn part_label_style(&self) -> super::content_conversion::PartLabelStyle {
        super::content_conversion::PartLabelStyle {
            font_size: self.part_label,
            bold: self.part_label_bold,
            italic: self.part_label_italic,
            underline: self.part_label_underline,
            font_family: self.part_label_font_family,
        }
    }

    /// This kind's measure-number/section-label style, bundled for
    /// `grid_to_absolute`'s `DirectiveLine` arm — split out of
    /// `resolve_row_element` to keep it under clippy's line-count limit.
    pub(super) fn directive_line_font_sizes(&self) -> DirectiveLineFontSizes {
        DirectiveLineFontSizes {
            measure_number: self.measure_number,
            section_label: self.section_label,
            section_label_vertical_padding_pt: self.section_label_vertical_padding_pt,
            measure_number_bold: self.measure_number_bold,
            measure_number_italic: self.measure_number_italic,
            measure_number_underline: self.measure_number_underline,
            measure_number_font_family: self.measure_number_font_family,
            section_label_bold: self.section_label_bold,
            section_label_italic: self.section_label_italic,
            section_label_underline: self.section_label_underline,
            section_label_font_family: self.section_label_font_family,
            sequence_bold: self.sequence_bold,
            sequence_italic: self.sequence_italic,
            sequence_underline: self.sequence_underline,
            sequence_font_family: self.sequence_font_family,
        }
    }
}

/// Horizontal padding in points reserved before each flush-left glyph type
/// (see `is_flush_left_glyph`), customizable per `Metadata::*_horizontal_padding_pt`
/// field (see `RenderConfig::element_paddings`). The same value widens that
/// content type's `layout_spacing::column_rod`, so increasing it genuinely
/// spreads elements apart rather than just nudging the glyph inside its
/// existing column — the two can't drift apart since both read from this
/// struct's `RenderConfig` source.
#[derive(Clone, Copy, Default)]
pub struct ElementPaddings {
    pub notes: f32,
    pub chords: f32,
    pub lyrics: f32,
    pub note_dash: f32,
}

/// Every font size and per-element horizontal padding `resolve`/`resolve_page`
/// needs, bundled into one struct (rather than passed as individual
/// arguments) so those two functions stay under the repo's
/// max-argument-count lint.
#[derive(Clone, Copy, Default)]
pub struct ResolveFontSizes {
    pub lyric: LyricFontSizes,
    pub notes: f32,
    pub chords: f32,
    pub labels: LabelFontSizes,
    pub paddings: ElementPaddings,
    /// Extra vertical padding in points, offsetting the footer page number
    /// upward from the page's bottom edge (see
    /// `Metadata::page_number.vertical_padding_pt`).
    pub page_number_vertical_padding_pt: f32,
    /// Which `FontFamily` each of `notes`/`chords`/`note_dash` renders in
    /// (see `RenderConfig::glyph_font_families`), used to measure a flush-left
    /// glyph's own left-side bearing (`flush_left_padding`) against the same
    /// font it actually renders in.
    pub glyph_font_families: GlyphFontFamilies,
}

pub fn resolve(
    pages: &[GridPage],
    note_number_width: f32,
    page_frame: PageFrame,
    font_sizes: ResolveFontSizes,
) -> Result<Vec<AbsolutePage>, IrrecoverableError> {
    pages
        .iter()
        .map(|page| resolve_page(page, note_number_width, page_frame, font_sizes))
        .collect()
}

/// Bundles the config threaded through row-element resolution, kept constant
/// across a page, so `resolve_row_element` stays under the clippy arg limit.
/// `notes_font_size`/`chords_font_size` size a note/chord glyph's own
/// left-side-bearing correction (see `flush_left_padding`); `lyric_font_sizes`
/// does the same for a lyric syllable's leading character.
#[derive(Clone, Copy)]
pub(super) struct RowResolveConfig {
    pub(super) note_number_width: f32,
    pub(super) lyric_font_sizes: LyricFontSizes,
    pub(super) notes_font_size: f32,
    pub(super) chords_font_size: f32,
    label_font_sizes: LabelFontSizes,
    pub(super) paddings: ElementPaddings,
    /// See `Metadata::page_number.vertical_padding_pt` / `resolve_row_element`.
    page_number_vertical_padding_pt: f32,
    /// See `ResolveFontSizes::glyph_font_families`.
    pub(super) glyph_font_families: GlyphFontFamilies,
}

/// A lyric syllable in a row grown for pinyin sits in the row's lower part,
/// below the band the pinyin is drawn in.
fn lyric_ruby_shift(content: &GridContent) -> f32 {
    match content {
        GridContent::LyricSyllable { ruby_band_pt, .. } => ruby_band_pt * 0.5,
        _ => 0.0,
    }
}

fn resolve_row_element(
    el: &GridElement,
    row: &GridRow,
    row_y: f32,
    geometry: &ColumnGeometry,
    config: RowResolveConfig,
) -> Result<Option<AbsoluteElement>, IrrecoverableError> {
    let raw_x_start = geometry.x_start(el.column as f32);
    let x_start = raw_x_start;
    // Computed from the actual start/end columns (not `col_width * span`) so
    // a span crossing measures of differing proportional width still gets
    // its true pixel extent.
    let span_width = geometry.x_start(el.column as f32 + el.column_span as f32) - raw_x_start;
    let x = match el.halign {
        HAlign::Start => x_start,
        HAlign::Center => {
            if is_flush_left_glyph(&el.content) {
                geometry
                    .glyph_left_anchor_x(el.column as f32, flush_left_padding(&el.content, config))
            } else {
                x_start + span_width * 0.5
            }
        }
        HAlign::End => x_start + span_width,
    };
    let bottom_padding = if matches!(el.content, GridContent::DirectiveLine { .. }) {
        crate::font_metrics::DIRECTIVE_LINE_BOTTOM_PADDING
    } else if el.valign == VAlign::Bottom && matches!(el.content, GridContent::Text { .. }) {
        // The only `GridContent::Text` element ever bottom-aligned is the
        // footer page number (see `make_footer_row`) — push it up from the
        // page's bottom edge by `page_number.vertical_padding_pt` instead of
        // growing the footer row itself, which already fills all remaining
        // page height regardless of this padding (see
        // `Metadata::page_number.vertical_padding_pt`).
        config.page_number_vertical_padding_pt
    } else {
        0.0
    };
    let y = match el.valign {
        VAlign::Top => row_y,
        VAlign::Center => row_y + row.height_pt * 0.5,
        VAlign::Bottom => row_y + row.height_pt - bottom_padding,
    } + lyric_ruby_shift(&el.content);

    if let Some(el) = resolve_span_marking(el, y, geometry, config) {
        return Ok(Some(el));
    }
    match &el.content {
        GridContent::MultiMeasureRest { count } => Ok(Some(resolve_multi_measure_rest(
            *count,
            x_start,
            span_width,
            y,
            config.paddings.notes,
        ))),
        GridContent::Rest {
            dotted,
            double_dotted,
            implicit_fill: true,
        } => Ok(Some(resolve_implicit_fill_rest(
            *dotted,
            *double_dotted,
            el.column,
            el.column_span,
            geometry,
            y,
        ))),
        content => {
            let Some(post_arc_content) = to_post_arc_content(content) else {
                return Ok(None);
            };
            Ok(grid_to_absolute(
                &post_arc_content,
                span_width,
                el.halign,
                config.label_font_sizes.part_label_style(),
                config.label_font_sizes.directive_line_font_sizes(),
            )?
            .map(|content| AbsoluteElement { x, y, content }))
        }
    }
}

fn resolve_page(
    page: &GridPage,
    note_number_width: f32,
    page_frame: PageFrame,
    font_sizes: ResolveFontSizes,
) -> Result<AbsolutePage, IrrecoverableError> {
    let page_geometry = PageGeometry {
        frame: page_frame,
        page_width_pt: page.width_pt,
    };
    let mut elements: Vec<AbsoluteElement> = Vec::new();
    let mut row_y = page_frame.margin_pt;
    let mut row_tops: Vec<f32> = Vec::with_capacity(page.rows.len());

    let row_config = RowResolveConfig {
        note_number_width,
        lyric_font_sizes: font_sizes.lyric,
        notes_font_size: font_sizes.notes,
        chords_font_size: font_sizes.chords,
        label_font_sizes: font_sizes.labels,
        paddings: font_sizes.paddings,
        page_number_vertical_padding_pt: font_sizes.page_number_vertical_padding_pt,
        glyph_font_families: font_sizes.glyph_font_families,
    };
    for row in &page.rows {
        row_tops.push(row_y);
        let geometry = page_geometry.row_geometry(row);
        for el in &row.elements {
            if let Some(element) = resolve_row_element(el, row, row_y, &geometry, row_config)? {
                elements.push(element);
            }
        }
        row_y += row.height_pt;
    }

    let mut highlight_elements = resolve_measure_highlights(
        &page.measure_highlights,
        &page.rows,
        &row_tops,
        page_geometry,
    );
    let error_elements =
        resolve_error_highlights(&page.error_highlights, &page.rows, &row_tops, page_geometry);
    highlight_elements.extend(error_elements);
    highlight_elements.extend(elements);
    highlight_elements.extend(resolve_click_target_elements(
        page,
        &row_tops,
        page_geometry,
        font_sizes.labels.measure_number,
        crate::grid_layout::types::TextStyleFlags {
            bold: font_sizes.labels.measure_number_bold,
            italic: font_sizes.labels.measure_number_italic,
            underline: font_sizes.labels.measure_number_underline,
        },
    ));

    Ok(AbsolutePage {
        width_pt: page.width_pt,
        height_pt: page.height_pt,
        elements: highlight_elements,
    })
}
