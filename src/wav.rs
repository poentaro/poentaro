use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};
use hound::{SampleFormat, WavSpec, WavWriter};
use midly::num::u4;
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use oxisynth::{MidiEvent, SoundFont, Synth, SynthDescriptor};
use std::io::Cursor;

pub(crate) const SAMPLE_RATE: u32 = 44100;
/// Target peak level before encoding (0.95 ≈ −0.4 dBFS), matching typical mastered music.
const TARGET_PEAK: f32 = 0.95;

#[path = "wav_loudness.rs"]
mod wav_loudness;
use wav_loudness::balancing_expression;

/// The General MIDI drum channel, whose program picks a kit rather than an
/// instrument, so it gets no loudness balancing.
const GM_PERCUSSION_CHANNEL: u8 = 9;
/// MIDI controller 11 (expression), which the loudness balancing uses.
const EXPRESSION_CONTROLLER: u8 = 11;

#[path = "wav_trim.rs"]
mod wav_trim;
use wav_trim::trim_and_fade;
pub use wav_trim::TrimWindow;

#[cfg(feature = "mp3")]
#[path = "wav_vocals.rs"]
mod wav_vocals;
use wav_vocals::mix_vocals;

#[path = "wav_mp3.rs"]
mod wav_mp3;
#[cfg(feature = "mp3")]
use wav_mp3::encode_mp3;

fn init_synth(sf2_bytes: &[u8]) -> Result<Synth, IrrecoverableError> {
    let mut synth = Synth::new(SynthDescriptor {
        sample_rate: SAMPLE_RATE as f32,
        ..Default::default()
    })
    .map_err(|_| {
        IrrecoverableError::new(IrrecoverableErrorKind::WavSynthInitFailed {
            span: Span::new(0, 0),
        })
    })?;

    let sf = SoundFont::load(&mut Cursor::new(sf2_bytes)).map_err(|_| {
        IrrecoverableError::new(IrrecoverableErrorKind::WavSoundfontLoadFailed {
            span: Span::new(0, 0),
        })
    })?;
    synth.add_font(sf, true);
    Ok(synth)
}

fn handle_midi_message(synth: &mut Synth, channel: u4, message: &MidiMessage) {
    let ch = channel.as_int();
    match message {
        MidiMessage::ProgramChange { program } => {
            synth
                .send_event(MidiEvent::ProgramChange {
                    channel: ch,
                    program_id: program.as_int(),
                })
                .ok();
            if ch != GM_PERCUSSION_CHANNEL {
                synth
                    .send_event(MidiEvent::ControlChange {
                        channel: ch,
                        ctrl: EXPRESSION_CONTROLLER,
                        value: balancing_expression(program.as_int()),
                    })
                    .ok();
            }
        }
        MidiMessage::NoteOn { key, vel } => {
            synth
                .send_event(MidiEvent::NoteOn {
                    channel: ch,
                    key: key.as_int(),
                    vel: vel.as_int(),
                })
                .ok();
        }
        MidiMessage::NoteOff { key, .. } => {
            synth
                .send_event(MidiEvent::NoteOff {
                    channel: ch,
                    key: key.as_int(),
                })
                .ok();
        }
        MidiMessage::Controller { controller, value } => {
            synth
                .send_event(MidiEvent::ControlChange {
                    channel: ch,
                    ctrl: controller.as_int(),
                    value: value.as_int(),
                })
                .ok();
        }
        _ => {}
    }
}

fn render_track(
    synth: &mut Synth,
    track: &midly::Track<'_>,
    tpq: u32,
    all_l: &mut Vec<f32>,
    all_r: &mut Vec<f32>,
) {
    let mut micros_per_beat: u32 = 500_000;
    for event in track.iter() {
        let delta = event.delta.as_int();
        if delta > 0 {
            let n = ticks_to_samples(delta, tpq, micros_per_beat);
            render_samples(synth, n, all_l, all_r);
        }
        match &event.kind {
            TrackEventKind::Meta(MetaMessage::Tempo(t)) => {
                micros_per_beat = t.as_int();
            }
            TrackEventKind::Midi { channel, message } => {
                handle_midi_message(synth, *channel, message);
            }
            _ => {}
        }
    }
}

