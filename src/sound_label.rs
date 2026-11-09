//! The `"N: Name"` label a `parts` line quotes to pick a part's sound
//! (e.g. `"40: Violin"`, or `"38: Acoustic Snare"` on a percussion part).
//! This module is the only place that formats or parses that text: callers
//! across the wasm boundary exchange the bare program/key number and get
//! display labels from here.

use crate::parser::parts_parser::InstrumentInfo;

#[derive(serde::Deserialize)]
struct GmPercussionEntry {
    key: u8,
    name: String,
}

/// GM percussion key names (keys 35-81), per the General MIDI percussion map.
static GM_PERCUSSION_NAMES: std::sync::LazyLock<Vec<GmPercussionEntry>> =
    std::sync::LazyLock::new(|| {
        serde_json::from_str(include_str!("gm_percussion.json")).unwrap_or_default()
    });

/// One pickable sound: its GM program number (or percussion key) and the
/// label the source quotes for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundChoice {
    pub program: u8,
    pub label: String,
}

/// Every sound the Edit Parts sound picker offers, per part kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundChoices {
    /// One per catalog instrument, in catalog order.
    pub instruments: Vec<SoundChoice>,
    /// One per named GM percussion key, in key order.
    pub percussion: Vec<SoundChoice>,
}

fn format_label(program: u8, name: &str) -> String {
    format!("{program}: {name}")
}

pub(crate) fn instrument_label(instrument: &InstrumentInfo) -> String {
    format_label(instrument.program, &instrument.name)
}

fn unknown_label(program: u8) -> String {
    format_label(program, "Unknown")
}

/// The label for `program`: a GM percussion key name when `is_percussion`,
/// otherwise the matching `instruments` entry's name (`"N: Unknown"` when
/// there's no such entry).
pub(crate) fn sound_label(
    program: u8,
    is_percussion: bool,
    instruments: &[InstrumentInfo],
) -> String {
    let name = if is_percussion {
        GM_PERCUSSION_NAMES
            .iter()
            .find(|entry| entry.key == program)
            .map(|entry| entry.name.as_str())
    } else {
        instruments
            .iter()
            .find(|instrument| instrument.program == program)
            .map(|instrument| instrument.name.as_str())
    };
    name.map_or_else(
        || unknown_label(program),
        |name| format_label(program, name),
    )
}

/// The program number a quoted sound label starts with (`"40: Violin"` -> 40);
/// `None` when `label` isn't in `"N: Name"` form.
pub(crate) fn parse_label_program(label: &str) -> Option<u8> {
    let (number, _) = label.split_once(": ")?;
    number.trim().parse().ok()
}

/// Every sound the Edit Parts sound picker offers, labelled.
pub fn list_sound_choices(instruments: &[InstrumentInfo]) -> SoundChoices {
    SoundChoices {
        instruments: instruments
            .iter()
            .map(|instrument| SoundChoice {
                program: instrument.program,
                label: instrument_label(instrument),
            })
            .collect(),
        percussion: GM_PERCUSSION_NAMES
            .iter()
            .map(|entry| SoundChoice {
                program: entry.key,
                label: format_label(entry.key, &entry.name),
            })
            .collect(),
    }
}

#[cfg(test)]
#[path = "sound_label_tests.rs"]
mod tests;
