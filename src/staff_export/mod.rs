//! Exports of the staff-notation view. Rust stops at MEI (`mei_export`);
//! Verovio, in the web worker, engraves it into SVG pages, which come back
//! here to be written as PDFs the same way the numbered pages are.
use crate::error::IrrecoverableError;
use crate::mei_export::ClefChoice;
use crate::parser::parts_parser::InstrumentInfo;
use crate::split_track::{
    part_display_name_map, split_pdf_filename, split_track_label, split_track_names,
    track_with_its_lyric_parts, SplitSelection,
};

/// One part's MEI for a split (one PDF per part) staff export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitMeiEntry {
    pub track_name: String,
    /// The PDF this part becomes, named like the numbered split export's.
    pub filename: String,
    pub mei: String,
}

/// The staff counterpart of `write_split_pdfs_from_source`: one MEI per
/// sounding part, each keeping the lyric parts that sing along to it.
pub fn render_split_meis_from_source(
    source: &str,
    filename: &str,
    base_name: &str,
    selection: SplitSelection,
    instruments: &[InstrumentInfo],
    clef_choice: ClefChoice,
) -> Result<Vec<SplitMeiEntry>, IrrecoverableError> {
    let score = crate::compile_with_preset(source, filename, instruments, selection.preset_name)?;
    let base_name = selection.file_base_name(base_name);
    let labels = part_display_name_map(source, filename)?;
    let all_parts = crate::list_parts_from_source(source, filename, instruments)?;
    Ok(split_track_names(source, filename, &score, selection)?
        .into_iter()
        .map(|track| {
            let enabled_tracks = track_with_its_lyric_parts(&score, &track);
            let mut part_score = score.clone();
            crate::apply_track_filter(&mut part_score, Some(&enabled_tracks));
            let display_names = crate::filter_part_list(all_parts.clone(), Some(&enabled_tracks))
                .into_iter()
                .map(|part| (part.abbreviation, part.display_name))
                .collect();
            SplitMeiEntry {
                filename: split_pdf_filename(&base_name, &split_track_label(&labels, &track)),
                mei: crate::mei_export::score_to_mei(&part_score, &display_names, clef_choice).mei,
                track_name: track,
            }
        })
        .collect())
}

/// Verovio's SVG pages for one part's PDF in a split staff export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffPdfPart {
    pub track_name: String,
    pub filename: String,
    pub pages: Vec<String>,
}

/// Verovio's `svgViewBox` pages carry only a `viewBox` in its own scaled
/// units, so they're given the numbered pages' physical A4 size (the
/// aspect ratio already matches: Verovio lays out on a 2100×2970 page).
#[cfg(feature = "pdf")]
fn with_a4_page_size(page: &str) -> String {
    let root_tag = page
        .find("<svg")
        .and_then(|start| page[start..].find('>').map(|end| &page[start..start + end]));
    if root_tag.is_some_and(|tag| tag.contains(" width=")) {
        return page.to_string();
    }
    page.replacen("<svg", r#"<svg width="210mm" height="297mm""#, 1)
}

/// Verovio puts a `<title class="labelAttr">` tooltip inside some `<text>`
/// (e.g. the score title); usvg lays it out as text, which pushes centred
/// text off the page, so it's dropped (a PDF has no tooltips anyway).
#[cfg(feature = "pdf")]
fn without_text_tooltips(page: &str) -> String {
    const OPEN: &str = r#"<title class="labelAttr">"#;
    const CLOSE: &str = "</title>";
    let mut kept = String::with_capacity(page.len());
    let mut rest = page;
    while let Some(start) = rest.find(OPEN) {
        let Some(length) = rest[start..].find(CLOSE) else {
            break;
        };
        kept.push_str(&rest[..start]);
        rest = &rest[start + length + CLOSE.len()..];
    }
    kept.push_str(rest);
    kept
}

/// Writes Verovio's staff pages as a PDF, embedding `source` (like the
/// numbered PDF) so the file can be imported back.
#[cfg(feature = "pdf")]
pub fn write_staff_pdf(
    pages: &[String],
    fonts: &crate::fonts::FontBytesByFamily,
    source: Option<&str>,
) -> Result<Vec<u8>, IrrecoverableError> {
    let printable_pages: Vec<String> = pages
        .iter()
        .map(|page| with_a4_page_size(&without_text_tooltips(page)))
        .collect();
    crate::pdf::write_pdf(&printable_pages, fonts, source)
}

/// One PDF per part, zipped like the numbered split export.
#[cfg(feature = "pdf")]
pub fn zip_staff_pdfs(
    parts: &[StaffPdfPart],
    fonts: &crate::fonts::FontBytesByFamily,
) -> Result<Vec<u8>, IrrecoverableError> {
    let entries = parts
        .iter()
        .map(|part| {
            Ok(crate::split_track::SplitPdfEntry {
                track_name: part.track_name.clone(),
                filename: part.filename.clone(),
                pdf: write_staff_pdf(&part.pages, fonts, None)?,
            })
        })
        .collect::<Result<Vec<_>, IrrecoverableError>>()?;
    crate::split_track::zip_split_pdfs(&entries)
}

#[cfg(test)]
mod tests;
