//! What a score sings: each notes part whose sound is a Chimera voice
//! (`sound: chimera soprano`), as the notes it sings with the syllables of
//! the lyric part sung on it (`lyrics [S, A]`). The singing itself is
//! `poentaro_singer::sing`, kept apart so a build that never sings doesn't
//! carry the voice.

mod singing_tracks;
mod sung_notes;

pub use poentaro_singer::{SingingPlan, SungNote, SungPart, SungSyllable, Voice};
pub use singing_tracks::singing_tracks;
pub use sung_notes::sung_notes;

use crate::ast::grouped::Score;
use crate::ast::parsed::SingingVoice;
use crate::error::IrrecoverableError;

/// Every singing part left in `score` (after visibility filtering) that has
/// a word to sing, in order of first appearance, ready for `poentaro_singer::sing` at `sample_rate`.
/// `score` must already be in playback order (see `midi::expand_navigation`).
/// No parts when nothing sings.
pub fn singing_plan(score: &Score, sample_rate: u32) -> Result<SingingPlan, IrrecoverableError> {
    let parts = singing_parts(score)
        .into_iter()
        .map(|part| {
            Ok(SungPart {
                voice: part.voice,
                volume_percent: part.volume_percent,
                notes: sung_notes(score, &part.abbreviation)?,
            })
        })
        .filter(|part| part.as_ref().map_or(true, sings_a_word))
        .collect::<Result<Vec<_>, IrrecoverableError>>()?;
    Ok(SingingPlan { sample_rate, parts })
}

fn sings_a_word(part: &SungPart) -> bool {
    part.notes
        .iter()
        .any(|note| matches!(note.syllable, SungSyllable::Word(_)))
}

struct SingingPart {
    abbreviation: String,
    voice: Voice,
    volume_percent: u8,
}

fn singing_parts(score: &Score) -> Vec<SingingPart> {
    let mut parts: Vec<SingingPart> = Vec::new();
    let slices = score
        .measures
        .iter()
        .flat_map(|measure| measure.parts.iter().map(|row| row.slice()));
    for slice in slices {
        let (Some(name), Some(voice)) = (&slice.name, slice.singing_voice) else {
            continue;
        };
        if slice.is_alternative_line() {
            continue;
        }
        if parts.iter().all(|part| &part.abbreviation != name) {
            parts.push(SingingPart {
                abbreviation: name.clone(),
                voice: singer_voice(voice),
                volume_percent: slice.volume,
            });
        }
    }
    parts
}

/// Silences every singing part's instrument in `score`, for audio where its
/// voice sings in its place. A MIDI file keeps the part's sound, as a voice
/// can't be written there.
pub fn silence_singing_parts(score: &mut Score) {
    score
        .measures
        .iter_mut()
        .flat_map(|measure| measure.parts.iter_mut().map(|row| row.slice_mut()))
        .filter(|slice| slice.singing_voice.is_some())
        .for_each(|slice| slice.volume = 0);
}

fn singer_voice(voice: SingingVoice) -> Voice {
    match voice {
        SingingVoice::Soprano => Voice::Soprano,
        SingingVoice::Alto => Voice::Alto,
        SingingVoice::Tenor => Voice::Tenor,
        SingingVoice::Bass => Voice::Bass,
    }
}

#[cfg(test)]
#[path = "singing_plan_tests.rs"]
mod tests;

#[cfg(all(test, feature = "wav"))]
#[path = "singing_tracks_tests.rs"]
mod singing_tracks_tests;
