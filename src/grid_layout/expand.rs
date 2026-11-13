use crate::compiler::types::{BarLineKind, MeasureBlock, MeasureRow};
use crate::grid_layout::layout::{
    block_column_width, chord_part_sub_row_heights, compute_bar_height, is_chord_only_row,
    is_lyric_row, note_part_sub_row_heights, LyricSizing, SystemBands, LABEL_COLS, MUSIC_START_COL,
};
use crate::grid_layout::types::{
    GridContent, GridElement, GridRow, HAlign, MeasureColumnLayout, VAlign,
};
use std::collections::HashMap;

#[path = "expand_elements.rs"]
mod elements;
use elements::{expand_measure_elements, MeasureRenderParams};

#[path = "expand_lyric.rs"]
mod lyric;
use lyric::expand_lyric_part;
pub(crate) use lyric::LyricPartParams;

pub(crate) struct NotePartParams<'a> {
    pub(crate) part_template: &'a MeasureRow,
    pub(crate) part_idx: usize,
    pub(crate) base: f32,
    pub(crate) column_count: u32,
    pub(crate) bar_height: f32,
    pub(crate) part_arcs: &'a [GridElement],
    /// Tuplet brackets for this part, placed in the topmost sub-row (see
    /// `note_part_sub_row_heights`). Always empty for a chord-only row,
    /// which has no `tuplet_bracket` sub-row.
    pub(crate) part_tuplet_brackets: &'a [GridElement],
    /// Whether this part draws a guitar-diagram band above everything else
    /// in this system (see `SystemBands`).
    pub(crate) has_guitar_diagrams: bool,
    pub(crate) measure_layout: &'a [MeasureColumnLayout],
    /// See `Metadata::notes.vertical_padding_pt` / `note_part_sub_row_heights`.
    pub(crate) notes_vertical_padding_pt: f32,
}

/// A part's sub-rows before any elements are pushed into them, plus the
/// indices locating its bands within them. A chord-only row has no
/// `tuplet_bracket` sub-row, so its topmost musical row is the arc row. A
/// note row without a tuplet in this system also drops that sub-row, so its
/// arc row is topmost too. A note row with a tuplet keeps `tuplet_bracket`
/// topmost, pushing its arc row down one (see `note_part_sub_row_heights`).
/// A part with guitar diagrams in this system puts its guitar-diagram band
/// above all of them, at index 0, pushing every other band down one.
struct PartSubRows {
    rows: Vec<GridRow>,
    /// The `tuplet_bracket` band, when the part has one; otherwise the
    /// topmost musical band.
    tuplet_sub: usize,
    arc_sub: usize,
    head_sub: usize,
    guitar_diagram_sub: Option<usize>,
}

/// Which optional bands one part's sub-rows include (see `PartSubRows`).
#[derive(Clone, Copy)]
struct PartBands {
    is_chord_only: bool,
    has_tuplet: bool,
    has_guitar_diagrams: bool,
}

fn build_part_sub_rows(
    bands: PartBands,
    base: f32,
    column_count: u32,
    measure_layout: &[MeasureColumnLayout],
    notes_vertical_padding_pt: f32,
) -> PartSubRows {
    let PartBands {
        is_chord_only,
        has_tuplet,
        has_guitar_diagrams,
    } = bands;
    let musical_heights: Vec<f32> = if is_chord_only {
        chord_part_sub_row_heights(base).to_vec()
    } else if has_tuplet {
        note_part_sub_row_heights(base, notes_vertical_padding_pt).to_vec()
    } else {
        // No tuplet in this system for this part: drop the `tuplet_bracket`
        // sub-row entirely rather than reserving its height unused (see
        // `note_part_height_pt`, which mirrors this for system-height math).
        note_part_sub_row_heights(base, notes_vertical_padding_pt)[1..].to_vec()
    };
    let guitar_diagram_heights =
        has_guitar_diagrams.then(crate::font_metrics::guitar_diagram::band_height_pt);
    let rows: Vec<GridRow> = guitar_diagram_heights
        .iter()
        .chain(&musical_heights)
        .map(|&h| GridRow {
            height_pt: h,
            column_count,
            has_label_region: true,
            measure_layout: measure_layout.to_vec(),
            elements: vec![],
        })
        .collect();
    let (arc_sub, head_sub) = if is_chord_only {
        (0, 1)
    } else if has_tuplet {
        (1, 3)
    } else {
        (0, 2)
    };
    let offset = usize::from(has_guitar_diagrams);
    PartSubRows {
        rows,
        tuplet_sub: offset,
        arc_sub: arc_sub + offset,
        head_sub: head_sub + offset,
        guitar_diagram_sub: has_guitar_diagrams.then_some(0),
    }
}

