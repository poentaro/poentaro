//! Rendering a score under a resolved part visibility (see
//! [`crate::resolve_part_visibility`]): which parts are drawn.
use crate::compiler::NoteLabels;
use crate::error::IrrecoverableError;
use crate::parser::parts_parser::InstrumentInfo;
use crate::{
    apply_track_filter, collect_measure_diagnostics, compile_with_preset, filter_part_list,
    list_parts_from_source, render_svgs_with_parts, RenderOutput, ResolvedPartVisibility,
};

/// Parse, group, apply a resolved part visibility, and render SVG page strings.
///
/// Parts outside `visibility.rendered_tracks` are not rendered; a lyric part
/// shown while its target is hidden still draws against the target's columns.
pub fn render_svgs_from_source_with_visibility(
    source: &str,
    filename: &str,
    visibility: &ResolvedPartVisibility,
    instruments: &[InstrumentInfo],
    note_labels: NoteLabels,
) -> Result<RenderOutput, IrrecoverableError> {
    let enabled_tracks = visibility.rendered_tracks.as_deref();
    let parts = filter_part_list(
        list_parts_from_source(source, filename, instruments)?,
        enabled_tracks,
    );
    let mut score = compile_with_preset(
        source,
        filename,
        instruments,
        visibility.preset_name.as_deref(),
    )?;
    apply_track_filter(&mut score, enabled_tracks);
    let mut diagnostics = collect_measure_diagnostics(&score);
    let result = render_svgs_with_parts(&score, &parts, Some(source), note_labels)?;
    diagnostics.extend(result.diagnostics);
    Ok(RenderOutput {
        svgs: result.svgs,
        diagnostics,
    })
}

/// Parse, group, apply a resolved part visibility, and write PDF bytes.
#[cfg(feature = "pdf")]
pub fn write_pdf_from_source_with_visibility(
    source: &str,
    filename: &str,
    visibility: &ResolvedPartVisibility,
    fonts: &crate::fonts::FontBytesByFamily,
    instruments: &[InstrumentInfo],
    note_labels: NoteLabels,
) -> Result<Vec<u8>, IrrecoverableError> {
    write_pdf_in_order(
        source,
        filename,
        visibility,
        fonts,
        instruments,
        crate::RenderView {
            note_labels,
            ..crate::RenderView::default()
        },
    )
}

/// Write a PDF in the selected score order.
#[cfg(feature = "pdf")]
pub fn write_pdf_in_order(
    source: &str,
    filename: &str,
    visibility: &ResolvedPartVisibility,
    fonts: &crate::fonts::FontBytesByFamily,
    instruments: &[InstrumentInfo],
    view: crate::RenderView,
) -> Result<Vec<u8>, IrrecoverableError> {
    let render_output = crate::render_documents_from_source_with_visibility(
        source,
        filename,
        visibility,
        instruments,
        view,
    )?;
    let svgs = crate::serializer::serialize(&render_output.documents, None);
    crate::pdf::write_pdf(&svgs, fonts, Some(source))
}
