use poentaro::{write_midi_from_source_filtered, write_split_midis_from_source, zip_split_entries};

use super::diagnostic_from_error;
use crate::opened_document::open_checked;
use crate::types::{GenerateMidiResponse, GenerateSplitMidisResponse};

pub(crate) fn generate_midi_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
) -> GenerateMidiResponse {
    if let Err(diagnostics) = open_checked(source, &[]) {
        return GenerateMidiResponse::Err { diagnostics };
    }

    match write_midi_from_source_filtered(source, "input.poentaro", enabled_tracks, &[]) {
        Ok(midi) => GenerateMidiResponse::Ok { midi },
        Err(e) => GenerateMidiResponse::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}

pub(crate) fn generate_split_midis_response(
    source: &str,
    base_name: &str,
    preset_name: Option<&str>,
) -> GenerateSplitMidisResponse {
    if let Err(diagnostics) = open_checked(source, &[]) {
        return GenerateSplitMidisResponse::Err { diagnostics };
    }

    match write_split_midis_from_source(
        source,
        "input.poentaro",
        base_name,
        poentaro::SplitSelection {
            tracks: &[],
            preset_name,
        },
    ) {
        Ok(entries) => match zip_split_entries(&entries) {
            Ok(zip) => GenerateSplitMidisResponse::Ok { zip },
            Err(e) => GenerateSplitMidisResponse::Err {
                diagnostics: vec![diagnostic_from_error(&e)],
            },
        },
        Err(e) => GenerateSplitMidisResponse::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}
