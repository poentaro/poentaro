use poentaro::write_mp3_for_measure_range_from_source;
use poentaro::write_split_mp3s_from_source;
use poentaro::zip_split_entries;
use poentaro::{singing_plan_for_measure_range_from_source, split_singing_plans_from_source};
use poentaro::{singing_plan_from_source_filtered, write_mp3_from_source_filtered, AudioExport};
use poentaro::{wav, MeasureRangeAudioOptions, MeasureRangeSelection};

use super::diagnostic_from_error;
use crate::opened_document::open_checked;
use crate::types::{GenerateMp3Response, GenerateSplitMp3sResponse};

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn generate_mp3_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
    soundfont: Vec<u8>,
    vocals: Vec<f32>,
) -> GenerateMp3Response {
    if let Err(diagnostics) = open_checked(source, &[]) {
        return GenerateMp3Response::Err { diagnostics };
    }

    match write_mp3_from_source_filtered(&AudioExport {
        source,
        filename: "input.poentaro",
        enabled_tracks,
        sf2_bytes: &soundfont,
        instruments: &[],
        vocals: &vocals,
    }) {
        Ok(mp3) => GenerateMp3Response::Ok { mp3 },
        Err(e) => GenerateMp3Response::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}

/// What the MP3 export of the same arguments sings, for the singer module;
/// `None` when nothing sings (or the source doesn't compile, which the
/// export itself then reports).
pub(crate) fn singing_plan_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
) -> Option<Vec<u8>> {
    singing_plan_from_source_filtered(source, "input.poentaro", enabled_tracks, &[])
        .ok()
        .filter(|plan| !plan.parts.is_empty())
        .map(|plan| plan.to_bytes())
}

/// What the measure-range audio of the same arguments sings, like
/// [`singing_plan_response`]; selected lyrics alone sing on their
/// `visible_tracks` singers.
pub(crate) fn singing_plan_for_measure_range_response(
    source: &str,
    selection: &MeasureRangeSelection,
    enabled_tracks: Option<&[String]>,
    visible_tracks: Option<&[String]>,
) -> Option<Vec<u8>> {
    singing_plan_for_measure_range_from_source(
        source,
        "input.poentaro",
        selection,
        enabled_tracks,
        visible_tracks,
        &[],
    )
    .ok()
    .filter(|plan| !plan.parts.is_empty())
    .map(|plan| plan.to_bytes())
}

/// What each track of a split audio export sings; empty when nothing sings
/// (or the source doesn't compile, which the export itself then reports).
pub(crate) fn split_singing_plans_response(
    source: &str,
    preset_name: Option<&str>,
) -> Vec<poentaro::TrackSingingPlan> {
    split_singing_plans_from_source(
        source,
        "input.poentaro",
        poentaro::SplitSelection {
            tracks: &[],
            preset_name,
        },
    )
    .unwrap_or_default()
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub(crate) fn generate_mp3_for_measure_range_response(
    source: &str,
    start_index: usize,
    end_index: usize,
    extend_to_last_occurrence: bool,
    respect_sequence: bool,
    sequence_entry_range: Option<std::ops::RangeInclusive<usize>>,
    enabled_tracks: Option<&[String]>,
    trim: Option<wav::TrimWindow>,
    soundfont: Vec<u8>,
    vocals: Vec<f32>,
) -> GenerateMp3Response {
    if let Err(diagnostics) = open_checked(source, &[]) {
        return GenerateMp3Response::Err { diagnostics };
    }

    match write_mp3_for_measure_range_from_source(
        source,
        "input.poentaro",
        &MeasureRangeSelection {
            range: start_index..=end_index,
            extend_to_last_occurrence,
            respect_sequence,
            sequence_entry_range,
        },
        &MeasureRangeAudioOptions {
            enabled_tracks,
            metronome: false,
            trim,
            vocals: &vocals,
        },
        &soundfont,
        &[],
    ) {
        Ok(mp3) => GenerateMp3Response::Ok { mp3 },
        Err(e) => GenerateMp3Response::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn generate_split_mp3s_response(
    source: &str,
    base_name: &str,
    preset_name: Option<&str>,
    soundfont: Vec<u8>,
    vocals: &[poentaro::TrackVocals],
) -> GenerateSplitMp3sResponse {
    if let Err(diagnostics) = open_checked(source, &[]) {
        return GenerateSplitMp3sResponse::Err { diagnostics };
    }

    match write_split_mp3s_from_source(
        source,
        "input.poentaro",
        base_name,
        poentaro::SplitSelection {
            tracks: &[],
            preset_name,
        },
        &soundfont,
        vocals,
    ) {
        Ok(entries) => match zip_split_entries(&entries) {
            Ok(zip) => GenerateSplitMp3sResponse::Ok { zip },
            Err(e) => GenerateSplitMp3sResponse::Err {
                diagnostics: vec![diagnostic_from_error(&e)],
            },
        },
        Err(e) => GenerateSplitMp3sResponse::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}
