//! Singing a score's singing parts for the CLI's audio exports.

use std::time::Instant;

use crate::ast::grouped::Score;
use crate::error::IrrecoverableError;

/// What `write_wav` / `write_mp3` take for `score` (as it will be played):
/// the instruments' MIDI, without the singing parts, and what they sing.
pub(crate) struct SungAudio {
    pub(crate) midi_bytes: Vec<u8>,
    pub(crate) vocals: Vec<f32>,
}

pub(crate) fn sung_audio(score: &Score) -> Result<SungAudio, IrrecoverableError> {
    Ok(SungAudio {
        midi_bytes: crate::midi::write_midi(&crate::audio_source::instruments_only(score.clone()))?,
        vocals: sung_vocals(score)?,
    })
}

/// The vocals `score` sings; empty when nothing sings.
fn sung_vocals(score: &Score) -> Result<Vec<f32>, IrrecoverableError> {
    let plan = crate::sing::singing_plan(score, crate::wav::SAMPLE_RATE)?;
    if plan.parts.is_empty() {
        return Ok(Vec::new());
    }
    let started = Instant::now();
    let vocals = poentaro_singer::sing(&plan);
    eprintln!(
        "sang {} part(s) in {:.1} s",
        plan.parts.len(),
        started.elapsed().as_secs_f64()
    );
    Ok(vocals)
}
