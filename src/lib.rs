#![cfg_attr(test, allow(clippy::disallowed_macros))]
#![forbid(dead_code)]
#![forbid(unused_variables)]
#![forbid(clippy::too_many_lines)]
#![forbid(clippy::indexing_slicing)]
#![forbid(clippy::too_many_arguments)]
#![forbid(clippy::wildcard_imports)]
#![forbid(clippy::type_complexity)]

#[macro_use]
mod keyword_enum;

pub mod ast;
#[cfg(feature = "midi")]
mod audio_source;
mod chord_lyric_placement;
#[cfg(feature = "cli")]
pub mod cli;
pub mod combiner;
pub mod compiler;
pub mod compositor;
pub mod consolidator;
pub mod coordinate_resolver;
pub mod desugar;
pub mod document;
mod document_render;
pub mod error;
pub mod error_reporter;
pub mod file_tree;
pub mod filters;
mod font_metrics;
pub mod fonts;
pub mod format_anyfix;
pub mod format_source;
pub mod grid_layout;
pub mod grouper;
pub mod grouping;
pub mod highlight;
pub mod layout;
mod lyric_slots;
pub mod lyric_spans;
pub mod measure_spans;
pub mod mei_export;
pub mod note_spans;
pub mod parser;
mod part_info;
pub mod part_stream;
pub mod pitch_description;
pub mod render_config;
pub mod renderer;
pub mod serializer;
pub mod sound_label;
mod sounding_pitch;
pub mod source_edit;
pub mod source_embed;
pub mod span_selection;
pub mod split_track;
pub mod staff_export;
pub mod symbols;
mod tuplet;
pub mod utils;
mod visibility_render;

#[cfg(feature = "midi")]
pub mod midi;
#[cfg(feature = "pdf")]
pub mod pdf;
#[cfg(feature = "wav")]
pub mod wav;

#[cfg(feature = "midi")]
pub use audio_source::*;
pub use compiler::NoteLabels;
pub use document_render::{
    render_documents_from_source_with_visibility,
    render_documents_with_highlight_range_and_visibility, render_mei_from_source_with_visibility,
    RenderDocumentOutput,
};
pub use filters::*;
pub use lyric_spans::*;
pub use measure_spans::*;
pub use note_spans::*;
pub use part_info::{
    list_part_declarations_from_source, list_parts_from_source, PartInfo, SourcePartDeclaration,
};
pub use span_selection::*;
pub use split_track::*;
pub use visibility_render::*;

use ast::grouped::Score;
use error::{Diagnostic, IrrecoverableError};
use parser::parts_parser::InstrumentInfo;

/// Output of a successful render: SVG page strings and any diagnostics.
#[derive(Debug)]
pub struct RenderOutput {
    /// One SVG string per page.
    pub svgs: Vec<String>,
    /// Diagnostics collected during grouping (e.g. lyrics underflow).
    /// The SVGs already contain colored overlays for affected measures; these
    /// diagnostics let callers surface them in editor view zones as well.
    pub diagnostics: Vec<Diagnostic>,
}

/// All diagnostics for a score: document-level plus per-measure.
pub fn collect_measure_diagnostics(score: &Score) -> Vec<Diagnostic> {
    score
        .document_diagnostics
        .iter()
        .cloned()
        .chain(
            score
                .measures
                .iter()
                .flat_map(|m| m.diagnostics.iter().cloned()),
        )
        .collect()
}

/// Parse and group a `.poentaro` source string into a [`Score`].
pub fn compile(
    source: &str,
    filename: &str,
    instruments: &[InstrumentInfo],
) -> Result<Score, IrrecoverableError> {
    let doc = parser::parse(source, filename, instruments)?;
    grouper::group(doc)
}

/// Like [`compile`], with `metadata` overlaid by the `preset` named
/// `preset_name`. `None`, or a name no preset has, compiles the song's own
/// `metadata`.
pub fn compile_with_preset(
    source: &str,
    filename: &str,
    instruments: &[InstrumentInfo],
    preset_name: Option<&str>,
) -> Result<Score, IrrecoverableError> {
    let doc = parser::parse(source, filename, instruments)?;
    grouper::group(match preset_name {
        Some(name) => doc.with_preset(name),
        None => doc,
    })
}

