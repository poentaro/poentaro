// ── Header rows are sized from their own font, not from `row_height` ────────

use crate::grid_layout::layout::make_header_rows;
use crate::grid_layout::types::{GridContent, GridRow, Header};

const ROW_HEIGHT: f32 = 24.0;

fn header(title_font_size: f32) -> Header {
    Header {
        title: Some("Title".into()),
        subtitle: Some("Subtitle".into()),
        author: Some("Author".into()),
        parts_list_columns: 3,
        title_font_size,
        subtitle_font_size: 19.0,
        author_font_size: 14.0,
        sequence_font_size: 12.0,
        part_legend_font_size: 12.0,
        ..Default::default()
    }
}

fn row_with_text<'a>(rows: &'a [GridRow], text: &str) -> &'a GridRow {
    rows.iter()
        .find(|row| {
            row.elements.iter().any(
                |el| matches!(&el.content, GridContent::Text { content, .. } if content == text),
            )
        })
        .unwrap_or_else(|| panic!("no header row holds {text:?}"))
}

#[test]
fn title_row_is_tall_enough_to_hold_a_large_title() {
    let rows = make_header_rows(&header(60.0), ROW_HEIGHT, true);
    let title_row = row_with_text(&rows, "Title");
    assert!(
        title_row.height_pt >= 60.0,
        "a 60pt title must fit in its own row, got {}pt",
        title_row.height_pt
    );
}

#[test]
fn title_row_shrinks_with_a_smaller_title() {
    let big = make_header_rows(&header(36.0), ROW_HEIGHT, true);
    let small = make_header_rows(&header(18.0), ROW_HEIGHT, true);
    assert!(
        row_with_text(&small, "Title").height_pt < row_with_text(&big, "Title").height_pt,
        "halving the title's font size should shrink its row"
    );
}

#[test]
fn subtitle_author_row_fits_the_taller_of_its_two_texts() {
    let mut big_author = header(36.0);
    big_author.author_font_size = 40.0;
    let rows = make_header_rows(&big_author, ROW_HEIGHT, true);
    let row = row_with_text(&rows, "Subtitle");
    assert!(
        row.height_pt >= 40.0,
        "a 40pt author must fit in the row it shares with the subtitle, got {}pt",
        row.height_pt
    );
}

#[test]
fn a_score_without_header_text_has_no_header_rows() {
    let rows = make_header_rows(
        &Header {
            title: None,
            subtitle: None,
            author: None,
            ..header(36.0)
        },
        ROW_HEIGHT,
        true,
    );
    assert!(
        rows.is_empty(),
        "expected no header rows, got {}",
        rows.len()
    );
}
