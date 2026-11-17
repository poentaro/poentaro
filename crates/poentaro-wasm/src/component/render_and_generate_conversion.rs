use super::*;

pub(super) fn render_response_to_wit(response: crate::types::RenderResponse) -> RenderResponse {
    match response {
        crate::types::RenderResponse::Ok {
            documents,
            diagnostics,
            diagnostic_view_zones,
        } => RenderResponse::Ok(RenderSuccess {
            documents: documents.iter().map(svg_document_to_wit).collect(),
            diagnostics: diagnostics.iter().map(diagnostic_to_wit).collect(),
            diagnostic_view_zones: diagnostic_view_zones
                .iter()
                .map(diagnostic_view_zone_to_wit)
                .collect(),
        }),
        crate::types::RenderResponse::Err {
            diagnostics,
            diagnostic_view_zones,
        } => RenderResponse::Err(RenderFailure {
            diagnostics: diagnostics.iter().map(diagnostic_to_wit).collect(),
            diagnostic_view_zones: diagnostic_view_zones
                .iter()
                .map(diagnostic_view_zone_to_wit)
                .collect(),
        }),
    }
}

pub(super) fn generate_wav_response_to_wit(
    response: crate::types::GenerateWavResponse,
) -> GenerateWavResponse {
    match response {
        crate::types::GenerateWavResponse::Ok { wav } => {
            GenerateWavResponse::Ok(GenerateWavSuccess { wav })
        }
        crate::types::GenerateWavResponse::Err { diagnostics } => {
            GenerateWavResponse::Err(diagnostics_error_to_wit(&diagnostics))
        }
    }
}

pub(super) fn generate_split_wavs_response_to_wit(
    response: crate::types::GenerateSplitWavsResponse,
) -> GenerateSplitWavsResponse {
    match response {
        crate::types::GenerateSplitWavsResponse::Ok { zip } => {
            GenerateSplitWavsResponse::Ok(GenerateSplitWavsSuccess { zip })
        }
        crate::types::GenerateSplitWavsResponse::Err { diagnostics } => {
            GenerateSplitWavsResponse::Err(diagnostics_error_to_wit(&diagnostics))
        }
    }
}

pub(super) fn generate_mp3_response_to_wit(
    response: crate::types::GenerateMp3Response,
) -> GenerateMp3Response {
    match response {
        crate::types::GenerateMp3Response::Ok { mp3 } => {
            GenerateMp3Response::Ok(GenerateMp3Success { mp3 })
        }
        crate::types::GenerateMp3Response::Err { diagnostics } => {
            GenerateMp3Response::Err(diagnostics_error_to_wit(&diagnostics))
        }
    }
}

pub(super) fn generate_split_mp3s_response_to_wit(
    response: crate::types::GenerateSplitMp3sResponse,
) -> GenerateSplitMp3sResponse {
    match response {
        crate::types::GenerateSplitMp3sResponse::Ok { zip } => {
            GenerateSplitMp3sResponse::Ok(GenerateSplitMp3sSuccess { zip })
        }
        crate::types::GenerateSplitMp3sResponse::Err { diagnostics } => {
            GenerateSplitMp3sResponse::Err(diagnostics_error_to_wit(&diagnostics))
        }
    }
}

pub(super) fn note_timing_to_wit(timing: &crate::types::NoteTimingOut) -> NoteTiming {
    NoteTiming {
        source_part_index: timing.source_part_index as u32,
        note_id: timing.note_id as u32,
        start_s: timing.start_s,
        end_s: timing.end_s,
    }
}

pub(super) fn note_timing_from_wit(timing: NoteTiming) -> crate::types::NoteTimingOut {
    crate::types::NoteTimingOut {
        source_part_index: timing.source_part_index as usize,
        note_id: timing.note_id as usize,
        start_s: timing.start_s,
        end_s: timing.end_s,
    }
}

pub(super) fn note_timings_response_to_wit(
    response: crate::types::NoteTimingsResponse,
) -> NoteTimingsResponse {
    match response {
        crate::types::NoteTimingsResponse::Ok { timings } => {
            NoteTimingsResponse::Ok(NoteTimingsSuccess {
                timings: timings.iter().map(note_timing_to_wit).collect(),
            })
        }
        crate::types::NoteTimingsResponse::Err { diagnostics } => {
            NoteTimingsResponse::Err(diagnostics_error_to_wit(&diagnostics))
        }
    }
}

