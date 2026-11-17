pub(crate) mod click_targets;
pub(crate) mod expand;
mod header_fit;
pub(crate) mod highlight;
pub mod layout;
pub mod page_layout;
pub(crate) mod playback_cursor;
pub(crate) mod slur_placement;
pub(crate) mod system_walk;
pub(crate) mod tuplet_placement;
pub mod types;

pub use layout::{layout, LayoutOutput};
pub use page_layout::{ContinuousPage, PageLayout, PageSize};
pub use types::{
    GridContent, GridElement, GridPage, GridRow, HAlign, Header, MeasureRange, PostArcGridContent,
    VAlign,
};

/// Where a page's content sits: the blank margin on every side of the page
/// (`Metadata::page_margin_pt`) and the fixed-width part-label column at the
/// start of each system (`Metadata::part_label_width_pt`). Shared by
/// `layout::layout` (system width, page breaks) and
/// `coordinate_resolver::resolve` (absolute positions), so the width a system
/// is packed into is the same width its columns are later spread across.
#[derive(Debug, Clone, Copy)]
pub struct PageFrame {
    pub margin_pt: f32,
    pub part_label_width_pt: f32,
}

impl PageFrame {
    /// The page width minus the margin on both sides.
    pub(crate) fn usable_width_pt(&self, page_width_pt: f32) -> f32 {
        page_width_pt - 2.0 * self.margin_pt
    }

    /// The page height minus the margin at the top and bottom.
    pub(crate) fn usable_height_pt(&self, page_height_pt: f32) -> f32 {
        page_height_pt - 2.0 * self.margin_pt
    }

    /// The music region's width: the usable width minus the part-label
    /// column.
    pub(crate) fn usable_music_width_pt(&self, page_width_pt: f32) -> f32 {
        self.usable_width_pt(page_width_pt) - self.part_label_width_pt
    }

    /// `row`'s column geometry placed on a page `page_width_pt` wide, so its
    /// `x_start` returns page coordinates (left margin included).
    pub(crate) fn row_geometry(&self, row: &GridRow, page_width_pt: f32) -> types::ColumnGeometry {
        row.column_geometry(
            self.usable_width_pt(page_width_pt),
            self.part_label_width_pt,
        )
        .offset_by(self.margin_pt)
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_accidental_spacing;
#[cfg(test)]
mod tests_column_rod_directive_line;
#[cfg(test)]
mod tests_column_rod_spring;
#[cfg(test)]
mod tests_column_span_weight;
#[cfg(test)]
mod tests_dot_spacing;
#[cfg(test)]
mod tests_header_fit;
#[cfg(test)]
mod tests_header_rows;
#[cfg(test)]
mod tests_letter_name_spacing;
#[cfg(test)]
mod tests_measure_spacing;
#[cfg(test)]
mod tests_measure_spacing_multi_measure_rest;
#[cfg(test)]
mod tests_system_break;
