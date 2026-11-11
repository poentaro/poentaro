use crate::compiler::types::{BarLineKind, CompileResult, ElementContent, MeasureBlock};
use crate::error::{Diagnostic, Span, Warning};
use crate::grid_layout::slur_placement::{build_measure_placements, resolve_slur_spans};
use crate::grid_layout::tuplet_placement::resolve_tuplet_spans;
use crate::grid_layout::types::Header;
use crate::grid_layout::types::{
    GridContent, GridElement, GridPage, GridRow, HAlign, MeasureColumnLayout, MeasureRange,
};
use crate::render_config::RenderConfig;
use std::collections::HashMap;

#[path = "layout_row_classification.rs"]
mod row_classification;
pub(crate) use row_classification::{is_chord_only_row, is_lyric_row, sings_along_to};

#[path = "layout_heights.rs"]
mod heights;
pub(crate) use heights::*;

// ── Column width helper ───────────────────────────────────────────────────────

/// Number of columns in a MeasureBlock (BarLine column + 1).
///
/// Takes the `max` `BarLine` column across **all** rows, not just the first:
/// every part's notes normally consume the same total columns (same time
/// signature/duration), so today all rows already agree and this is a no-op
/// change — but a standalone `lyrics` part's row always has its `BarLine` at
/// column `0` regardless of what other parts in the measure need, so relying
/// on `rows.first()` alone would wrongly shrink the block if such a row
/// happened to come first.
pub(crate) fn block_column_width(block: &MeasureBlock) -> u32 {
    block
        .rows
        .iter()
        .filter_map(|row| {
            row.elements
                .iter()
                .find(|e| e.content == ElementContent::BarLine)
        })
        .map(|e| e.column + 1)
        .max()
        .unwrap_or(1)
}

#[path = "layout_spacing.rs"]
mod spacing;
pub(crate) use spacing::build_measure_column_layout;
#[cfg(test)]
pub(crate) use spacing::measure_column_weights;

#[path = "layout_system_bands.rs"]
mod system_bands;
pub(crate) use system_bands::{system_bands, SystemBands};

#[path = "layout_systems.rs"]
mod systems;
pub(crate) use systems::{
    compute_bar_height, pack_into_systems, system_has_any_decoration, system_lyric_height_pt,
    system_musical_height_pt, LABEL_COLS, MUSIC_START_COL,
};

#[path = "layout_decoration.rs"]
mod decoration;
#[path = "layout_page_heights.rs"]
mod page_heights;
#[path = "layout_running_title.rs"]
mod running_title;
pub(crate) use super::expand::expand_system_to_rows;
use super::expand::make_footer_row;
pub(crate) use decoration::{
    directive_line_rod_width, directive_line_should_emit, make_header_rows,
};
use decoration::{make_decoration_row, make_separator_row};
use page_heights::PageUsableHeights;

fn system_total_height(
    system: &[MeasureBlock],
    base: f32,
    bands: &SystemBands,
    lyric_sizing: LyricSizing,
) -> f32 {
    let Some(first) = system.first() else {
        return 0.0;
    };
    let musical =
        system_musical_height_pt(first, base, bands, lyric_sizing.notes_vertical_padding_pt);
    let lyric = system_lyric_height_pt(first, base, lyric_sizing);
    let deco = if system_has_any_decoration(system) {
        crate::font_metrics::directive_line_row_height()
    } else {
        0.0
    };
    musical + lyric + deco
}

/// One system's summed column rods against the page's usable music width —
/// the overflow check for the spring-and-rod spacing model (see the
/// `column_geometry` docs). `None` when the system fits; `Some` with
/// a `Diagnostic::Warning(WarningKind::MeasureOverflow)` spanning the union
/// of `system`'s measures' source spans otherwise, so the editor can point
/// at the real source text behind an overflowing system.
fn system_overflow_diagnostic(
    system: &[MeasureBlock],
    measure_layout: &[MeasureColumnLayout],
    page_width_pt: f32,
    config: &RenderConfig,
) -> Option<Diagnostic> {
    let total_rod: f32 = measure_layout.iter().map(|m| m.rod_pt).sum();
    let available = config.page_frame().usable_music_width_pt(page_width_pt);
    if total_rod <= available {
        return None;
    }
    let span = system
        .iter()
        .map(|block| block.source_span)
        .reduce(|a, b| Span::new(a.start.min(b.start), a.end.max(b.end)))?;
    Some(Diagnostic::Warning(Warning::measure_overflow(
        span, total_rod, available,
    )))
}

