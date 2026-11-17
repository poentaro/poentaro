use super::{encode_wav, normalize_peak, ticks_to_samples, TARGET_PEAK};

#[test]
fn ticks_to_samples_quarter_note_at_120bpm() {
    // 120 BPM = 500_000 µs/beat, TPQ = 480
    // quarter note = 480 ticks = 0.5 s = 22050 samples @ 44100 Hz
    assert_eq!(ticks_to_samples(480, 480, 500_000), 22050);
}

#[test]
fn ticks_to_samples_half_note_at_120bpm() {
    assert_eq!(ticks_to_samples(960, 480, 500_000), 44100);
}

#[test]
fn encode_wav_has_riff_wave_header() {
    let l = vec![0.0f32; 44100];
    let r = vec![0.0f32; 44100];
    let bytes = encode_wav(&l, &r).unwrap();
    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");
}

#[test]
fn normalize_peak_scales_to_target() {
    let mut left = vec![0.1f32, -0.2, 0.05];
    let mut right = vec![0.15f32, -0.1, 0.3];
    normalize_peak(&mut left, &mut right);
    let peak = left
        .iter()
        .chain(right.iter())
        .map(|sample| sample.abs())
        .fold(0.0f32, f32::max);
    assert!((peak - TARGET_PEAK).abs() < 1e-6);
}

#[test]
fn normalize_peak_leaves_silence_unchanged() {
    let mut left = vec![0.0f32; 4];
    let mut right = vec![0.0f32; 4];
    normalize_peak(&mut left, &mut right);
    assert!(left.iter().all(|sample| *sample == 0.0));
    assert!(right.iter().all(|sample| *sample == 0.0));
}

#[test]
fn encode_wav_stereo_16bit_44100() {
    let l = vec![0.0f32; 100];
    let r = vec![0.0f32; 100];
    let bytes = encode_wav(&l, &r).unwrap();
    // WAV spec chunk: channels=2, sample_rate=44100, bits=16
    // bytes 22-23: channels (little-endian u16)
    assert_eq!(u16::from_le_bytes([bytes[22], bytes[23]]), 2);
    // bytes 24-27: sample rate (little-endian u32)
    assert_eq!(
        u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]),
        44100
    );
    // bytes 34-35: bits per sample (little-endian u16)
    assert_eq!(u16::from_le_bytes([bytes[34], bytes[35]]), 16);
}
