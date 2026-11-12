//! Mixing a sung vocal track into the rendered instruments.

/// Adds `vocals` (mono) to both channels, scaled by the instruments' own
/// peak so a vocal sample of 1.0 is as loud as the loudest instrument
/// moment; the instruments are padded with silence when the vocals run on.
pub(super) fn mix_vocals(left: &mut Vec<f32>, right: &mut Vec<f32>, vocals: &[f32]) {
    let instrument_peak = left
        .iter()
        .chain(right.iter())
        .fold(0.0f32, |peak, sample| peak.max(sample.abs()));
    let scale = if instrument_peak > 0.0 {
        instrument_peak
    } else {
        1.0
    };
    for channel in [left, right] {
        if channel.len() < vocals.len() {
            channel.resize(vocals.len(), 0.0);
        }
        channel
            .iter_mut()
            .zip(vocals)
            .for_each(|(sample, vocal)| *sample += vocal * scale);
    }
}

#[cfg(test)]
#[path = "wav_vocals_tests.rs"]
mod tests;
