use crate::grid_layout::types::{ColumnGeometry, GridRow};
use crate::grid_layout::PageFrame;

/// One page's frame (margin + part-label column) together with its width:
/// everything needed to turn a row's grid columns into page x-coordinates.
#[derive(Clone, Copy)]
pub(super) struct PageGeometry {
    pub(super) frame: PageFrame,
    pub(super) page_width_pt: f32,
}

impl PageGeometry {
    /// `row`'s column geometry in page coordinates (left margin included).
    pub(super) fn row_geometry(&self, row: &GridRow) -> ColumnGeometry {
        self.frame.row_geometry(row, self.page_width_pt)
    }
}
