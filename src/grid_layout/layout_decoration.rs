use super::running_title::make_running_title_rows;
use crate::compiler::types::{Decoration, MeasureBlock};
use crate::compositor::types::TextSpan;
use crate::grid_layout::layout::{
    block_column_width, decoration_row_height, header_gap_row_height, separator_row_height,
    text_row_height, LABEL_COLS, MUSIC_START_COL,
};
use crate::grid_layout::types::{
    GridContent, GridElement, GridRow, HAlign, Header, MeasureColumnLayout, PartListEntry, VAlign,
};

/// Whether `block`'s (index `index` within its system) `DirectiveLine`
/// decoration actually draws a line — the system's first block always does;
/// a later block only does when it changes something a reader needs to see
/// (a label, or a key/bpm/time-signature change), so an otherwise-plain
/// mid-system measure doesn't grow a redundant, bare bar number. Shared with
/// `click_targets::compute_all_bar_number_click_targets`, which needs to
/// know the same thing to place each drawn bar number's own click target.
pub(crate) fn directive_line_should_emit(index: usize, dec: &Decoration) -> bool {
    index == 0 || {
        let Decoration::DirectiveLine {
            label,
            key,
            bpm,
            time_signature,
            ..
        } = dec;
        label.is_some() || key.is_some() || bpm.is_some() || time_signature.is_some()
    }
}

/// The rendered width (points) `dec`'s directive line will need once drawn
/// — computed the same way `content_conversion::directive_line_content`
/// positions the actual line, just against `Decoration`'s pre-render fields
/// instead of the resolved `PostArcGridContent`. Keep the span text built
/// here in sync with `build_directive_line_spans` in
/// `coordinate_resolver/content_conversion.rs` if a directive field is ever
/// added/changed — both build the same line from the same source data, one
/// before layout, one after.
pub(crate) fn directive_line_rod_width(
    dec: &Decoration,
    measure_number_font_size: f32,
    section_label_font_size: f32,
    section_label_bold: bool,
) -> f32 {
    let Decoration::DirectiveLine {
        label,
        bar_number,
        key,
        bpm,
        time_signature,
    } = dec;
    let bar_number_span = bar_number.map(|n| TextSpan {
        content: n.to_string(),
        bold: false,
        italic: false,
        underline: false,
        font_size: measure_number_font_size,
    });
    let mut spans = Vec::new();
    if let Some(key_str) = key {
        spans.push(TextSpan {
            content: format!("  {key_str}"),
            bold: false,
            italic: false,
            underline: false,
            font_size: 12.0,
        });
    }
    if let Some(b) = bpm {
        spans.push(TextSpan {
            content: format!("  \u{2669}={b}"),
            bold: false,
            italic: false,
            underline: false,
            font_size: 12.0,
        });
    }
    if let Some((n, d)) = time_signature {
        spans.push(TextSpan {
            content: format!("  {n}/{d}"),
            bold: false,
            italic: false,
            underline: false,
            font_size: 12.0,
        });
    }
    crate::font_metrics::directive_line_width(
        bar_number_span.as_ref(),
        label.as_deref(),
        &spans,
        section_label_font_size,
        section_label_bold,
    )
}

fn directive_line_element(dec: &Decoration, col: u32) -> GridElement {
    let Decoration::DirectiveLine {
        label,
        bar_number,
        key,
        bpm,
        time_signature,
    } = dec;
    GridElement {
        column: col,
        column_span: 1,
        halign: HAlign::Start,
        valign: VAlign::Bottom,
        content: GridContent::DirectiveLine {
            label: label.clone(),
            bar_number: *bar_number,
            key: key.clone(),
            bpm: *bpm,
            time_signature: *time_signature,
        },
    }
}

pub(super) fn make_decoration_row(
    system: &[MeasureBlock],
    measure_layout: &[MeasureColumnLayout],
) -> GridRow {
    let total_musical_cols: u32 = system.iter().map(block_column_width).sum();
    let music_column_count = MUSIC_START_COL + total_musical_cols;
    let mut elements: Vec<GridElement> = Vec::new();

    // Each block's DirectiveLine is aligned to its own leading barline column
    // (for non-first blocks, this is the same column as the previous block's
    // ending barline) rather than to the first musical column. The barline
    // column is the one place in the grid whose x-position doesn't depend on
    // how many musical columns this system happens to contain (see
    // `ColumnGeometry::x_start`: at `column == LABEL_COLS` the density-scaled
    // term is zero), so anchoring there keeps a measure's label at a
    // consistent position relative to its own bar line across systems.
    //
    // For non-first blocks, only emit a DirectiveLine when there is a label
    // or another directive change.
    let mut leading_barline_col = LABEL_COLS;
    for (index, block) in system.iter().enumerate() {
        if let Some(dec) = block.decorations.first() {
            if directive_line_should_emit(index, dec) {
                elements.push(directive_line_element(dec, leading_barline_col));
            }
        }
        leading_barline_col += block_column_width(block);
    }

    GridRow {
        height_pt: crate::font_metrics::directive_line_row_height(),
        column_count: music_column_count,
        has_label_region: true,
        measure_layout: measure_layout.to_vec(),
        elements,
    }
}

pub(super) fn make_separator_row() -> GridRow {
    GridRow {
        height_pt: separator_row_height(),
        column_count: 1,
        has_label_region: false,
        measure_layout: vec![],
        elements: vec![GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::Start,
            valign: VAlign::Center,
            content: GridContent::HorizontalLine,
        }],
    }
}