/// Parse MIDI, render it through the synth, render a decay tail, optionally
/// trim/fade to a window, and normalize peak level. Shared by [`write_wav`]
/// and (behind the `mp3` feature) [`write_mp3`] — the two differ only in how
/// the resulting `(left, right)` samples are encoded to bytes.
fn render_score_to_samples(
    midi_bytes: &[u8],
    sf2_bytes: &[u8],
    trim: Option<TrimWindow>,
    vocals: &[f32],
) -> Result<(Vec<f32>, Vec<f32>), IrrecoverableError> {
    let smf = Smf::parse(midi_bytes).map_err(|_| {
        IrrecoverableError::new(IrrecoverableErrorKind::WavInvalidMidiBytes {
            span: Span::new(0, 0),
        })
    })?;
    let tpq = match smf.header.timing {
        Timing::Metrical(t) => t.as_int() as u32,
        Timing::Timecode(..) => 480,
    };

    let mut synth = init_synth(sf2_bytes)?;
    let mut all_l: Vec<f32> = Vec::new();
    let mut all_r: Vec<f32> = Vec::new();

    let track = smf.tracks.first().ok_or_else(|| {
        IrrecoverableError::new(IrrecoverableErrorKind::internal_invariant(
            Span::new(0, 0),
            "internal invariant: MIDI file has no tracks",
        ))
    })?;
    render_track(&mut synth, track, tpq, &mut all_l, &mut all_r);

    // Render 1 second of tail so reverb fully decays
    render_samples(&mut synth, SAMPLE_RATE as usize, &mut all_l, &mut all_r);

    mix_vocals(&mut all_l, &mut all_r, vocals);

    if let Some(trim) = trim {
        trim_and_fade(&mut all_l, &mut all_r, trim);
    }

    normalize_peak(&mut all_l, &mut all_r);
    Ok((all_l, all_r))
}

/// Synthesize a score to WAV bytes. `vocals` is a mono track at
/// [`SAMPLE_RATE`] mixed in the centre (see `mix_vocals`); empty when
/// nothing sings.
pub fn write_wav(
    midi_bytes: &[u8],
    sf2_bytes: &[u8],
    trim: Option<TrimWindow>,
    vocals: &[f32],
) -> Result<Vec<u8>, IrrecoverableError> {
    let (all_l, all_r) = render_score_to_samples(midi_bytes, sf2_bytes, trim, vocals)?;
    encode_wav(&all_l, &all_r)
}

/// Synthesize a score to 128kbps CBR MP3 bytes — the same rendering pipeline
/// as [`write_wav`], only the final encode step differs. No user-facing
/// bitrate knob: this project's scores are small and largely monophonic, so
/// a fixed rate is enough.
///
/// Available only when the `mp3` feature is enabled at build time.
#[cfg(feature = "mp3")]
pub fn write_mp3(
    midi_bytes: &[u8],
    sf2_bytes: &[u8],
    trim: Option<TrimWindow>,
    vocals: &[f32],
) -> Result<Vec<u8>, IrrecoverableError> {
    let (all_l, all_r) = render_score_to_samples(midi_bytes, sf2_bytes, trim, vocals)?;
    encode_mp3(&all_l, &all_r)
}

pub fn write_preview_wav(
    program_number: u8,
    sf2_bytes: &[u8],
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut synth = init_synth(sf2_bytes)?;
    let mut all_l = Vec::new();
    let mut all_r = Vec::new();

    // do re mi sol (scale degrees 1 2 3 5 in C major)
    let melody: [u8; 4] = [60, 62, 64, 67];
    let sustain_samples = (SAMPLE_RATE as f32 * 0.35) as usize;
    let gap_samples = (SAMPLE_RATE as f32 * 0.05) as usize;

    synth
        .send_event(MidiEvent::ProgramChange {
            channel: 0,
            program_id: program_number,
        })
        .ok();
    for key in melody {
        synth
            .send_event(MidiEvent::NoteOn {
                channel: 0,
                key,
                vel: 80,
            })
            .ok();
        render_samples(&mut synth, sustain_samples, &mut all_l, &mut all_r);
        synth
            .send_event(MidiEvent::NoteOff { channel: 0, key })
            .ok();
        render_samples(&mut synth, gap_samples, &mut all_l, &mut all_r);
    }
    render_samples(&mut synth, SAMPLE_RATE as usize / 2, &mut all_l, &mut all_r); // 0.5 s tail

    normalize_peak(&mut all_l, &mut all_r);
    encode_wav(&all_l, &all_r)
}

