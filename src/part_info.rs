use crate::ast::parsed::midi_note_name;
use crate::error::IrrecoverableError;
use crate::parser::parts_parser::{
    InstrumentInfo, SourcePartMode, SourceRawPartDecl, DEFAULT_PART_OCTAVE_OFFSET,
    DEFAULT_PART_VOLUME,
};
use crate::source_edit::{PartMode, PartSettings};

/// A part declared in the `parts` section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartInfo {
    /// Abbreviation used in score row labels and `--tracks` filtering.
    pub abbreviation: String,
    /// Full display name from the declaration left-hand side.
    pub display_name: String,
    /// Whether the part makes sound. False for a lyric part, which only
    /// sings along to its target's notes.
    pub sounds: bool,
}

/// Source-level part declaration for the Edit Parts modal (before follow inheritance).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePartDeclaration {
    pub abbreviation: String,
    pub display_name: String,
    pub line_number: u32,
    pub settings: PartSettings,
    /// Display label for the sound the part plays (see [`crate::sound_label`]):
    /// `settings.program`, or else the range instrument's default sound.
    pub sound_label: Option<String>,
    /// What the range pickers show when the part writes no range.
    pub default_range_labels: RangeBoundLabels,
}

/// The range pickers' labels for a part that writes no range: its
/// instrument's bounds (`C3 (tenor)`), `as target` on a follow part naming
/// no instrument, or `none`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeBoundLabels {
    pub lowest: String,
    pub highest: String,
}

fn default_range_labels(raw: &SourceRawPartDecl) -> RangeBoundLabels {
    match (raw.range_instrument, &raw.mode) {
        (Some(instrument), _) => {
            let range = instrument.range();
            let label = |midi| format!("{} ({})", midi_note_name(midi), instrument.keyword());
            RangeBoundLabels {
                lowest: label(range.lowest),
                highest: label(range.highest),
            }
        }
        (None, SourcePartMode::Follow) => RangeBoundLabels {
            lowest: "as target".to_string(),
            highest: "as target".to_string(),
        },
        (None, _) => RangeBoundLabels {
            lowest: "none".to_string(),
            highest: "none".to_string(),
        },
    }
}

fn map_raw_to_source_declaration(
    raw: SourceRawPartDecl,
    instruments: &[InstrumentInfo],
) -> SourcePartDeclaration {
    let program = raw.soundfont.map(|soundfont| soundfont.0);
    let is_percussion = matches!(raw.mode, SourcePartMode::Percussion);
    // A line that quotes no sound but names an instrument plays that
    // instrument's sound; label it so the modal shows what will play.
    let sounding_program = program.or(raw
        .range_instrument
        .map(|instrument| instrument.default_soundfont().0));
    let sound_label = sounding_program
        .map(|program| crate::sound_label::sound_label(program, is_percussion, instruments));
    let default_range_labels = default_range_labels(&raw);
    SourcePartDeclaration {
        abbreviation: raw.abbreviation,
        display_name: raw.display_name,
        line_number: raw.line_number,
        settings: PartSettings {
            mode: PartMode::from_source_mode(raw.mode, raw.follow_target),
            range_instrument: raw.range_instrument,
            range_lowest: raw.written_range.map(|range| range.lowest),
            range_highest: raw.written_range.map(|range| range.highest),
            program,
            volume: raw.volume.unwrap_or(DEFAULT_PART_VOLUME),
            octave_offset: raw.octave_offset.unwrap_or(DEFAULT_PART_OCTAVE_OFFSET),
        },
        sound_label,
        default_range_labels,
    }
}

/// List source-level part declarations from a `.poentaro` source string.
///
/// Returns what is written on each `parts` line, without follow inheritance.
pub fn list_part_declarations_from_source(
    source: &str,
    _filename: &str,
    instruments: &[InstrumentInfo],
) -> Result<Vec<SourcePartDeclaration>, IrrecoverableError> {
    let mut errors = Vec::new();
    let raw_declarations =
        crate::parser::anyx::source_part_declarations(source, &mut errors, instruments);

    // Lyric parts have no sound, volume or octave to edit.
    Ok(raw_declarations
        .into_iter()
        .filter(|raw| raw.mode != SourcePartMode::Lyrics)
        .map(|raw| map_raw_to_source_declaration(raw, instruments))
        .collect())
}

/// List part declarations from a `.poentaro` source string.
///
/// Lyric parts are listed like any other part, right after the part they sing
/// along to. A part's alternative lines (`S.2`) aren't listed: they show and
/// hide with the part.
pub fn list_parts_from_source(
    source: &str,
    filename: &str,
    instruments: &[InstrumentInfo],
) -> Result<Vec<PartInfo>, IrrecoverableError> {
    let doc = crate::parser::parse(source, filename, instruments)?;
    Ok(doc
        .declarations
        .into_iter()
        .filter(|declaration| declaration.alternative_line.is_none())
        .map(|declaration| PartInfo {
            abbreviation: declaration.abbreviation,
            display_name: declaration.display_name,
            sounds: declaration.kind.sounds(),
        })
        .collect())
}