/// The `preset` named `preset_name`, for [`resolve_part_visibility`].
/// `None` when the source has no such preset (the menu then shows Default).
pub fn chosen_preset_from_source(
    source: &str,
    filename: &str,
    preset_name: &str,
) -> Result<Option<ChosenPreset>, IrrecoverableError> {
    let doc = parser::parse(source, filename, &[])?;
    Ok(doc.preset(preset_name).map(|preset| ChosenPreset {
        name: preset.name.clone(),
        visible_parts: preset.visible_abbreviations(),
        soloed_parts: preset.soloed_abbreviations(),
    }))
}

/// Supply every `FontFamily` role's raw font bytes for real glyph-advance
/// measurement during layout. Which role measures which text is decided by
/// `font_metrics`, not by the caller. Only meaningful on `wasm32`, where the
/// wasm binary doesn't embed the fonts at compile time (see `font_metrics`);
/// a no-op on the host CLI build, which embeds them via `include_bytes!`
/// instead.
pub fn set_layout_font_bytes(fonts: fonts::FontBytesByFamily) {
    font_metrics::set_layout_font_bytes(fonts);
}

/// Drop parts whose abbreviation is not in `enabled_tracks`, so the header's part-list
/// legend does not list parts hidden by a track filter.
///
/// `None` keeps every part.
pub(crate) fn filter_part_list(
    parts: Vec<PartInfo>,
    enabled_tracks: Option<&[String]>,
) -> Vec<PartInfo> {
    let Some(tracks) = enabled_tracks else {
        return parts;
    };
    parts
        .into_iter()
        .filter(|part| tracks.contains(&part.abbreviation))
        .collect()
}

fn build_header(score: &Score, parts: &[PartInfo]) -> grid_layout::types::Header {
    let part_list = parts
        .iter()
        .filter(|part| part.abbreviation != part.display_name)
        .map(|part| grid_layout::types::PartListEntry {
            abbreviation: part.abbreviation.clone(),
            display_name: part.display_name.clone(),
        })
        .collect();
    let sequence = score.sequence.as_ref().map(|spans| {
        spans
            .iter()
            .map(|span| grid_layout::types::SequenceEntryInfo {
                label: span.label.clone(),
                part_filter: span.part_filter_display.as_ref().map(|display| {
                    grid_layout::types::SequenceEntryPartFilter {
                        kind: display.kind,
                        parts: display.parts.clone(),
                    }
                }),
            })
            .collect()
    });
    grid_layout::types::Header {
        title: score.metadata.title.clone(),
        subtitle: score.metadata.subtitle.clone(),
        author: score.metadata.author.clone(),
        part_list,
        parts_list_columns: score.metadata.parts_list_columns,
        sequence,
        title_font_size: score.metadata.title_style.font_size as f32,
        subtitle_font_size: score.metadata.subtitle_style.font_size as f32,
        author_font_size: score.metadata.author_style.font_size as f32,
        sequence_font_size: score.metadata.sequence.font_size as f32,
        part_legend_font_size: score.metadata.part_legend.font_size as f32,
        title_bold: score.metadata.title_style.bold,
        title_italic: score.metadata.title_style.italic,
        title_underline: score.metadata.title_style.underline,
        title_font_family: score.metadata.title_style.font_family,
        subtitle_bold: score.metadata.subtitle_style.bold,
        subtitle_italic: score.metadata.subtitle_style.italic,
        subtitle_underline: score.metadata.subtitle_style.underline,
        subtitle_font_family: score.metadata.subtitle_style.font_family,
        author_bold: score.metadata.author_style.bold,
        author_italic: score.metadata.author_style.italic,
        author_underline: score.metadata.author_style.underline,
        author_font_family: score.metadata.author_style.font_family,
        part_legend_bold: score.metadata.part_legend.bold,
        part_legend_italic: score.metadata.part_legend.italic,
        part_legend_underline: score.metadata.part_legend.underline,
        part_legend_font_family: score.metadata.part_legend.font_family,
        part_legend_vertical_padding_pt: score.metadata.part_legend.vertical_padding_pt as f32,
        sequence_vertical_padding_pt: score.metadata.sequence.vertical_padding_pt as f32,
        title_vertical_padding_pt: score.metadata.title_style.vertical_padding_pt as f32,
        subtitle_vertical_padding_pt: score.metadata.subtitle_style.vertical_padding_pt as f32,
        author_vertical_padding_pt: score.metadata.author_style.vertical_padding_pt as f32,
        running_title_font_size: score.metadata.running_title.font_size as f32,
        running_title_bold: score.metadata.running_title.bold,
        running_title_italic: score.metadata.running_title.italic,
        running_title_underline: score.metadata.running_title.underline,
        running_title_font_family: score.metadata.running_title.font_family,
        running_title_vertical_padding_pt: score.metadata.running_title.vertical_padding_pt as f32,
    }
}

