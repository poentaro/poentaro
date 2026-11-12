use poentaro::note_timings_for_range_from_source;
use poentaro::note_timings_from_source;
use poentaro::wav;
use poentaro::write_split_wavs_from_source;
use poentaro::zip_split_entries;
use poentaro::{
    write_wav_for_measure_range_from_source, MeasureRangeAudioOptions, MeasureRangeSelection,
};
use poentaro::{write_wav_from_source_filtered, AudioExport};

use super::diagnostic_from_error;
use crate::opened_document::open_checked;
use crate::types::{
    GenerateSplitWavsResponse, GenerateWavResponse, NoteTimingOut, NoteTimingsResponse,
};

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn generate_wav_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
    metronome: bool,
    soundfont: Vec<u8>,
    vocals: Vec<f32>,
) -> GenerateWavResponse {
    if let Err(diagnostics) = open_checked(source, &[]) {
        return GenerateWavResponse::Err { diagnostics };
    }

    let export = AudioExport {
        source,
        filename: "input.poentaro",
        enabled_tracks,
        sf2_bytes: &soundfont,
        instruments: &[],
        vocals: &vocals,
    };
    match write_wav_from_source_filtered(&export, metronome) {
        Ok(wav) => GenerateWavResponse::Ok { wav },
        Err(e) => GenerateWavResponse::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub(crate) fn generate_wav_for_measure_range_response(
    source: &str,
    start_index: usize,
    end_index: usize,
    extend_to_last_occurrence: bool,
    respect_sequence: bool,
    sequence_entry_range: Option<std::ops::RangeInclusive<usize>>,
    enabled_tracks: Option<&[String]>,
    metronome: bool,
    trim: Option<wav::TrimWindow>,
    soundfont: Vec<u8>,
    vocals: Vec<f32>,
) -> GenerateWavResponse {
    if let Err(diagnostics) = open_checked(source, &[]) {
        return GenerateWavResponse::Err { diagnostics };
    }

    match write_wav_for_measure_range_from_source(
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
            metronome,
            trim,
            vocals: &vocals,
        },
        &soundfont,
        &[],
    ) {
        Ok(wav) => GenerateWavResponse::Ok { wav },
        Err(e) => GenerateWavResponse::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}

pub(crate) fn list_note_timings_response(
    source: &str,
    visible_tracks: Option<&[String]>,
    enabled_tracks: Option<&[String]>,
) -> NoteTimingsResponse {
    if let Err(diagnostics) = open_checked(source, &[]) {
        return NoteTimingsResponse::Err { diagnostics };
    }

    match note_timings_from_source(
        source,
        "input.poentaro",
        visible_tracks,
        enabled_tracks,
        &[],
    ) {
        Ok(timings) => NoteTimingsResponse::Ok {
            timings: timings
                .into_iter()
                .map(|t| NoteTimingOut {
                    source_part_index: t.source_part_index,
                    note_id: t.note_id,
                    start_s: t.start_s,
                    end_s: t.end_s,
                })
                .collect(),
        },
        Err(e) => NoteTimingsResponse::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn list_note_timings_for_range_response(
    source: &str,
    start_index: usize,
    end_index: usize,
    extend_to_last_occurrence: bool,
    respect_sequence: bool,
    sequence_entry_range: Option<std::ops::RangeInclusive<usize>>,
    visible_tracks: Option<&[String]>,
    enabled_tracks: Option<&[String]>,
) -> NoteTimingsResponse {
    if let Err(diagnostics) = open_checked(source, &[]) {
        return NoteTimingsResponse::Err { diagnostics };
    }

    match note_timings_for_range_from_source(
        source,
        "input.poentaro",
        &MeasureRangeSelection {
            range: start_index..=end_index,
            extend_to_last_occurrence,
            respect_sequence,
            sequence_entry_range,
        },
        visible_tracks,
        enabled_tracks,
        &[],
    ) {
        Ok(timings) => NoteTimingsResponse::Ok {
            timings: timings
                .into_iter()
                .map(|t| NoteTimingOut {
                    source_part_index: t.source_part_index,
                    note_id: t.note_id,
                    start_s: t.start_s,
                    end_s: t.end_s,
                })
                .collect(),
        },
        Err(e) => NoteTimingsResponse::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn generate_instrument_preview_wav_response(
    program_number: u8,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    match wav::write_preview_wav(program_number, &soundfont) {
        Ok(wav) => GenerateWavResponse::Ok { wav },
        Err(e) => GenerateWavResponse::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn generate_percussion_preview_wav_response(
    key: u8,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    match wav::write_percussion_preview_wav(key, &soundfont) {
        Ok(wav) => GenerateWavResponse::Ok { wav },
        Err(e) => GenerateWavResponse::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn generate_split_wavs_response(
    source: &str,
    base_name: &str,
    preset_name: Option<&str>,
    soundfont: Vec<u8>,
    vocals: &[poentaro::TrackVocals],
) -> GenerateSplitWavsResponse {
    if let Err(diagnostics) = open_checked(source, &[]) {
        return GenerateSplitWavsResponse::Err { diagnostics };
    }

    match write_split_wavs_from_source(
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
            Ok(zip) => GenerateSplitWavsResponse::Ok { zip },
            Err(e) => GenerateSplitWavsResponse::Err {
                diagnostics: vec![diagnostic_from_error(&e)],
            },
        },
        Err(e) => GenerateSplitWavsResponse::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}
