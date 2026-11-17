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
    let render_output = render_svgs_from_source_with_visibility(
        source,
        filename,
        visibility,
        instruments,
        note_labels,
    )?;
    crate::pdf::write_pdf(&render_output.svgs, fonts, Some(source))
}
