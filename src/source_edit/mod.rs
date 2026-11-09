mod anyfix_edit;
mod anyfix_part_edit;
mod metadata_edit;
mod octave_shift;
mod preset_edit;
mod range_edit;
mod slur_toggle;
mod tie_toggle;
pub use metadata_edit::{
    parse_metadata_fields, update_metadata_field, MetadataEdit, MetadataFields,
};
pub use octave_shift::{shift_part_octave, shift_range_octave};
pub use preset_edit::{
    add_preset_section, follow_preset_rename, list_presets, new_preset_name_error,
    set_preset_part_shown, set_preset_part_soloed, update_preset_field, NewPreset, PartShown,
    PartSoloed, PresetEdit, PresetSummary,
};
pub use range_edit::{ByteRange, RangeEditResult};
pub use slur_toggle::toggle_range_slur;
pub use tie_toggle::toggle_range_tie;

use crate::ast::parsed::RangeInstrument;
use crate::parser::parts_parser::SourcePartMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartMode {
    Chords,
    Notes,
    Percussion,
    Follow { target: String },
    Lyrics { target: String },
}

impl PartMode {
    /// Builds a `PartMode` from the parser's [`SourcePartMode`] tag.
    /// `follow_target` is only consulted for `SourcePartMode::Follow`.
    pub fn from_source_mode(kind: SourcePartMode, follow_target: Option<String>) -> Self {
        match kind {
            SourcePartMode::Chords => Self::Chords,
            SourcePartMode::Notes => Self::Notes,
            SourcePartMode::Percussion => Self::Percussion,
            SourcePartMode::Follow => Self::Follow {
                target: follow_target.unwrap_or_default(),
            },
            SourcePartMode::Lyrics => Self::Lyrics {
                target: follow_target.unwrap_or_default(),
            },
        }
    }
}

/// A part's settings as its `parts` line writes them, with an omitted
/// volume/octave suffix read as its default. Round-trips through
/// [`update_part_declaration`], which omits a default-valued suffix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartSettings {
    pub mode: PartMode,
    /// The instrument whose range the part's notes are checked against;
    /// `None` when the line names none (a follow part then inherits its
    /// target's).
    pub range_instrument: Option<RangeInstrument>,
    /// The sound's GM program number (a GM percussion key on a percussion
    /// part); `None` when the line names no sound.
    pub program: Option<u8>,
    pub volume: u8,
    pub octave_offset: i8,
}

/// Rewrites the part declaring `abbreviation` to `settings`, with the volume and
/// octave offset clamped into [`crate::parser::parts_parser::PART_SETTING_LIMITS`];
/// `None` when there's no such part.
pub fn update_part_declaration(
    source: &str,
    abbreviation: &str,
    settings: &PartSettings,
) -> Option<String> {
    anyfix_part_edit::update_part_declaration(source, abbreviation, settings)
}

#[cfg(test)]
mod tests;
