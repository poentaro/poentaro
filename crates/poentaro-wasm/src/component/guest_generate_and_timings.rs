//! Real bodies for the `Guest` methods `guest_impl.rs` dispatches to. Kept
//! at the exact by-value parameter types the `Guest` trait requires (see
//! `mod.rs`'s doc comment), so `needless_pass_by_value` is relaxed here.
#![allow(clippy::needless_pass_by_value)]

use super::*;

// Phase 3, group 5 of PLAN-wit-bindgen-migration.md: same underlying
// logic as `wasm_boundary`'s `lib_wav::generate_wav`/
// `lib_wav::generate_split_wavs`, `lib_mp3::generate_mp3`/
// `lib_mp3::generate_split_mp3s`, `lib_pdf::generate_pdf`/
// `lib_pdf::generate_split_pdfs`, and `lib_midi::generate_midi`/
// `lib_midi::generate_split_midis` (`crate::responses::generate_wav_response`/
// `generate_split_wavs_response`/`generate_mp3_response`/
// `generate_split_mp3s_response`/`generate_pdf_response`/
// `generate_split_pdfs_response`/`generate_midi_response`/
// `generate_split_midis_response`), just converting to/from the
// WIT-generated shapes instead of `JsValue`/`serde_wasm_bindgen`. Both
// the old `#[wasm_bindgen] fn`s and these methods coexist until Phase 6.
// This group required extending Phase 1's per-site `cfg_attr` type/module
// conversion (previously only done for `types.rs`/`metadata_types.rs`/
// `svg_types.rs`/etc.) to `types_export.rs` and
// `responses_wav.rs`/`responses_mp3.rs`/`responses_pdf.rs`/
// `responses_midi.rs` — those files, and the `mod` declarations
// referencing them in `lib.rs`/`responses.rs`, were previously gated on
// `wasm-bindgen-boundary` *as a whole module*, not per-site, since
// nothing outside the boundary needed to call into them until now. See
// PLAN-wit-bindgen-migration.md's Status section for detail.
//
// Only the 8 functions the plan's Phase 3 ordering explicitly names
// (`generate_midi`/`generate_wav`/`generate_pdf`/`generate_mp3` and
// their `split` variants) are ported here. `generate_wav_for_measure_range`,
// `generate_mp3_for_measure_range`, `list_note_timings`,
// `list_note_timings_for_range`, `generate_instrument_preview_wav`, and
// `generate_percussion_preview_wav` live in the same `lib_wav.rs`/
// `lib_mp3.rs` files but are not named anywhere in the plan's ordering —
// deliberately left unported here, not silently folded in or dropped;
// ported separately in Phase 3, group 6 below, per the plan's Status
// section.

pub(super) fn generate_wav(
    source: String,
    enabled_tracks: Option<Vec<String>>,
    metronome: bool,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    generate_wav_response_to_wit(crate::responses::generate_wav_response(
        &source,
        enabled_tracks.as_deref(),
        metronome,
        soundfont,
    ))
}

pub(super) fn generate_split_wavs(
    source: String,
    base_name: String,
    preset_name: Option<String>,
    soundfont: Vec<u8>,
) -> GenerateSplitWavsResponse {
    generate_split_wavs_response_to_wit(crate::responses::generate_split_wavs_response(
        &source,
        &base_name,
        preset_name.as_deref(),
        soundfont,
    ))
}

pub(super) fn generate_mp3(
    source: String,
    enabled_tracks: Option<Vec<String>>,
    soundfont: Vec<u8>,
    vocals: Vec<f32>,
) -> GenerateMp3Response {
    generate_mp3_response_to_wit(crate::responses::generate_mp3_response(
        &source,
        enabled_tracks.as_deref(),
        soundfont,
        vocals,
    ))
}

pub(super) fn singing_plan(source: String, enabled_tracks: Option<Vec<String>>) -> Option<Vec<u8>> {
    crate::responses::singing_plan_response(&source, enabled_tracks.as_deref())
}

