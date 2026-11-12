//! The voice a `notes` part sings in when its sound is a Chimera voice
//! (`notes { sound: chimera soprano }`, or by default for a choral
//! instrument; see `syntax.md`, "Singing").

use super::RangeInstrument;

keyword_enum! {
    /// A synthesized choir voice. A part with one sings the syllables of the
    /// lyric part bound to it instead of playing a General MIDI sound.
    pub enum SingingVoice {
        Soprano => "soprano",
        Alto => "alto",
        Tenor => "tenor",
        Bass => "bass",
    }
}

impl SingingVoice {
    /// The word before the voice in `sound: chimera soprano`.
    pub const SOUND_WORD: &'static str = "chimera";

    /// The voice a part naming `instrument` sings in when it names no sound.
    pub fn default_for(instrument: RangeInstrument) -> Option<Self> {
        match instrument {
            RangeInstrument::Soprano => Some(Self::Soprano),
            RangeInstrument::Alto => Some(Self::Alto),
            RangeInstrument::Tenor => Some(Self::Tenor),
            RangeInstrument::Bass => Some(Self::Bass),
            _ => None,
        }
    }
}
