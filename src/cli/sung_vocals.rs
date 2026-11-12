//! Singing a score's voiced lyric parts for the CLI's audio exports.

use std::time::Instant;

use crate::ast::grouped::Score;
use crate::error::IrrecoverableError;

/// The vocals `score` (as it will be played) sings, for `write_wav` /
/// `write_mp3`; empty when no lyric part has a voice.
pub(crate) fn sung_vocals(score: &Score) -> Result<Vec<f32>, IrrecoverableError> {
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