pub(super) fn generate_pdf_response_to_wit(
    response: crate::types::GeneratePdfResponse,
) -> GeneratePdfResponse {
    match response {
        crate::types::GeneratePdfResponse::Ok { pdf } => {
            GeneratePdfResponse::Ok(GeneratePdfSuccess { pdf })
        }
        crate::types::GeneratePdfResponse::Err { diagnostics } => {
            GeneratePdfResponse::Err(diagnostics_error_to_wit(&diagnostics))
        }
    }
}

pub(super) fn generate_split_pdfs_response_to_wit(
    response: crate::types::GenerateSplitPdfsResponse,
) -> GenerateSplitPdfsResponse {
    match response {
        crate::types::GenerateSplitPdfsResponse::Ok { zip } => {
            GenerateSplitPdfsResponse::Ok(GenerateSplitPdfsSuccess { zip })
        }
        crate::types::GenerateSplitPdfsResponse::Err { diagnostics } => {
            GenerateSplitPdfsResponse::Err(diagnostics_error_to_wit(&diagnostics))
        }
    }
}

pub(super) fn generate_midi_response_to_wit(
    response: crate::types::GenerateMidiResponse,
) -> GenerateMidiResponse {
    match response {
        crate::types::GenerateMidiResponse::Ok { midi } => {
            GenerateMidiResponse::Ok(GenerateMidiSuccess { midi })
        }
        crate::types::GenerateMidiResponse::Err { diagnostics } => {
            GenerateMidiResponse::Err(diagnostics_error_to_wit(&diagnostics))
        }
    }
}

pub(super) fn generate_split_midis_response_to_wit(
    response: crate::types::GenerateSplitMidisResponse,
) -> GenerateSplitMidisResponse {
    match response {
        crate::types::GenerateSplitMidisResponse::Ok { zip } => {
            GenerateSplitMidisResponse::Ok(GenerateSplitMidisSuccess { zip })
        }
        crate::types::GenerateSplitMidisResponse::Err { diagnostics } => {
            GenerateSplitMidisResponse::Err(diagnostics_error_to_wit(&diagnostics))
        }
    }
}

/// The WIT `font-bytes-by-family` record → the core crate's same-shaped
/// role-keyed struct.
pub(super) fn font_bytes_by_family_from_wit(
    fonts: FontBytesByFamily,
) -> poentaro::fonts::FontBytesByFamily {
    let FontBytesByFamily {
        serif,
        sans_serif,
        monospace,
    } = fonts;
    poentaro::fonts::FontBytesByFamily {
        serif,
        sans_serif,
        monospace,
    }
}

pub(super) fn note_labels_from_wit(labels: NoteLabels) -> poentaro::NoteLabels {
    match labels {
        NoteLabels::Degrees => poentaro::NoteLabels::Degrees,
        NoteLabels::LetterNames => poentaro::NoteLabels::LetterNames,
        NoteLabels::HarmonicaTabs => poentaro::NoteLabels::HarmonicaTabs,
        NoteLabels::GuitarChords => poentaro::NoteLabels::GuitarChords,
    }
}

pub(super) fn page_layout_from_wit(layout: PageLayout) -> poentaro::grid_layout::PageLayout {
    match layout {
        PageLayout::A4 => poentaro::grid_layout::PageLayout::A4,
        PageLayout::Continuous(page) => {
            poentaro::grid_layout::PageLayout::Continuous(poentaro::grid_layout::ContinuousPage {
                // One point per CSS pixel, so max view draws the score at
                // three quarters of its printed size: a few more measures
                // per line than true size, and still larger than an A4
                // page shrunk to a phone.
                width_pt: page.width_px,
            })
        }
    }
}

pub(super) fn clef_choice_from_wit(clef: StaffClef) -> poentaro::mei_export::ClefChoice {
    match clef {
        StaffClef::Violin => poentaro::mei_export::ClefChoice::Violin,
        StaffClef::Cello => poentaro::mei_export::ClefChoice::Cello,
    }
}