pub(super) fn generate_split_mp3s(
    source: String,
    base_name: String,
    preset_name: Option<String>,
    soundfont: Vec<u8>,
) -> GenerateSplitMp3sResponse {
    generate_split_mp3s_response_to_wit(crate::responses::generate_split_mp3s_response(
        &source,
        &base_name,
        preset_name.as_deref(),
        soundfont,
    ))
}

pub(super) fn generate_pdf(
    source: String,
    visibility: ResolvedPartVisibility,
    fonts: FontBytesByFamily,
    note_labels: NoteLabels,
) -> GeneratePdfResponse {
    generate_pdf_response_to_wit(crate::responses::generate_pdf_response(
        &source,
        &resolved_part_visibility_from_wit(visibility),
        &font_bytes_by_family_from_wit(fonts),
        note_labels_from_wit(note_labels),
    ))
}

pub(super) fn generate_split_pdfs(
    source: String,
    base_name: String,
    preset_name: Option<String>,
    fonts: FontBytesByFamily,
    note_labels: NoteLabels,
) -> GenerateSplitPdfsResponse {
    generate_split_pdfs_response_to_wit(crate::responses::generate_split_pdfs_response(
        &source,
        &base_name,
        preset_name.as_deref(),
        &font_bytes_by_family_from_wit(fonts),
        note_labels_from_wit(note_labels),
    ))
}

pub(super) fn generate_midi(
    source: String,
    enabled_tracks: Option<Vec<String>>,
) -> GenerateMidiResponse {
    generate_midi_response_to_wit(crate::responses::generate_midi_response(
        &source,
        enabled_tracks.as_deref(),
    ))
}

pub(super) fn generate_split_midis(
    source: String,
    base_name: String,
    preset_name: Option<String>,
) -> GenerateSplitMidisResponse {
    generate_split_midis_response_to_wit(crate::responses::generate_split_midis_response(
        &source,
        &base_name,
        preset_name.as_deref(),
    ))
}

// Phase 3, group 6 of PLAN-wit-bindgen-migration.md: the six functions
// group 5's Status entry flagged as living in the same `lib_wav.rs`/
// `lib_mp3.rs` files but not named in the plan's explicit Phase 3
// ordering list. Same underlying logic as `wasm_boundary`'s
// `lib_wav::generate_wav_for_measure_range`/`lib_wav::list_note_timings`/
// `lib_wav::list_note_timings_for_range`/
// `lib_wav::generate_instrument_preview_wav`/
// `lib_wav::generate_percussion_preview_wav` and
// `lib_mp3::generate_mp3_for_measure_range`
// (`crate::responses::generate_wav_for_measure_range_response`/
// `list_note_timings_response`/`list_note_timings_for_range_response`/
// `generate_instrument_preview_wav_response`/
// `generate_percussion_preview_wav_response`/
// `generate_mp3_for_measure_range_response`), just converting to/from the
// WIT-generated shapes instead of `JsValue`/`serde_wasm_bindgen`. Both
// the old `#[wasm_bindgen] fn`s and these methods coexist until Phase 6.
// `crate::sequence_entry_range`/`crate::trim_window` (previously reachable
// only from the `wasm-bindgen` boundary) had to be ungated the same way
// group 5 ungated `types_export.rs`/`responses_*.rs` — see
// PLAN-wit-bindgen-migration.md's Status section. This closes the scope
// gap group 5 flagged: Phase 3's function porting is now complete.

pub(super) fn generate_wav_for_measure_range(
    source: String,
    start_index: u32,
    end_index: u32,
    extend_to_last_occurrence: bool,
    respect_sequence: bool,
    sequence_entry_start_index: Option<u32>,
    sequence_entry_end_index: Option<u32>,
    enabled_tracks: Option<Vec<String>>,
    metronome: bool,
    trim_start_s: Option<f64>,
    trim_end_s: Option<f64>,
    trim_next_note_start_s: Option<f64>,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    generate_wav_response_to_wit(crate::responses::generate_wav_for_measure_range_response(
        &source,
        start_index as usize,
        end_index as usize,
        extend_to_last_occurrence,
        respect_sequence,
        crate::sequence_entry_range(
            sequence_entry_start_index.map(|v| v as usize),
            sequence_entry_end_index.map(|v| v as usize),
        ),
        enabled_tracks.as_deref(),
        metronome,
        crate::trim_window(trim_start_s, trim_end_s, trim_next_note_start_s),
        soundfont,
    ))
}