pub(crate) fn expand_note_part(
    system: &[MeasureBlock],
    params: &NotePartParams<'_>,
) -> Vec<GridRow> {
    let part_template = params.part_template;
    let part_idx = params.part_idx;
    let base = params.base;
    let column_count = params.column_count;
    let bar_height = params.bar_height;
    let part_arcs = params.part_arcs;
    let is_chord_only = is_chord_only_row(part_template);
    let has_tuplet = !params.part_tuplet_brackets.is_empty();
    let PartSubRows {
        rows: mut sub_rows,
        tuplet_sub,
        arc_sub,
        head_sub,
        guitar_diagram_sub,
    } = build_part_sub_rows(
        PartBands {
            is_chord_only,
            has_tuplet,
            has_guitar_diagrams: params.has_guitar_diagrams,
        },
        base,
        column_count,
        params.measure_layout,
        params.notes_vertical_padding_pt,
    );
    if !part_template.label.is_empty() {
        if let Some(row) = sub_rows.get_mut(head_sub) {
            row.elements.push(GridElement {
                column: 0,
                column_span: LABEL_COLS,
                halign: HAlign::Center,
                valign: VAlign::Center,
                content: GridContent::RowLabel(part_template.label.clone()),
            });
        }
    }
    if part_idx == 0 {
        if let Some(row) = sub_rows.get_mut(0) {
            row.elements.push(GridElement {
                column: LABEL_COLS,
                column_span: 1,
                // `Start`, not `Center`: this column's width tracks each
                // system's musical density, but its x_start is pinned to the
                // fixed-width label region's right edge (see
                // `GridRow::column_geometry`). Centering here would offset
                // the line by half of a density-dependent column width,
                // making it drift left/right between systems even though
                // the label region itself is now fixed-width.
                halign: HAlign::Start,
                valign: VAlign::Top,
                content: GridContent::BarLine {
                    height_pt: bar_height,
                    kind: BarLineKind::Single,
                },
            });
        }
    }
    let mut measure_col_offset: u32 = 0;
    let last_block_idx = system.len().saturating_sub(1);
    for (block_idx, block) in system.iter().enumerate() {
        let col_w = block_column_width(block);
        if let Some(part_row) = block.rows.get(part_idx) {
            expand_measure_elements(
                part_row,
                measure_col_offset,
                &MeasureRenderParams {
                    head_sub,
                    guitar_diagram_sub,
                    bar_height,
                    part_idx,
                    is_last_block: block_idx == last_block_idx,
                },
                &mut sub_rows,
            );
        }
        measure_col_offset += col_w;
    }
    if let Some(row) = sub_rows.get_mut(arc_sub) {
        row.elements.extend_from_slice(part_arcs);
    }
    if has_tuplet {
        if let Some(row) = sub_rows.get_mut(tuplet_sub) {
            row.elements.extend_from_slice(params.part_tuplet_brackets);
        }
    }
    sub_rows
}

/// Convert a system's measures into flat GridRows.
/// Does not include decoration, separator, header, or footer rows.
pub(crate) fn expand_system_to_rows(
    system: &[MeasureBlock],
    base: f32,
    system_arcs: &HashMap<usize, Vec<GridElement>>,
    system_tuplet_brackets: &HashMap<usize, Vec<GridElement>>,
    measure_layout: &[MeasureColumnLayout],
    lyric_sizing: LyricSizing,
) -> Vec<GridRow> {
    let Some(first) = system.first() else {
        return vec![];
    };
    let total_musical_cols: u32 = system.iter().map(block_column_width).sum();
    let column_count = MUSIC_START_COL + total_musical_cols;
    let bands = SystemBands::of(system, system_tuplet_brackets.keys().copied().collect());
    let bar_height = compute_bar_height(first, base, &bands, lyric_sizing);
    let mut all_rows: Vec<GridRow> = Vec::new();
    for (part_idx, part_template) in first.rows.iter().enumerate() {
        if is_lyric_row(part_template) {
            all_rows.push(expand_lyric_part(
                system,
                &LyricPartParams {
                    part_idx,
                    base,
                    column_count,
                    bar_height,
                    measure_layout,
                    lyric_sizing,
                },
            ));
        } else {
            let part_arcs: &[GridElement] =
                system_arcs.get(&part_idx).map_or(&[], |v| v.as_slice());
            let part_tuplet_brackets: &[GridElement] = system_tuplet_brackets
                .get(&part_idx)
                .map_or(&[], |v| v.as_slice());
            all_rows.extend(expand_note_part(
                system,
                &NotePartParams {
                    part_template,
                    part_idx,
                    base,
                    column_count,
                    bar_height,
                    part_arcs,
                    part_tuplet_brackets,
                    has_guitar_diagrams: bands.has_guitar_diagrams(part_idx),
                    measure_layout,
                    notes_vertical_padding_pt: lyric_sizing.notes_vertical_padding_pt,
                },
            ));
        }
    }
    all_rows
}

/// A page's footer row: its page number, in `config`'s page-number style,
/// sized to `height_pt`, whatever is left of the page.
pub(crate) fn make_footer_row(
    config: &crate::render_config::RenderConfig,
    page_num: u32,
    total_pages: u32,
    height_pt: f32,
) -> GridRow {
    GridRow {
        height_pt,
        column_count: 1,
        has_label_region: false,
        measure_layout: vec![],
        elements: vec![GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::Center,
            valign: VAlign::Bottom,
            content: GridContent::Text {
                content: format!("{page_num} / {total_pages}"),
                font_size: config.page_number_font_size as f32,
                bold: config.page_number_bold,
                italic: config.page_number_italic,
                underline: config.page_number_underline,
                font_family: config.page_number_font_family,
            },
        }],
    }
}
