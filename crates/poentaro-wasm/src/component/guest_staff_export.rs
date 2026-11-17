//! Real bodies for the staff-notation export `Guest` methods
//! `guest_impl.rs` dispatches to: per-part MEI out, Verovio's SVG pages back
//! in as PDFs. Kept at the exact by-value parameter types the `Guest` trait
//! requires (see `mod.rs`'s doc comment), so `needless_pass_by_value` is
//! relaxed here.
#![allow(clippy::needless_pass_by_value)]

use super::*;
use poentaro::staff_export::StaffPdfPart as CoreStaffPdfPart;

fn single_error(error: &poentaro::error::IrrecoverableError) -> DiagnosticsError {
    diagnostics_error_to_wit(&[crate::diagnostics::diagnostic_from_error(error)])
}

pub(super) fn render_split_meis(
    source: String,
    base_name: String,
    preset_name: Option<String>,
    raw_instruments: Vec<InstrumentInfo>,
    clef: StaffClef,
) -> RenderSplitMeisResponse {
    let instruments: Vec<poentaro::parser::parts_parser::InstrumentInfo> = raw_instruments
        .into_iter()
        .map(instrument_info_from_wit)
        .collect();
    match poentaro::staff_export::render_split_meis_from_source(
        &source,
        "input.poentaro",
        &base_name,
        poentaro::SplitSelection {
            tracks: &[],
            preset_name: preset_name.as_deref(),
        },
        &instruments,
        clef_choice_from_wit(clef),
    ) {
        Ok(entries) => RenderSplitMeisResponse::Ok(
            entries
                .into_iter()
                .map(|entry| SplitMei {
                    track_name: entry.track_name,
                    file_name: entry.filename,
                    mei: entry.mei,
                })
                .collect(),
        ),
        Err(error) => RenderSplitMeisResponse::Err(single_error(&error)),
    }
}

pub(super) fn generate_staff_pdf(
    pages: Vec<String>,
    source: String,
    fonts: FontBytesByFamily,
) -> GeneratePdfResponse {
    match poentaro::staff_export::write_staff_pdf(
        &pages,
        &font_bytes_by_family_from_wit(fonts),
        Some(&source),
    ) {
        Ok(pdf) => GeneratePdfResponse::Ok(GeneratePdfSuccess { pdf }),
        Err(error) => GeneratePdfResponse::Err(single_error(&error)),
    }
}

pub(super) fn generate_split_staff_pdfs(
    parts: Vec<StaffPdfPart>,
    fonts: FontBytesByFamily,
) -> GenerateSplitPdfsResponse {
    let parts: Vec<CoreStaffPdfPart> = parts
        .into_iter()
        .map(|part| CoreStaffPdfPart {
            track_name: part.track_name,
            filename: part.file_name,
            pages: part.pages,
        })
        .collect();
    match poentaro::staff_export::zip_staff_pdfs(&parts, &font_bytes_by_family_from_wit(fonts)) {
        Ok(zip) => GenerateSplitPdfsResponse::Ok(GenerateSplitPdfsSuccess { zip }),
        Err(error) => GenerateSplitPdfsResponse::Err(single_error(&error)),
    }
}
