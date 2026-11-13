// ── A continuous page shrinks its header to fit, a fixed-size page doesn't ──

use crate::compositor::types::FontFamily;
use crate::font_metrics::text_width_for_family;
use crate::grid_layout::page_layout::{ContinuousPage, PageLayout};
use crate::grid_layout::types::{Header, PartListEntry};
use std::borrow::Cow;

const TITLE: &str = "Minuets in G Major and G Minor";

fn entry(abbreviation: &str, display_name: &str) -> PartListEntry {
    PartListEntry {
        abbreviation: abbreviation.into(),
        display_name: display_name.into(),
    }
}

fn header() -> Header {
    Header {
        title: Some(TITLE.into()),
        subtitle: Some("Christian Petzold".into()),
        author: Some("BWV Anh. 114".into()),
        part_list: vec![
            entry("p", "Pluck (Guzheng, Liuqin, etc)"),
            entry("s", "Sustained (Violin, Harmonica, etc)"),
            entry("c", "Chord (Piano, Guitar, etc)"),
        ],
        parts_list_columns: 3,
        title_font_size: 28.0,
        subtitle_font_size: 16.0,
        author_font_size: 14.0,
        part_legend_font_size: 12.0,
        ..Default::default()
    }
}

fn continuous() -> PageLayout {
    PageLayout::Continuous(ContinuousPage { width_pt: 300.0 })
}

fn fitted(width_pt: f32) -> Header {
    continuous().fit_header(&header(), width_pt).into_owned()
}

#[test]
fn a_fixed_size_page_keeps_the_header_as_written() {
    let header = header();
    assert!(matches!(
        PageLayout::A4.fit_header(&header, 100.0),
        Cow::Borrowed(_)
    ));
}

#[test]
fn a_title_too_wide_for_the_page_shrinks_to_its_width() {
    let fitted = fitted(300.0);
    assert!(fitted.title_font_size < 28.0, "{}", fitted.title_font_size);
    let width = text_width_for_family(FontFamily::default(), TITLE, fitted.title_font_size);
    assert!(width <= 300.01, "title is {width}pt wide");
}

#[test]
fn a_header_that_fits_keeps_its_sizes_and_columns() {
    let fitted = fitted(2000.0);
    assert_eq!(fitted.title_font_size, 28.0);
    assert_eq!(fitted.subtitle_font_size, 16.0);
    assert_eq!(fitted.author_font_size, 14.0);
    assert_eq!(fitted.parts_list_columns, 3);
}

#[test]
fn the_part_legend_drops_columns_until_each_entry_fits() {
    assert_eq!(fitted(300.0).parts_list_columns, 1);
    let two = fitted(500.0).parts_list_columns;
    assert_eq!(two, 2, "500pt holds two of the legend's columns");
}

#[test]
fn a_subtitle_and_author_that_would_overlap_shrink_together() {
    let fitted = fitted(200.0);
    assert!(fitted.subtitle_font_size < 16.0);
    let ratio = fitted.author_font_size / fitted.subtitle_font_size;
    assert!((ratio - 14.0 / 16.0).abs() < 1e-4, "ratio {ratio}");
}
