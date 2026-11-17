//! Shrinks a score's header to fit a narrow continuous page, where a title
//! or part legend sized for A4 would run off the edge or overlap.

use super::types::{Header, PartListEntry};
use crate::compositor::types::FontFamily;
use crate::font_metrics::text_width_for_family;

/// Bold text is measured with the regular face, so allow for its wider
/// strokes.
const BOLD_WIDTH_ALLOWANCE: f32 = 1.08;

/// `header` with its title, subtitle and author shrunk until each fits
/// `width_pt`, and its part legend down to as many columns as fit.
pub(crate) fn fit_header_to_width(header: &Header, width_pt: f32) -> Header {
    let width_of = |text: &Option<String>, family: FontFamily, font_size: f32, bold: bool| {
        text.as_deref().map_or(0.0, |text| {
            text_width_for_family(family, text, font_size)
                * if bold { BOLD_WIDTH_ALLOWANCE } else { 1.0 }
        })
    };
    let title_width = width_of(
        &header.title,
        header.title_font_family,
        header.title_font_size,
        header.title_bold,
    );
    let subtitle_width = width_of(
        &header.subtitle,
        header.subtitle_font_family,
        header.subtitle_font_size,
        header.subtitle_bold,
    );
    let author_width = width_of(
        &header.author,
        header.author_font_family,
        header.author_font_size,
        header.author_bold,
    );
    // The subtitle is centred and the author flush right, so they clear
    // each other while the subtitle's half and the author fit half the row.
    let byline_width = if header.subtitle.is_some() {
        subtitle_width + 2.0 * author_width
    } else {
        author_width
    };
    let byline_scale = scale_to_fit(byline_width, width_pt);
    Header {
        title_font_size: header.title_font_size * scale_to_fit(title_width, width_pt),
        subtitle_font_size: header.subtitle_font_size * byline_scale,
        author_font_size: header.author_font_size * byline_scale,
        parts_list_columns: legend_columns(header, width_pt),
        ..header.clone()
    }
}

fn scale_to_fit(content_width_pt: f32, width_pt: f32) -> f32 {
    if content_width_pt > width_pt {
        width_pt / content_width_pt
    } else {
        1.0
    }
}

/// The most legend columns, up to the score's own count, whose widest entry
/// still fits its column with a gap after it; at least one.
fn legend_columns(header: &Header, width_pt: f32) -> u32 {
    let widest_entry = header
        .part_list
        .iter()
        .map(|entry| {
            text_width_for_family(
                header.part_legend_font_family,
                &entry.legend_text(),
                header.part_legend_font_size,
            ) * if header.part_legend_bold {
                BOLD_WIDTH_ALLOWANCE
            } else {
                1.0
            }
        })
        .fold(0.0, f32::max);
    let gap = header.part_legend_font_size;
    (1..=header.parts_list_columns)
        .rev()
        .find(|&columns| widest_entry + gap <= width_pt / columns as f32)
        .unwrap_or(1)
}

impl PartListEntry {
    /// The entry as the part legend shows it, e.g. `S — Soprano`.
    pub(crate) fn legend_text(&self) -> String {
        format!("{} \u{2014} {}", self.abbreviation, self.display_name)
    }
}
