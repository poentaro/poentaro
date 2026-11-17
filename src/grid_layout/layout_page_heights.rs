use super::make_header_rows;
use crate::grid_layout::page_layout::PageLayout;
use crate::grid_layout::types::Header;
use crate::render_config::RenderConfig;

/// Height (points) available to systems on the first page versus every later
/// page, which differ because later pages carry only the small running title.
/// A continuous page has no height limit, so every system lands on it.
#[derive(Clone, Copy)]
pub(super) struct PageUsableHeights {
    first_page: f32,
    later_pages: f32,
}

impl PageUsableHeights {
    pub(super) fn new(header: &Header, config: &RenderConfig, page_layout: PageLayout) -> Self {
        let page_height_pt = match page_layout {
            PageLayout::Paged(size) => size.height_pt,
            PageLayout::Continuous(_) => f32::INFINITY,
        };
        let base = config.row_height as f32;
        let below_header = |is_first_page: bool| {
            let header_h: f32 = make_header_rows(header, base, is_first_page)
                .iter()
                .map(|r| r.height_pt)
                .sum();
            config.page_frame().usable_height_pt(page_height_pt) - header_h - base * 0.40
        };
        Self {
            first_page: below_header(true),
            later_pages: below_header(false),
        }
    }

    pub(super) fn for_page(self, page_index: usize) -> f32 {
        if page_index == 0 {
            self.first_page
        } else {
            self.later_pages
        }
    }
}
