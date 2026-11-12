//! What a score sings: each lyric part with a voice (`lyrics [S] { voice:
//! female }`), as the notes and syllables it sings. The singing itself is
//! `poentaro_singer::sing`, kept apart so a build that never sings doesn't
//! carry the voice.

mod sung_notes;

pub use poentaro_singer::{SingingPlan, SungNote, SungPart, SungSyllable, Voice};
pub use sung_notes::sung_notes;

use crate::ast::grouped::Score;
use crate::ast::parsed::SingingVoice;
use crate::error::IrrecoverableError;

/// Every voiced lyric part left in `score` (after visibility filtering), in
/// declaration order, ready for `poentaro_singer::sing` at `sample_rate`.
/// `score` must already be in playback order (see `midi::expand_navigation`).
/// No parts when nothing sings.
pub fn singing_plan(score: &Score, sample_rate: u32) -> Result<SingingPlan, IrrecoverableError> {
    let parts = voiced_lyric_parts(score)
        .into_iter()
        .map(|part| {
            Ok(SungPart {
                voice: part.voice,
                volume_percent: part.volume_percent,
                notes: sung_notes(score, &part.abbreviation)?,
            })
        })
        .collect::<Result<Vec<_>, IrrecoverableError>>()?;
    Ok(SingingPlan { sample_rate, parts })
}

struct VoicedLyricPart {
    abbreviation: String,
    voice: Voice,
    volume_percent: u8,
}

fn voiced_lyric_parts(score: &Score) -> Vec<VoicedLyricPart> {
    let mut parts: Vec<VoicedLyricPart> = Vec::new();
    let slices = score
        .measures
        .iter()
        .flat_map(|measure| measure.parts.iter().map(|row| row.slice()));
    for slice in slices {
        let (Some(name), Some(lyrics)) = (&slice.name, &slice.lyrics) else {
            continue;
        };
        let Some(voice) = lyrics.singing_voice else {
            continue;
        };
        if parts.iter().all(|part| &part.abbreviation != name) {
            parts.push(VoicedLyricPart {
                abbreviation: name.clone(),
                voice: singer_voice(voice),
                volume_percent: slice.volume,
            });
        }
    }
    parts
}

fn singer_voice(voice: SingingVoice) -> Voice {
    match voice {
        SingingVoice::Male => Voice::Male,
        SingingVoice::Female => Voice::Female,
        SingingVoice::Child => Voice::Child,
    }
}

#[cfg(test)]
#[path = "singing_plan_tests.rs"]
mod tests;
