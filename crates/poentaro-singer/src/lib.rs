#![cfg_attr(test, allow(clippy::disallowed_macros))]
//! Sings a score's voiced lyric parts.
//!
//! The score side (`poentaro::sing`) turns a score into a [`SingingPlan`]:
//! plain notes with the syllable each sings. This crate turns a plan into
//! samples with the chimera voice ([`sing`], behind the `render` feature).
//! The two halves only share the plan, so the web app ships the voice as its
//! own wasm module, loaded only when a score sings.

use serde::{Deserialize, Serialize};

#[cfg(feature = "render")]
mod render;
#[cfg(feature = "render")]
pub use render::sing;

/// Everything to sing in one score, in playback order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SingingPlan {
    /// The rate [`sing`] renders at, to match the instruments it's mixed with.
    pub sample_rate: u32,
    pub parts: Vec<SungPart>,
}

/// One lyric part, sung by one voice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SungPart {
    pub voice: Voice,
    /// The part's peak, as a percentage of the instruments' peak.
    pub volume_percent: u8,
    pub notes: Vec<SungNote>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Voice {
    Male,
    Female,
    Child,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SungNote {
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub midi: u8,
    pub syllable: SungSyllable,
}

/// What a note sings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SungSyllable {
    /// A new syllable, as written in the lyric line.
    Word(String),
    /// The previous syllable's vowel carries on at this note's pitch: a held
    /// `-` in the lyrics, or a slur onto a different pitch.
    Held,
    /// Nothing to sing (a `_` measure, or fewer syllables than notes).
    Silent,
}

impl SingingPlan {
    /// The plan as bytes, to pass between wasm modules without either side
    /// in between having to understand it.
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|error| format!("invalid singing plan: {error}"))
    }
}
