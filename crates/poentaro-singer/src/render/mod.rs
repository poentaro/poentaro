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
    mix(plan, &sing_pieces(plan))
}

/// One stretch of one part that a voice sings on its own (see [`segments`]),
/// sung. Pieces don't depend on each other, so a song's pieces can be sung
/// in parallel and then [`mix`]ed.
#[derive(Debug, Clone, PartialEq)]
pub struct SungPiece {
    /// The index of its part in the plan.
    pub part: usize,
    /// Where it starts in the song, in samples from time zero.
    pub start_sample: usize,
    pub samples: Vec<f32>,
}

struct Piece<'a> {
    part: usize,
    voice: Voice,
    phrases: Vec<Vec<&'a SungNote>>,
}

fn pieces(plan: &SingingPlan) -> impl Iterator<Item = Piece<'_>> {
    plan.parts.iter().enumerate().flat_map(|(part, sung)| {
        segments(phrases(&sung.notes))
            .into_iter()
            .map(move |phrases| Piece {
                part,
                voice: sung.voice,
                phrases,
            })
    })
}

/// How many pieces [`sing_piece`] can sing for `plan`.
pub fn piece_count(plan: &SingingPlan) -> usize {
    pieces(plan).count()
}

/// The piece at `index` (below [`piece_count`]) sung; `None` past the last.
pub fn sing_piece(plan: &SingingPlan, index: usize) -> Option<SungPiece> {
    let piece = pieces(plan).nth(index)?;
    Some(render_piece(&piece, f64::from(plan.sample_rate)))
}

/// `pieces` (in any order) summed into one track, each part scaled so it
/// peaks at its volume.
pub fn mix(plan: &SingingPlan, pieces: &[SungPiece]) -> Vec<f32> {
    plan.parts
        .iter()
        .enumerate()
        .fold(Vec::new(), |mut mix, (index, part)| {
            let voice = pieces.iter().filter(|piece| piece.part == index).fold(
                Vec::new(),
                |mut voice, piece| {
                    add_at(&mut voice, piece.start_sample, &piece.samples);
                    voice
                },
            );
            let peak = f32::from(part.volume_percent) / 100.0;
            add_at(&mut mix, 0, &scaled_to_peak(voice, peak));
            mix
        })
}

/// Every piece sung, one thread per core.
#[cfg(not(target_arch = "wasm32"))]
fn sing_pieces(plan: &SingingPlan) -> Vec<SungPiece> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let count = piece_count(plan);
    let threads = std::thread::available_parallelism()
        .map_or(1, std::num::NonZeroUsize::get)
        .min(count);
    let next = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        let workers = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    std::iter::from_fn(|| sing_piece(plan, next.fetch_add(1, Ordering::Relaxed)))
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>();
        workers
            .into_iter()
            .flat_map(|worker| worker.join().unwrap_or_default())
            .collect()
    })
}

/// Every piece sung in turn: wasm has no threads (the web app spreads pieces
/// over web workers instead).
#[cfg(target_arch = "wasm32")]
fn sing_pieces(plan: &SingingPlan) -> Vec<SungPiece> {
    (0..piece_count(plan))
        .filter_map(|index| sing_piece(plan, index))
        .collect()
}

/// One piece's voice, placed in the song. Its voice starts a little before
/// its first note, so the first syllable's onset isn't cut off.
fn render_piece(piece: &Piece, sample_rate: f64) -> SungPiece {
    let origin = piece
        .phrases
        .first()
        .and_then(|phrase| phrase.first())
        .map_or(0.0, |first| first.start_seconds - LEAD_SECONDS);
    let samples = render_segment(&piece.phrases, piece.voice, origin, sample_rate);
    let offset = (origin * sample_rate).round() as i64;
    let skip = usize::try_from(-offset).unwrap_or(0);
    SungPiece {
        part: piece.part,
        start_sample: usize::try_from(offset).unwrap_or(0),
        samples: samples.get(skip..).unwrap_or_default().to_vec(),
    }
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
