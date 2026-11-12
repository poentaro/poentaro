//! The chimera boundary: a plan's notes in, mono samples out.

mod lyrics;

use chimera::anatomy::human::{HumanBody, BODY_FEMALE, BODY_MALE};
use chimera::phonetics::sing::{midi_to_hz, plan_song, SongOptions};
use chimera::voice::human_voice::{HumanVoice, SourcePatch};

use crate::{SingingPlan, SungNote, SungSyllable, Voice};
use lyrics::chimera_notes;

/// The voice starts this long before a segment's first note, so a first
/// syllable's onset consonants (sung before the beat) aren't cut off.
const LEAD_SECONDS: f64 = 0.5;
/// Notes closer than this belong to one phrase (one breath).
const PHRASE_GAP_SECONDS: f64 = 0.02;
/// A rest at least this long lets the voice stop and start afresh.
const SEGMENT_GAP_SECONDS: f64 = 2.0;
/// Rendered past a segment's last planned gesture, for the voice to die away.
const TAIL_SECONDS: f64 = 0.4;
const OVERSAMPLE: usize = 2;
const VIBRATO_CENTS: f64 = 35.0;

impl Voice {
    fn body(self) -> HumanBody {
        match self {
            Voice::Soprano | Voice::Alto => BODY_FEMALE,
            Voice::Tenor | Voice::Bass => BODY_MALE,
        }
    }

    /// chimera's name for [`Voice::body`].
    fn name(self) -> &'static str {
        match self {
            Voice::Soprano | Voice::Alto => "female",
            Voice::Tenor | Voice::Bass => "male",
        }
    }
}

/// Every part in `plan` sung and summed into one mono track at the plan's
/// sample rate, starting at the score's time zero. Each part peaks at its
/// volume, where 1.0 is the instruments' peak.
pub fn sing(plan: &SingingPlan) -> Vec<f32> {
    let sample_rate = f64::from(plan.sample_rate);
    plan.parts.iter().fold(Vec::new(), |mut mix, part| {
        let peak = f32::from(part.volume_percent) / 100.0;
        let samples = scaled_to_peak(render_part(&part.notes, part.voice, sample_rate), peak);
        add_at(&mut mix, 0, &samples);
        mix
    })
}

/// One part's voice, only run around what it sings: phrases close together
/// share one voice (so breaths and pitch carry over), and a long rest starts
/// a fresh one, so instrumental stretches cost nothing.
fn render_part(notes: &[SungNote], voice: Voice, sample_rate: f64) -> Vec<f32> {
    let mut output = Vec::new();
    for segment in segments(phrases(notes)) {
        let Some(first) = segment.first().and_then(|phrase| phrase.first()) else {
            continue;
        };
        let origin = first.start_seconds - LEAD_SECONDS;
        let samples = render_segment(&segment, voice, origin, sample_rate);
        let offset = (origin * sample_rate).round() as i64;
        let skip = usize::try_from(-offset).unwrap_or(0);
        let start = usize::try_from(offset).unwrap_or(0);
        add_at(&mut output, start, samples.get(skip..).unwrap_or_default());
    }
    output
}

/// One voice singing `phrases`, with voice time zero at score time `origin`.
fn render_segment(
    phrases: &[Vec<&SungNote>],
    voice: Voice,
    origin: f64,
    sample_rate: f64,
) -> Vec<f32> {
    let mut human = HumanVoice::new(sample_rate, voice.body(), OVERSAMPLE);
    if let Some(first) = phrases.first().and_then(|phrase| phrase.first()) {
        human.set_source(&SourcePatch {
            f0: Some(midi_to_hz(f64::from(first.midi))),
            voiced: Some(true),
            pressure: Some(9.0),
            tension: Some(0.55),
            ..Default::default()
        });
    }
    let end = phrases
        .iter()
        .filter_map(|phrase| {
            let start = phrase.first()?.start_seconds;
            let plan = plan_song(
                &chimera_notes(phrase),
                &SongOptions {
                    body: voice.name().into(),
                    // One beat is one second, so beats are seconds and
                    // tempo changes inside a phrase need no special care.
                    bpm: 60.0,
                    t0: start - origin,
                    vibrato: VIBRATO_CENTS,
                    ..Default::default()
                },
            );
            human.schedule(&plan.gestures);
            Some(plan.end)
        })
        .fold(0.0f64, f64::max);
    let mut samples = vec![0f32; ((end + TAIL_SECONDS) * sample_rate) as usize];
    samples
        .chunks_mut(128)
        .for_each(|block| human.render(block));
    samples
}

/// Phrases grouped so that a rest of [`SEGMENT_GAP_SECONDS`] or more starts
/// a new group.
fn segments(phrases: Vec<Vec<&SungNote>>) -> Vec<Vec<Vec<&SungNote>>> {
    let mut segments: Vec<Vec<Vec<&SungNote>>> = Vec::new();
    for phrase in phrases {
        let previous_end = segments
            .last()
            .and_then(|segment| segment.last())
            .and_then(|previous| previous.last())
            .map(|previous| previous.end_seconds);
        let joins = previous_end
            .zip(phrase.first())
            .is_some_and(|(end, next)| next.start_seconds - end < SEGMENT_GAP_SECONDS);
        match segments.last_mut() {
            Some(segment) if joins => segment.push(phrase),
            _ => segments.push(vec![phrase]),
        }
    }
    segments
}

/// Runs of sung notes with no rest between them. A held note with nothing
/// before it to hold is dropped.
fn phrases(notes: &[SungNote]) -> Vec<Vec<&SungNote>> {
    let mut phrases: Vec<Vec<&SungNote>> = Vec::new();
    let mut previous_end: Option<f64> = None;
    for note in notes {
        let joins = previous_end.is_some_and(|end| note.start_seconds - end < PHRASE_GAP_SECONDS);
        match (&note.syllable, joins, phrases.last_mut()) {
            (SungSyllable::Silent, _, _) | (SungSyllable::Held, false, _) => {
                previous_end = None;
                continue;
            }
            (_, true, Some(phrase)) => phrase.push(note),
            _ => phrases.push(vec![note]),
        }
        previous_end = Some(note.end_seconds);
    }
    phrases
}

/// Adds `samples` into `output` from index `start`, growing it as needed.
fn add_at(output: &mut Vec<f32>, start: usize, samples: &[f32]) {
    let end = start + samples.len();
    if output.len() < end {
        output.resize(end, 0.0);
    }
    output
        .iter_mut()
        .skip(start)
        .zip(samples)
        .for_each(|(out, sample)| *out += sample);
}

fn scaled_to_peak(samples: Vec<f32>, peak: f32) -> Vec<f32> {
    let current = samples
        .iter()
        .fold(0.0f32, |max, sample| max.max(sample.abs()));
    if current == 0.0 {
        return samples;
    }
    samples
        .into_iter()
        .map(|sample| sample / current * peak)
        .collect()
}

#[cfg(test)]
mod tests;