/// Turns the score's closing bar line into a [`BarLineKind::Final`] double
/// bar. `rows` must be the expanded rows of the score's last system, whose
/// only `HAlign::End` bar line is its last measure's closing one (every other
/// closing bar line is `HAlign::Center`, and the system's leading one is
/// `HAlign::Start` — see `expand_elements::MeasureRenderParams::is_last_block`).
pub(crate) fn mark_final_bar_line(rows: &mut [GridRow]) {
    rows.iter_mut()
        .flat_map(|row| row.elements.iter_mut())
        .filter(|e| e.halign == HAlign::End)
        .for_each(|e| {
            if let GridContent::BarLine { kind, .. } = &mut e.content {
                *kind = BarLineKind::Final;
            }
        });
}

#[derive(Clone, Copy)]
struct PageRowsParams<'a> {
    systems: &'a [Vec<MeasureBlock>],
    header: &'a Header,
    config: &'a RenderConfig,
    arc_map: &'a HashMap<(usize, usize), Vec<GridElement>>,
    tuplet_bracket_map: &'a HashMap<(usize, usize), Vec<GridElement>>,
    abs_system_index_start: usize,
    is_first_page: bool,
    is_last_page: bool,
    page_width_pt: f32,
}

/// [`build_page_rows`]'s output: the page's rows plus any overflow
/// diagnostics found while laying out its systems.
struct PageRowsResult {
    rows: Vec<GridRow>,
    diagnostics: Vec<Diagnostic>,
}

fn build_page_rows(params: &PageRowsParams<'_>) -> PageRowsResult {
    let PageRowsParams {
        systems,
        header,
        config,
        arc_map,
        tuplet_bracket_map,
        abs_system_index_start,
        is_first_page,
        is_last_page,
        page_width_pt,
    } = *params;
    let base = config.row_height as f32;
    let mut rows: Vec<GridRow> = make_header_rows(header, base, is_first_page);
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for (sys_idx, system) in systems.iter().enumerate() {
        if sys_idx > 0 && !config.hide_system_dividers {
            rows.push(make_separator_row());
        }
        let Some(first) = system.first() else {
            continue;
        };
        let measure_layout = build_measure_column_layout(system, config);
        diagnostics.extend(system_overflow_diagnostic(
            system,
            &measure_layout,
            page_width_pt,
            config,
        ));
        if system_has_any_decoration(system) {
            rows.push(make_decoration_row(system, &measure_layout));
        }
        let abs_sys = abs_system_index_start + sys_idx;
        let system_arcs: HashMap<usize, Vec<GridElement>> = first
            .rows
            .iter()
            .enumerate()
            .filter_map(|(consolidated_idx, row)| {
                arc_map
                    .get(&(abs_sys, row.source_part_index))
                    .map(|arcs| (consolidated_idx, arcs.clone()))
            })
            .collect();
        let system_tuplet_brackets: HashMap<usize, Vec<GridElement>> = first
            .rows
            .iter()
            .enumerate()
            .filter_map(|(consolidated_idx, row)| {
                tuplet_bracket_map
                    .get(&(abs_sys, row.source_part_index))
                    .map(|brackets| (consolidated_idx, brackets.clone()))
            })
            .collect();
        let mut system_rows = expand_system_to_rows(
            system,
            base,
            &system_arcs,
            &system_tuplet_brackets,
            &measure_layout,
            config.lyric_sizing(),
        );
        if is_last_page && sys_idx + 1 == systems.len() {
            mark_final_bar_line(&mut system_rows);
        }
        rows.extend(system_rows);
    }
    PageRowsResult { rows, diagnostics }
}

use super::click_targets::{compute_highlight_and_click_infos, HighlightAndClickInfosParams};
#[cfg(test)]
pub(crate) use super::highlight::compute_measure_highlight_location;

/// Greedily packs `systems` into pages, each page holding as many systems as
/// fit within `usable_h` (accounting for inter-system separator gaps). Split
/// out of `layout` to keep that function under its line-count cap.
fn pack_page_systems(
    systems: Vec<Vec<MeasureBlock>>,
    tuplet_bracket_map: &HashMap<(usize, usize), Vec<GridElement>>,
    base: f32,
    usable_heights: PageUsableHeights,
    hide_system_dividers: bool,
    lyric_sizing: LyricSizing,
) -> Vec<Vec<Vec<MeasureBlock>>> {
    let mut page_systems: Vec<Vec<Vec<MeasureBlock>>> = Vec::new();
    let mut current_page: Vec<Vec<MeasureBlock>> = Vec::new();
    let mut used_h: f32 = 0.0;

    for (abs_sys, system) in systems.into_iter().enumerate() {
        let bands = system_bands(&system, tuplet_bracket_map, abs_sys);
        let sys_h = system_total_height(&system, base, &bands, lyric_sizing);
        let gap = if current_page.is_empty() || hide_system_dividers {
            0.0
        } else {
            separator_row_height()
        };
        let usable_h = usable_heights.for_page(page_systems.len());
        if !current_page.is_empty() && used_h + gap + sys_h > usable_h {
            page_systems.push(std::mem::take(&mut current_page));
            used_h = 0.0;
        }
        used_h += gap + sys_h;
        current_page.push(system);
    }
    page_systems.push(current_page);
    page_systems
}