/// Percussion channel/program matching [`crate::midi::mod`]'s GM Standard Kit preamble, so
/// the preview sounds like the same kit that renders in exported audio.
const PERCUSSION_PREVIEW_CHANNEL: u8 = GM_PERCUSSION_CHANNEL;
const GM_STANDARD_KIT_PROGRAM: u8 = 0;

pub fn write_percussion_preview_wav(
    key: u8,
    sf2_bytes: &[u8],
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut synth = init_synth(sf2_bytes)?;
    let mut all_l = Vec::new();
    let mut all_r = Vec::new();

    let hit_samples = (SAMPLE_RATE as f32 * 0.35) as usize;
    let gap_samples = (SAMPLE_RATE as f32 * 0.15) as usize;

    synth
        .send_event(MidiEvent::ProgramChange {
            channel: PERCUSSION_PREVIEW_CHANNEL,
            program_id: GM_STANDARD_KIT_PROGRAM,
        })
        .ok();
    for _ in 0..2 {
        synth
            .send_event(MidiEvent::NoteOn {
                channel: PERCUSSION_PREVIEW_CHANNEL,
                key,
                vel: 100,
            })
            .ok();
        render_samples(&mut synth, hit_samples, &mut all_l, &mut all_r);
        synth
            .send_event(MidiEvent::NoteOff {
                channel: PERCUSSION_PREVIEW_CHANNEL,
                key,
            })
            .ok();
        render_samples(&mut synth, gap_samples, &mut all_l, &mut all_r);
    }
    render_samples(&mut synth, SAMPLE_RATE as usize / 2, &mut all_l, &mut all_r); // 0.5 s tail

    normalize_peak(&mut all_l, &mut all_r);
    encode_wav(&all_l, &all_r)
}

#[cfg(test)]
#[path = "wav_loudness_tests.rs"]
mod wav_loudness_tests;

#[cfg(test)]
#[path = "wav_percussion_tests.rs"]
mod wav_percussion_tests;

fn normalize_peak(left: &mut [f32], right: &mut [f32]) {
    let peak = left
        .iter()
        .chain(right.iter())
        .map(|sample| sample.abs())
        .fold(0.0f32, f32::max);
    if peak <= 0.0 {
        return;
    }
    let gain = TARGET_PEAK / peak;
    for sample in left.iter_mut().chain(right.iter_mut()) {
        *sample *= gain;
    }
}

fn ticks_to_samples(ticks: u32, tpq: u32, micros_per_beat: u32) -> usize {
    ((ticks as f64 * SAMPLE_RATE as f64 * micros_per_beat as f64) / (tpq as f64 * 1_000_000.0))
        as usize
}

fn render_samples(synth: &mut Synth, n: usize, l: &mut Vec<f32>, r: &mut Vec<f32>) {
    let prev = l.len();
    l.resize(prev + n, 0.0);
    r.resize(prev + n, 0.0);
    let l_tail = l.split_at_mut(prev).1;
    let r_tail = r.split_at_mut(prev).1;
    synth.write_f32(n, l_tail, 0, 1, r_tail, 0, 1);
}

fn encode_wav(l: &[f32], r: &[f32]) -> Result<Vec<u8>, IrrecoverableError> {
    let spec = WavSpec {
        channels: 2,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut buf: Vec<u8> = Vec::new();
    let mut writer = WavWriter::new(Cursor::new(&mut buf), spec).map_err(|e| {
        IrrecoverableError::new(IrrecoverableErrorKind::WavWriterCreateFailed {
            span: Span::new(0, 0),
            source: e.to_string(),
        })
    })?;
    for (ls, rs) in l.iter().zip(r.iter()) {
        writer
            .write_sample((ls.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
            .map_err(|e| {
                IrrecoverableError::new(IrrecoverableErrorKind::WavWriteSampleFailed {
                    span: Span::new(0, 0),
                    source: e.to_string(),
                })
            })?;
        writer
            .write_sample((rs.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
            .map_err(|e| {
                IrrecoverableError::new(IrrecoverableErrorKind::WavWriteSampleFailed {
                    span: Span::new(0, 0),
                    source: e.to_string(),
                })
            })?;
    }
    writer.finalize().map_err(|e| {
        IrrecoverableError::new(IrrecoverableErrorKind::WavFinalizeFailed {
            span: Span::new(0, 0),
            source: e.to_string(),
        })
    })?;
    Ok(buf)
}

#[cfg(test)]
#[path = "wav_tests.rs"]
mod tests;
