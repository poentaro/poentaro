use super::header_fit::fit_header_to_width;
use super::types::Header;
use std::borrow::Cow;

/// The page(s) a score is laid out on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageLayout {
    /// Fixed-size pages, each holding as many systems as fit, with the
    /// score's own margins and a page number at the foot of each page. Print,
    /// export and the preview's default view use [`PageSize::A4`].
    Paged(PageSize),
    /// The preview's max view: one page as tall as the score, so it never
    /// breaks, with no page number and only a thin margin.
    Continuous(ContinuousPage),
}

impl PageLayout {
    pub const A4: PageLayout = PageLayout::Paged(PageSize::A4);

    pub fn width_pt(&self) -> f32 {
        match self {
            PageLayout::Paged(size) => size.width_pt,
            PageLayout::Continuous(page) => page.width_pt.max(MIN_CONTINUOUS_WIDTH_PT),
        }
    }

    /// The blank margin around the page's content: the score's own
    /// `page_margin_pt` on a fixed-size page, a thin fixed one on a
    /// continuous page, which sits flush in the preview with no page edge
    /// to keep content away from.
    pub fn margin_pt(&self, score_margin_pt: u32) -> u32 {
        match self {
            PageLayout::Paged(_) => score_margin_pt,
            PageLayout::Continuous(_) => CONTINUOUS_MARGIN_PT,
        }
    }

    /// `header` as this layout shows it, `content_width_pt` wide: a fixed-size
    /// page keeps the score's own sizes, which print as written, while a
    /// continuous page, often as narrow as a phone, shrinks the title and part
    /// legend to fit.
    pub(crate) fn fit_header<'a>(
        &self,
        header: &'a Header,
        content_width_pt: f32,
    ) -> Cow<'a, Header> {
        match self {
            PageLayout::Paged(_) => Cow::Borrowed(header),
            PageLayout::Continuous(_) => Cow::Owned(fit_header_to_width(header, content_width_pt)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageSize {
    pub width_pt: f32,
    pub height_pt: f32,
}

impl PageSize {
    pub const A4: PageSize = PageSize {
        width_pt: 595.0,
        height_pt: 842.0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContinuousPage {
    /// The width systems wrap to, normally the preview's own width.
    pub width_pt: f32,
}

/// A continuous page's margin on every side.
const CONTINUOUS_MARGIN_PT: u32 = 8;

/// The narrowest a continuous page gets, so a collapsed or tiny preview
/// still leaves room for the part labels and a measure.
const MIN_CONTINUOUS_WIDTH_PT: f32 = 160.0;