/// [`layout`]'s return value: the laid-out pages plus any diagnostics found
/// during layout itself (currently just `WarningKind::MeasureOverflow`,
/// from [`system_overflow_diagnostic`]) — distinct from the diagnostics
/// `collect_measure_diagnostics` gathers from the pre-layout, grouped
/// `Score`, which has no visibility into layout-stage spacing.
pub struct LayoutOutput {
    pub pages: Vec<GridPage>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Public entry point: convert compiler blocks to GridPages.
pub fn layout(
    compile_result: &CompileResult,
    config: &RenderConfig,
    header: &Header,
    page_width_pt: f32,
    page_height_pt: f32,
    highlighted_measure_ranges: Option<Vec<MeasureRange>>,
) -> LayoutOutput {
    let base = config.row_height as f32;
    let blocks = &compile_result.blocks;
    let frame = config.page_frame();
    let systems = pack_into_systems(blocks, config, frame.usable_music_width_pt(page_width_pt));

    let measure_placements = build_measure_placements(&systems);
    let arc_map = resolve_slur_spans(&compile_result.slur_spans, &measure_placements, &systems);
    let tuplet_bracket_map =
        resolve_tuplet_spans(&compile_result.tuplet_spans, &measure_placements);

    let page_systems = pack_page_systems(
        systems,
        &tuplet_bracket_map,
        base,
        PageUsableHeights::new(header, config, page_height_pt),
        config.hide_system_dividers,
        config.lyric_sizing(),
    );

    let highlight_and_click_infos =
        compute_highlight_and_click_infos(&HighlightAndClickInfosParams {
            blocks,
            lyric_links: &compile_result.lyric_links,
            page_systems: &page_systems,
            tuplet_bracket_map: &tuplet_bracket_map,
            header,
            base,
            hide_system_dividers: config.hide_system_dividers,
            highlighted_measure_ranges,
        });

    let total_pages = page_systems.len() as u32;
    let mut abs_system_index_start: usize = 0;
    let mut pages: Vec<GridPage> = Vec::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for (page_idx, page_sys) in page_systems.into_iter().enumerate() {
        let PageRowsResult {
            mut rows,
            diagnostics: page_diagnostics,
        } = build_page_rows(&PageRowsParams {
            systems: &page_sys,
            header,
            config,
            arc_map: &arc_map,
            tuplet_bracket_map: &tuplet_bracket_map,
            abs_system_index_start,
            is_first_page: page_idx == 0,
            is_last_page: page_idx as u32 + 1 == total_pages,
            page_width_pt,
        });
        diagnostics.extend(page_diagnostics);
        let body_height: f32 = rows.iter().map(|r| r.height_pt).sum();
        let remaining_height = frame.usable_height_pt(page_height_pt) - body_height;
        rows.push(page_footer_row(
            config,
            page_idx as u32 + 1,
            total_pages,
            remaining_height,
        ));
        abs_system_index_start += page_sys.len();
        let page_targets = PageHighlightsAndTargets::for_page(&highlight_and_click_infos, page_idx);
        pages.push(GridPage {
            width_pt: page_width_pt,
            height_pt: page_height_pt,
            rows,
            measure_highlights: page_targets.measure_highlights,
            error_highlights: page_targets.error_highlights,
            measure_click_targets: page_targets.measure_click_targets,
            playback_cursor_targets: page_targets.playback_cursor_targets,
            part_label_click_targets: page_targets.part_label_click_targets,
            lyric_click_targets: page_targets.lyric_click_targets,
            lyric_label_click_targets: page_targets.lyric_label_click_targets,
            bar_number_click_targets: page_targets.bar_number_click_targets,
            bar_line_click_targets: page_targets.bar_line_click_targets,
        });
    }
    LayoutOutput { pages, diagnostics }
}

/// Builds a page's footer row from `config`'s page-number style — split out
/// of [`layout`] to keep it under clippy's line-count limit.
fn page_footer_row(
    config: &RenderConfig,
    page_num: u32,
    total_pages: u32,
    remaining_height: f32,
) -> GridRow {
    make_footer_row(
        page_num,
        total_pages,
        config.page_number_font_size as f32,
        crate::grid_layout::types::TextStyleFlags {
            bold: config.page_number_bold,
            italic: config.page_number_italic,
            underline: config.page_number_underline,
        },
        config.page_number_font_family,
        remaining_height,
    )
}

#[path = "layout_page_targets.rs"]
mod page_targets;
use page_targets::PageHighlightsAndTargets;

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;
