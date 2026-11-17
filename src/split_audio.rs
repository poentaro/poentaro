//! Split-track audio export: one WAV or MP3 per track, each singing its own
//! voice when the track is a singing part.

use crate::ast::grouped::Score;
use crate::error::IrrecoverableError;
use crate::filters::filter_tracks;
use crate::sing::SingingPlan;
use crate::split_track::{
    part_display_name_map, split_track_filename, split_track_label, split_track_names,
    track_with_its_lyric_parts, SplitFileEntry, SplitSelection,
};

/// What one track of a split audio export sings (see
/// [`split_singing_plans_from_source`]).
pub struct TrackSingingPlan {
    pub track_name: String,
    pub plan: SingingPlan,
}

/// One track's sung samples, from `poentaro_singer::sing` of its
/// [`TrackSingingPlan`].
pub struct TrackVocals {
    pub track_name: String,
    pub samples: Vec<f32>,
}

struct SplitTrack {
    name: String,
    filename_label: String,
    score: Score,
}

/// Each track of a split export, as the score it plays on its own: the track
/// with the lyric parts sung on it.
fn split_tracks(
    source: &str,
    filename: &str,
    selection: SplitSelection,
) -> Result<Vec<SplitTrack>, IrrecoverableError> {
    let score = crate::compile_with_preset(source, filename, &[], selection.preset_name)?;
    let display_names = part_display_name_map(source, filename)?;
    Ok(split_track_names(source, filename, &score, selection)?
        .into_iter()
        .map(|track| {
            let mut track_score = score.clone();
            filter_tracks(
                &mut track_score,
                &track_with_its_lyric_parts(&score, &track),
            );
            SplitTrack {
                filename_label: split_track_label(&display_names, &track),
                name: track,
                score: track_score,
            }
        })
        .collect())
}

/// What each track of a split audio export of the same arguments sings; only
/// the tracks that sing.
pub fn split_singing_plans_from_source(
    source: &str,
    filename: &str,
    selection: SplitSelection,
) -> Result<Vec<TrackSingingPlan>, IrrecoverableError> {
    split_tracks(source, filename, selection)?
        .into_iter()
        .map(|track| {
            Ok(TrackSingingPlan {
                plan: crate::sing::singing_plan(&track.score, crate::wav::SAMPLE_RATE)?,
                track_name: track.name,
            })
        })
        .filter(|plan| {
            plan.as_ref()
                .map_or(true, |plan| !plan.plan.parts.is_empty())
        })
        .collect()
}

/// How one split track's audio file is made from its MIDI and vocals.
struct SplitAudioFormat<Encode> {
    encode: Encode,
    extension: &'static str,
}

fn write_split_audio(
    source: &str,
    filename: &str,
    base_name: &str,
    selection: SplitSelection,
    vocals: &[TrackVocals],
    format: SplitAudioFormat<impl Fn(&[u8], &[f32]) -> Result<Vec<u8>, IrrecoverableError>>,
) -> Result<Vec<SplitFileEntry>, IrrecoverableError> {
    let SplitAudioFormat { encode, extension } = format;
    let base_name = selection.file_base_name(base_name);
    split_tracks(source, filename, selection)?
        .into_iter()
        .map(|track| {
            let midi_bytes =
                crate::midi::write_midi(&crate::audio_source::instruments_only(track.score))?;
            let samples = vocals
                .iter()
                .find(|vocals| vocals.track_name == track.name)
                .map_or(&[][..], |vocals| vocals.samples.as_slice());
            Ok(SplitFileEntry {
                filename: split_track_filename(&base_name, &track.filename_label, extension),
                bytes: encode(&midi_bytes, samples)?,
                track_name: track.name,
            })
        })
        .collect()
}

/// Parse once, synthesize one WAV file per track, mixing in each track's
/// `vocals` (see [`split_singing_plans_from_source`]).
pub fn write_split_wavs_from_source(
    source: &str,
    filename: &str,
    base_name: &str,
    selection: SplitSelection,
    sf2_bytes: &[u8],
    vocals: &[TrackVocals],
) -> Result<Vec<SplitFileEntry>, IrrecoverableError> {
    let encode = |midi: &[u8], vocals: &[f32]| crate::wav::write_wav(midi, sf2_bytes, None, vocals);
    let format = SplitAudioFormat {
        encode,
        extension: "wav",
    };
    write_split_audio(source, filename, base_name, selection, vocals, format)
}

/// Parse once, synthesize one MP3 file per track, mixing in each track's
/// `vocals` (see [`split_singing_plans_from_source`]).
#[cfg(feature = "mp3")]
pub fn write_split_mp3s_from_source(
    source: &str,
    filename: &str,
    base_name: &str,
    selection: SplitSelection,
    sf2_bytes: &[u8],
    vocals: &[TrackVocals],
) -> Result<Vec<SplitFileEntry>, IrrecoverableError> {
    let encode = |midi: &[u8], vocals: &[f32]| crate::wav::write_mp3(midi, sf2_bytes, None, vocals);
    let format = SplitAudioFormat {
        encode,
        extension: "mp3",
    };
    write_split_audio(source, filename, base_name, selection, vocals, format)
}