pub(super) fn generate_mp3_for_measure_range(
    source: String,
    start_index: u32,
    end_index: u32,
    extend_to_last_occurrence: bool,
    respect_sequence: bool,
    sequence_entry_start_index: Option<u32>,
    sequence_entry_end_index: Option<u32>,
    enabled_tracks: Option<Vec<String>>,
    trim_start_s: Option<f64>,
    trim_end_s: Option<f64>,
    trim_next_note_start_s: Option<f64>,
    soundfont: Vec<u8>,
) -> GenerateMp3Response {
    generate_mp3_response_to_wit(crate::responses::generate_mp3_for_measure_range_response(
        &source,
        start_index as usize,
        end_index as usize,
        extend_to_last_occurrence,
        respect_sequence,
        crate::sequence_entry_range(
            sequence_entry_start_index.map(|v| v as usize),
            sequence_entry_end_index.map(|v| v as usize),
        ),
        enabled_tracks.as_deref(),
        crate::trim_window(trim_start_s, trim_end_s, trim_next_note_start_s),
        soundfont,
    ))
}

pub(super) fn list_note_timings(
    source: String,
    visible_tracks: Option<Vec<String>>,
    enabled_tracks: Option<Vec<String>>,
) -> NoteTimingsResponse {
    note_timings_response_to_wit(crate::responses::list_note_timings_response(
        &source,
        visible_tracks.as_deref(),
        enabled_tracks.as_deref(),
    ))
}

pub(super) fn list_note_timings_for_range(
    source: String,
    start_index: u32,
    end_index: u32,
    extend_to_last_occurrence: bool,
    respect_sequence: bool,
    sequence_entry_start_index: Option<u32>,
    sequence_entry_end_index: Option<u32>,
    visible_tracks: Option<Vec<String>>,
    enabled_tracks: Option<Vec<String>>,
) -> NoteTimingsResponse {
    note_timings_response_to_wit(crate::responses::list_note_timings_for_range_response(
        &source,
        start_index as usize,
        end_index as usize,
        extend_to_last_occurrence,
        respect_sequence,
        crate::sequence_entry_range(
            sequence_entry_start_index.map(|v| v as usize),
            sequence_entry_end_index.map(|v| v as usize),
        ),
        visible_tracks.as_deref(),
        enabled_tracks.as_deref(),
    ))
}

pub(super) fn generate_instrument_preview_wav(
    program_number: u8,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    generate_wav_response_to_wit(crate::responses::generate_instrument_preview_wav_response(
        program_number,
        soundfont,
    ))
}

pub(super) fn generate_percussion_preview_wav(key: u8, soundfont: Vec<u8>) -> GenerateWavResponse {
    generate_wav_response_to_wit(crate::responses::generate_percussion_preview_wav_response(
        key, soundfont,
    ))
}

pub(super) fn note_selection_trim_window(
    cells: Vec<NoteCellIn>,
    note_timings: Vec<NoteTiming>,
) -> Option<TrimWindow> {
    let cells: Vec<crate::note_selection_types::NoteCellIn> =
        cells.into_iter().map(note_cell_in_from_wit).collect();
    let note_timings: Vec<crate::types::NoteTimingOut> =
        note_timings.into_iter().map(note_timing_from_wit).collect();
    crate::note_selection_trim::note_selection_trim_window(&cells, &note_timings).map(|window| {
        TrimWindow {
            start_s: window.start_s,
            end_s: window.end_s,
            next_note_start_s: window.next_note_start_s,
        }
    })
}