/// Typed SVG document trees plus any diagnostics found during layout (e.g.
/// `WarningKind::MeasureOverflow`) — distinct from the per-measure
/// diagnostics `collect_measure_diagnostics` gathers from the pre-layout
/// grouped `Score`, which callers merge in on top.
struct SvgDocsResult {
    documents: Vec<renderer::new_types::SvgDocument>,
    diagnostics: Vec<Diagnostic>,
}

fn render_svg_docs_with_parts(
    score: &Score,
    parts: &[PartInfo],
    note_labels: NoteLabels,
) -> Result<SvgDocsResult, IrrecoverableError> {
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = build_header(score, parts);
    let compile_result = compiler::label_note_heads(compiler::compile(score), score, note_labels);
    let compile_result = consolidator::consolidate(compile_result);
    let grid_layout::LayoutOutput {
        pages: grid_pages,
        diagnostics,
    } = grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None);
    let abs = coordinate_resolver::resolve(
        &grid_pages,
        config.note_number_width as f32,
        config.page_frame(),
        coordinate_resolver::ResolveFontSizes {
            lyric: config.lyric_font_sizes(),
            notes: config.notes_font_size(),
            chords: config.chords_font_size(),
            labels: coordinate_resolver::LabelFontSizes {
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
    Ok(SvgDocsResult {
        documents: renderer::new_renderer::render_new(&abs, &config),
        diagnostics,
    })
}

/// Serialized SVG page strings plus any diagnostics found during layout —
/// the serialized-string mirror of [`SvgDocsResult`].
pub(crate) struct SvgsResult {
    pub(crate) svgs: Vec<String>,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

pub(crate) fn render_svgs_with_parts(
    score: &Score,
    parts: &[PartInfo],
    source: Option<&str>,
    note_labels: NoteLabels,
) -> Result<SvgsResult, IrrecoverableError> {
    let result = render_svg_docs_with_parts(score, parts, note_labels)?;
    Ok(SvgsResult {
        svgs: serializer::serialize(&result.documents, source),
        diagnostics: result.diagnostics,
    })
}

/// Layout and render a [`Score`] into one SVG string per page.
pub fn render_svgs(score: &Score) -> Result<Vec<String>, IrrecoverableError> {
    Ok(render_svgs_with_parts(score, &[], None, NoteLabels::Degrees)?.svgs)
}

/// Parse, group, and render a `.poentaro` source string into SVG page strings.
pub fn render_svgs_from_source(
    source: &str,
    filename: &str,
    instruments: &[InstrumentInfo],
) -> Result<RenderOutput, IrrecoverableError> {
    render_svgs_from_source_filtered(source, filename, None, instruments)
}

/// Parse, group, optionally filter tracks, and render SVG page strings.
///
/// When `enabled_tracks` is `None`, all parts are rendered.
/// When `Some(tracks)` is empty, no parts are rendered.
pub fn render_svgs_from_source_filtered(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> Result<RenderOutput, IrrecoverableError> {
    render_svgs_from_source_with_visibility(
        source,
        filename,
        &ResolvedPartVisibility {
            rendered_tracks: enabled_tracks.map(<[String]>::to_vec),
            ..Default::default()
        },
        instruments,
        NoteLabels::Degrees,
    )
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tuplet_tests.rs"]
mod tuplet_tests;
