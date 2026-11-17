//! A chord symbol's harmony as written in degrees, formatted either as its
//! numbered symbol (`♯1m⁷/5`) or, in the letter-name view, as letter names
//! (`F♯m⁷/C♯`).

use crate::ast::grouped::GroupedChordNote;
use crate::ast::parsed::{Accidental, BassDegree, Extension, NumberedPitch, TriadQuality};
use crate::font_metrics;
use crate::pitch_description::spelling::SpelledPitch;

/// The chord a `ChordSymbol` was formatted from: its root and optional slash
/// bass as scale degrees, and its quality.
#[derive(Debug, Clone, PartialEq)]
pub struct ChordDegrees {
    pub degree: NumberedPitch,
    pub accidental: Accidental,
    pub triad: TriadQuality,
    pub extension: Option<Extension>,
    pub bass: Option<BassDegree>,
}

impl ChordDegrees {
    pub(super) fn of(chord: &GroupedChordNote) -> Self {
        ChordDegrees {
            degree: chord.degree.clone(),
            accidental: chord.accidental.clone(),
            triad: chord.triad.clone(),
            extension: chord.extension.clone(),
            bass: chord.bass.clone(),
        }
    }

    /// The numbered symbol, e.g. `♯1m⁷/5`. Accidentals lead their degree,
    /// matching note heads — see `render_chord_symbol`, which hangs the
    /// leading accidental into the column's accidental lead.
    pub(super) fn degree_symbol(&self) -> String {
        self.symbol(|degree, accidental| {
            format!(
                "{}{}",
                font_metrics::accidental_symbol(accidental),
                degree.to_digit()
            )
        })
    }

    /// The symbol in letter names against the key whose tonic is `tonic`,
    /// e.g. `F♯m⁷/C♯`. Accidentals trail their letter, so the column needs no
    /// accidental lead.
    pub(super) fn letter_symbol(&self, tonic: SpelledPitch) -> String {
        self.symbol(|degree, accidental| tonic.scale_degree(degree, accidental).engraved_name())
    }

    fn symbol(&self, name: impl Fn(&NumberedPitch, &Accidental) -> String) -> String {
        let triad = match self.triad {
            TriadQuality::Major => "",
            TriadQuality::Minor => "m",
            TriadQuality::Diminished => "°",
            TriadQuality::Augmented => "⁺",
            TriadQuality::Sus2 => "sus2",
            TriadQuality::Sus4 => "sus4",
        };
        let extension = match &self.extension {
            Some(Extension::DominantSeventh) => "⁷",
            Some(Extension::MajorSeventh) => "△⁷",
            None => "",
        };
        let bass = self
            .bass
            .as_ref()
            .map(|bass| format!("/{}", name(&bass.degree, &bass.accidental)))
            .unwrap_or_default();
        format!(
            "{}{triad}{extension}{bass}",
            name(&self.degree, &self.accidental)
        )
    }
}
