//! The instrument a `notes` part names on its `parts` line (e.g.
//! `Violin [V] = notes violin`), which sets the range its notes are checked
//! against and the sound it plays when the line quotes none (see
//! `syntax.md`, "Instrument range").

use super::Soundfont;

keyword_enum! {
    /// An instrument or voice whose playable range a part's notes are checked
    /// against. Only the range is checked, not which notes inside it the
    /// instrument can play.
    pub enum RangeInstrument {
        Guitar => "guitar",
        Violin => "violin",
        Cello => "cello",
        Liuqin => "liuqin",
        Guzheng => "guzheng",
        Harmonica => "harmonica",
        Harmonica16 => "harmonica16",
        Bass => "bass",
        Tenor => "tenor",
        Alto => "alto",
        Soprano => "soprano",
    }
}

/// An inclusive range of sounding pitches, as MIDI note numbers (C4 = 60).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PitchRange {
    pub lowest: u8,
    pub highest: u8,
}

impl RangeInstrument {
    /// The name the Edit Parts modal shows: the source keyword, plus the
    /// Chinese name where there is one.
    pub fn label(self) -> &'static str {
        match self {
            Self::Guitar => "guitar",
            Self::Violin => "violin",
            Self::Cello => "cello",
            Self::Liuqin => "liuqin 柳琴",
            Self::Guzheng => "guzheng 古箏",
            Self::Harmonica => "harmonica (12-hole)",
            Self::Harmonica16 => "harmonica (16-hole)",
            Self::Bass => "bass (voice)",
            Self::Tenor => "tenor",
            Self::Alto => "alto",
            Self::Soprano => "soprano",
        }
    }

    /// The sounding pitches this instrument can reach.
    pub fn range(self) -> PitchRange {
        let (lowest, highest) = match self {
            // Standard tuning E2 A2 D3 G3 B3 E4, up to the 19th fret.
            Self::Guitar => (40, 83),
            // Open G string to the end of the fingerboard.
            Self::Violin => (55, 105),
            // Standard tuning C2 G2 D3 A3, practical top.
            Self::Cello => (36, 76),
            // G-D-G-D tuning (G3 D4 G4 D5), practical top.
            Self::Liuqin => (55, 91),
            // 21 strings in D major pentatonic, four octaves.
            Self::Guzheng => (38, 86),
            // 12-hole chromatic in C, hole 1 blow to hole 12 blow with the slide.
            Self::Harmonica => (60, 97),
            // 16-hole chromatic in C, an octave lower at hole 1 (C3).
            Self::Harmonica16 => (48, 97),
            // SATB part-writing ranges.
            Self::Bass => (41, 62),
            Self::Tenor => (48, 67),
            Self::Alto => (55, 74),
            Self::Soprano => (60, 79),
        };
        PitchRange { lowest, highest }
    }

    /// The sound a part naming this instrument plays when its line quotes
    /// no soundfont: the closest General MIDI program.
    pub fn default_soundfont(self) -> Soundfont {
        Soundfont(match self {
            Self::Guitar => 24,                        // Acoustic Guitar (nylon)
            Self::Violin => 40,                        // Violin
            Self::Cello => 42,                         // Cello
            Self::Liuqin => 106, // Shamisen: GM has no liuqin, this is the closest plucked lute
            Self::Guzheng => 107, // Koto: the Japanese zither descended from the guzheng
            Self::Harmonica | Self::Harmonica16 => 22, // Harmonica
            Self::Bass | Self::Tenor | Self::Alto | Self::Soprano => 52, // Choir Aahs
        })
    }
}
