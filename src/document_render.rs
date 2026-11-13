use crate::ast::grouped::Score;
use crate::compiler::NoteLabels;
use crate::error::IrrecoverableError;
use crate::filters::ResolvedPartVisibility;
use crate::grid_layout::PageLayout;
use crate::part_info::PartInfo;

/// Output of a successful render: typed SVG document tree and any diagnostics.
#[derive(Debug)]
pub struct RenderDocumentOutput {
    /// One typed SVG document per page.
    pub documents: Vec<crate::renderer::new_types::SvgDocument>,
    /// Diagnostics collected during grouping (e.g. lyrics underflow).
    pub diagnostics: Vec<crate::error::Diagnostic>,
}

/// Typed SVG document trees plus any diagnostics found during layout (e.g.
/// `WarningKind::MeasureOverflow`) — merged by callers on top of
/// `collect_measure_diagnostics`'s pre-layout, grouped-`Score` diagnostics.
struct DocumentsResult {
    documents: Vec<crate::renderer::new_types::SvgDocument>,
    diagnostics: Vec<crate::error::Diagnostic>,
}

/// Lays `score` out on `page_layout` and renders each page, with
/// `highlighted_measure_ranges` (disjoint, inclusive) highlighted.
fn render_documents(
    score: &Score,
    parts: &[PartInfo],
    highlighted_measure_ranges: Option<Vec<crate::grid_layout::MeasureRange>>,
    note_labels: NoteLabels,
    page_layout: PageLayout,
) -> Result<DocumentsResult, IrrecoverableError> {
    let mut config = crate::render_config::RenderConfig::from_metadata(&score.metadata);
    config.page_margin_pt = page_layout.margin_pt(config.page_margin_pt);
    let header = crate::build_header(score, parts);
    let compile_result =
        crate::compiler::label_note_heads(crate::compiler::compile(score), score, note_labels);
    let compile_result = crate::consolidator::consolidate(compile_result);
    let crate::grid_layout::LayoutOutput {
        pages: grid_pages,
        diagnostics,
    } = crate::grid_layout::layout(
        &compile_result,
        &config,
        &header,
        page_layout,
        highlighted_measure_ranges,
    );
    let abs = crate::coordinate_resolver::resolve(
        &grid_pages,
        config.note_number_width as f32,
        config.page_frame(),
        crate::coordinate_resolver::ResolveFontSizes {
            lyric: config.lyric_font_sizes(),
            notes: config.notes_font_size(),
            chords: config.chords_font_size(),
            labels: crate::coordinate_resolver::LabelFontSizes {
                measure_number: config.measure_number_font_size as f32,
                section_label: config.section_label_font_size as f32,
                section_label_vertical_padding_pt: config.section_label_vertical_padding_pt(),
                part_label: config.part_label_font_size as f32,
                measure_number_bold: config.measure_number_bold,
                measure_number_italic: config.measure_number_italic,
                measure_number_underline: config.measure_number_underline,
                measure_number_font_family: config.measure_number_font_family,
                section_label_bold: config.section_label_bold,
                section_label_italic: config.section_label_italic,
                section_label_underline: config.section_label_underline,
                section_label_font_family: config.section_label_font_family,
                part_label_bold: config.part_label_bold,
                part_label_italic: config.part_label_italic,
                part_label_underline: config.part_label_underline,
                part_label_font_family: config.part_label_font_family,
                sequence_bold: config.sequence_bold,
                sequence_italic: config.sequence_italic,
                sequence_underline: config.sequence_underline,
                sequence_font_family: config.sequence_font_family,
            },
            paddings: config.element_paddings(),
            page_number_vertical_padding_pt: config.page_number_vertical_padding_pt(),
            glyph_font_families: config.glyph_font_families,
        },
    )?;
    Ok(DocumentsResult {
        documents: crate::renderer::new_renderer::render_new(&abs, &config),
        diagnostics,
    })
}

