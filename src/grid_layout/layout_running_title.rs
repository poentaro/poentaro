use crate::grid_layout::layout::{header_gap_row_height, text_row_height};
use crate::grid_layout::types::{GridContent, GridElement, GridRow, HAlign, Header, VAlign};

/// The small title shown alone (no subtitle or author) at the top of every
/// page after the first; `None` when the score has no title.
fn make_running_title_row(header: &Header) -> Option<GridRow> {
    header.title.as_ref().map(|title| GridRow {
        height_pt: text_row_height(
            header.running_title_font_size,
            header.running_title_vertical_padding_pt,
        ),
        column_count: 1,
        has_label_region: false,
        measure_layout: vec![],
        elements: vec![GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::Center,
            valign: VAlign::Center,
            content: GridContent::Text {
                content: title.clone(),
                font_size: header.running_title_font_size,
                bold: header.running_title_bold,
                italic: header.running_title_italic,
                underline: header.running_title_underline,
                font_family: header.running_title_font_family,
            },
        }],
    })
}

/// Every header row of a page after the first: the running title and the
/// blank gap below it; nothing when the score has no title.
pub(super) fn make_running_title_rows(header: &Header, base: f32) -> Vec<GridRow> {
    make_running_title_row(header)
        .into_iter()
        .chain(header.title.as_ref().map(|_| GridRow {
            height_pt: header_gap_row_height(base),
            column_count: 1,
            has_label_region: false,
            measure_layout: vec![],
            elements: vec![],
        }))
        .collect()
}