fn make_sequence_rows(header: &Header, base: f32) -> Vec<GridRow> {
    header
        .sequence
        .as_ref()
        .map(|entries| {
            vec![
                GridRow {
                    height_pt: header_gap_row_height(base),
                    column_count: 1,
                    has_label_region: false,
                    measure_layout: vec![],
                    elements: vec![],
                },
                GridRow {
                    height_pt: decoration_row_height(base)
                        + 2.0 * header.sequence_vertical_padding_pt,
                    column_count: 1,
                    has_label_region: false,
                    measure_layout: vec![],
                    elements: vec![GridElement {
                        column: 0,
                        column_span: 1,
                        halign: HAlign::Start,
                        valign: VAlign::Center,
                        content: GridContent::SequenceLine {
                            entries: entries.clone(),
                            font_size: header.sequence_font_size,
                        },
                    }],
                },
            ]
        })
        .unwrap_or_default()
}

fn make_title_row(header: &Header) -> Option<GridRow> {
    header.title.as_ref().map(|title| GridRow {
        height_pt: text_row_height(header.title_font_size, header.title_vertical_padding_pt),
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
                font_size: header.title_font_size,
                bold: header.title_bold,
                italic: header.title_italic,
                underline: header.title_underline,
                font_family: header.title_font_family,
            },
        }],
    })
}

/// The subtitle (centred) and author (right-aligned) share one row, sized to
/// the taller of the two; `None` when the score sets neither.
fn make_subtitle_author_row(header: &Header) -> Option<GridRow> {
    let mut elements: Vec<GridElement> = Vec::new();
    if let Some(subtitle) = &header.subtitle {
        elements.push(GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::Center,
            valign: VAlign::Center,
            content: GridContent::Text {
                content: subtitle.clone(),
                font_size: header.subtitle_font_size,
                bold: header.subtitle_bold,
                italic: header.subtitle_italic,
                underline: header.subtitle_underline,
                font_family: header.subtitle_font_family,
            },
        });
    }
    if let Some(author) = &header.author {
        elements.push(GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::End,
            valign: VAlign::Center,
            content: GridContent::Text {
                content: author.clone(),
                font_size: header.author_font_size,
                bold: header.author_bold,
                italic: header.author_italic,
                underline: header.author_underline,
                font_family: header.author_font_family,
            },
        });
    }
    let padding = header
        .subtitle_vertical_padding_pt
        .max(header.author_vertical_padding_pt);
    let subtitle_height = header
        .subtitle
        .as_ref()
        .map(|_| text_row_height(header.subtitle_font_size, padding));
    let author_height = header
        .author
        .as_ref()
        .map(|_| text_row_height(header.author_font_size, padding));
    let height_pt = subtitle_height
        .into_iter()
        .chain(author_height)
        .reduce(f32::max)?;
    Some(GridRow {
        height_pt,
        column_count: 1,
        has_label_region: false,
        measure_layout: vec![],
        elements,
    })
}

/// Blank space separating the title/subtitle/author block from the part
/// list or the music below it; omitted when the score has no header text.
fn make_header_gap_row(header: &Header, base: f32) -> Option<GridRow> {
    let has_header_text =
        header.title.is_some() || header.subtitle.is_some() || header.author.is_some();
    has_header_text.then(|| GridRow {
        height_pt: header_gap_row_height(base),
        column_count: 1,
        has_label_region: false,
        measure_layout: vec![],
        elements: vec![],
    })
}

/// The rows above the music: the full header (title, subtitle, author, part
/// list, `sequence` line) on the first page, only the running title (and
/// the gap below it) on every later page.
pub(crate) fn make_header_rows(header: &Header, base: f32, is_first_page: bool) -> Vec<GridRow> {
    if !is_first_page {
        return make_running_title_rows(header, base);
    }
    let entries: Vec<&PartListEntry> = header
        .part_list
        .iter()
        .filter(|entry| entry.abbreviation != entry.display_name)
        .collect();
    let part_list_rows = make_part_list_rows(&entries, header);

    make_title_row(header)
        .into_iter()
        .chain(make_subtitle_author_row(header))
        .chain(make_header_gap_row(header, base))
        .chain(part_list_rows)
        .chain(make_sequence_rows(header, base))
        .collect()
}

fn make_part_list_rows(entries: &[&PartListEntry], header: &Header) -> Vec<GridRow> {
    entries
        .chunks(header.parts_list_columns as usize)
        .map(|chunk| GridRow {
            height_pt: text_row_height(
                header.part_legend_font_size,
                header.part_legend_vertical_padding_pt,
            ),
            column_count: header.parts_list_columns,
            has_label_region: false,
            measure_layout: vec![],
            elements: chunk
                .iter()
                .enumerate()
                .map(|(col_idx, entry)| GridElement {
                    column: col_idx as u32,
                    column_span: 1,
                    halign: HAlign::Start,
                    valign: VAlign::Center,
                    content: GridContent::Text {
                        content: format!("{} \u{2014} {}", entry.abbreviation, entry.display_name),
                        font_size: header.part_legend_font_size,
                        bold: header.part_legend_bold,
                        italic: header.part_legend_italic,
                        underline: header.part_legend_underline,
                        font_family: header.part_legend_font_family,
                    },
                })
                .collect(),
        })
        .collect()
}