/// How the preview draws a score: what its note heads read and the pages it
/// lands on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderView {
    pub note_labels: NoteLabels,
    pub page_layout: PageLayout,
}

impl Default for RenderView {
    /// Scale degrees on A4 pages, as the score prints.
    fn default() -> Self {
        RenderView {
            note_labels: NoteLabels::default(),
            page_layout: PageLayout::A4,
        }
    }
}

/// Parse, group, apply a resolved part visibility, and return typed SVG document trees.
///
/// Parts outside `visibility.rendered_tracks` are not rendered; a lyric part
/// shown while its target is hidden still draws against the target's columns.
pub fn render_documents_from_source_with_visibility(
    source: &str,
    filename: &str,
    visibility: &ResolvedPartVisibility,
    instruments: &[crate::parser::parts_parser::InstrumentInfo],
    view: RenderView,
) -> Result<RenderDocumentOutput, IrrecoverableError> {
    render_documents_from_source(source, filename, None, visibility, instruments, view)
}

/// Like [`render_documents_from_source_with_visibility`], with `measure_ranges`
/// (disjoint, inclusive) highlighted.
pub fn render_documents_with_highlight_range_and_visibility(
    source: &str,
    filename: &str,
    measure_ranges: &[crate::grid_layout::MeasureRange],
    visibility: &ResolvedPartVisibility,
    instruments: &[crate::parser::parts_parser::InstrumentInfo],
    view: RenderView,
) -> Result<RenderDocumentOutput, IrrecoverableError> {
    render_documents_from_source(
        source,
        filename,
        Some(measure_ranges.to_vec()),
        visibility,
        instruments,
        view,
    )
}

fn render_documents_from_source(
    source: &str,
    filename: &str,
    highlighted_measure_ranges: Option<Vec<crate::grid_layout::MeasureRange>>,
    visibility: &ResolvedPartVisibility,
    instruments: &[crate::parser::parts_parser::InstrumentInfo],
    view: RenderView,
) -> Result<RenderDocumentOutput, IrrecoverableError> {
    let enabled_tracks = visibility.rendered_tracks.as_deref();
    let parts = crate::filter_part_list(
        crate::list_parts_from_source(source, filename, instruments)?,
        enabled_tracks,
    );
    let mut score = crate::compile_with_preset(
        source,
        filename,
        instruments,
        visibility.preset_name.as_deref(),
    )?;
    crate::apply_track_filter(&mut score, enabled_tracks);
    let mut diagnostics = crate::collect_measure_diagnostics(&score);
    let result = render_documents(
        &score,
        &parts,
        highlighted_measure_ranges,
        view.note_labels,
        view.page_layout,
    )?;
    diagnostics.extend(result.diagnostics);
    Ok(RenderDocumentOutput {
        documents: result.documents,
        diagnostics,
    })
}

/// Parse, group, apply a resolved part visibility, and export the score as
/// MEI for Verovio to engrave as staff notation (see `mei_export`). Parts
/// outside `visibility.rendered_tracks` get no staff, chord line or verse.
pub fn render_mei_from_source_with_visibility(
    source: &str,
    filename: &str,
    visibility: &ResolvedPartVisibility,
    instruments: &[crate::parser::parts_parser::InstrumentInfo],
    clef_choice: crate::mei_export::ClefChoice,
) -> Result<crate::mei_export::MeiDocument, IrrecoverableError> {
    let enabled_tracks = visibility.rendered_tracks.as_deref();
    let display_names = crate::filter_part_list(
        crate::list_parts_from_source(source, filename, instruments)?,
        enabled_tracks,
    )
    .into_iter()
    .map(|part| (part.abbreviation, part.display_name))
    .collect();
    let mut score = crate::compile_with_preset(
        source,
        filename,
        instruments,
        visibility.preset_name.as_deref(),
    )?;
    crate::apply_track_filter(&mut score, enabled_tracks);
    Ok(crate::mei_export::score_to_mei(
        &score,
        &display_names,
        clef_choice,
    ))
}
