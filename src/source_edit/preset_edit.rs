//! Edits to `preset "Name"` sections for the web editor's preset menu:
//! rewriting one preset's `visible_parts`, `soloed_parts` or `notation` line, adding a new
//! preset, and following a preset renamed in the editor. Each edit touches
//! only the lines it needs, so the rest of the file keeps its formatting. A
//! document in anyfix expressions goes through [`super::anyfix_edit`].

use itertools::Itertools;

use super::anyfix_edit;
use crate::ast::parsed::{Notation, PresetKey, DEFAULT_PRESET_NAME};

/// One line a preset menu control writes into a `preset` section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresetEdit {
    /// The parts shown and heard, in order; an empty list hides every part.
    VisibleParts(Vec<String>),
    /// The parts soloed, in order; an empty list removes the line.
    SoloedParts(Vec<String>),
    Notation(Notation),
}

impl PresetEdit {
    pub(super) fn key(&self) -> PresetKey {
        match self {
            Self::VisibleParts(_) => PresetKey::VisibleParts,
            Self::SoloedParts(_) => PresetKey::SoloedParts,
            Self::Notation(_) => PresetKey::Notation,
        }
    }

    pub(super) fn value(&self) -> String {
        match self {
            Self::VisibleParts(parts) | Self::SoloedParts(parts) => parts.join(", "),
            Self::Notation(notation) => notation.keyword().to_owned(),
        }
    }

    /// Whether writing this edit removes its line instead: only an empty
    /// `soloed_parts`, which solos nothing, the same as no line.
    pub(super) fn removes_line(&self) -> bool {
        matches!(self, Self::SoloedParts(parts) if parts.is_empty())
    }
}

/// A preset for "New preset…": the parts, solos and notation shown when it
/// was made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewPreset {
    pub name: String,
    pub visible_parts: Vec<String>,
    pub soloed_parts: Vec<String>,
    pub notation: Notation,
}

/// One entry of the preset menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresetSummary {
    pub name: String,
    pub notation: Option<Notation>,
}

/// The presets the menu lists, in file order. Read from the sections alone,
/// so a half-typed score doesn't empty the menu. Like the parser, it skips
/// `"Default"` and every repeat of an earlier name.
pub fn list_presets(source: &str) -> Vec<PresetSummary> {
    anyfix_edit::list_presets(source)
}

/// Why "New preset…" can't use `name`, or `None` when it can.
pub fn new_preset_name_error(source: &str, name: &str) -> Option<String> {
    let name = name.trim();
    if name.is_empty() {
        Some("Give the preset a name.".to_owned())
    } else if name.contains(['"', '\n', '\r']) {
        Some("A preset name can't contain quotes or line breaks.".to_owned())
    } else if name.eq_ignore_ascii_case(DEFAULT_PRESET_NAME) {
        Some(format!(
            "{DEFAULT_PRESET_NAME} is the song without a preset."
        ))
    } else if list_presets(source)
        .iter()
        .any(|preset| preset.name == name)
    {
        Some(format!("A preset is already called {name}."))
    } else {
        None
    }
}

/// The chosen preset's name after editing `previous_source` into `source`:
/// `name` while `source` still has it, else the preset now at its position
/// when the edit renamed it (same number of presets, and a name that is
/// new). `None` when `source` no longer has the preset.
pub fn follow_preset_rename(previous_source: &str, source: &str, name: &str) -> Option<String> {
    let names = |text| {
        list_presets(text)
            .into_iter()
            .map(|preset| preset.name)
            .collect_vec()
    };
    let current = names(source);
    if current.iter().any(|current_name| current_name == name) {
        return Some(name.to_owned());
    }
    let previous = names(previous_source);
    let position = previous
        .iter()
        .position(|previous_name| previous_name == name)?;
    current
        .get(position)
        .filter(|renamed| current.len() == previous.len() && !renamed.is_empty())
        .filter(|renamed| !previous.contains(renamed))
        .cloned()
}

/// Rewrites the preset's last `edit.key()` line (the one the parser keeps),
/// or adds the line under its header. An edit that [removes its
/// line](PresetEdit::removes_line) deletes every `edit.key()` line instead.
/// Returns `source` unchanged when no preset is called `name`.
pub fn update_preset_field(source: &str, name: &str, edit: &PresetEdit) -> String {
    anyfix_edit::update_preset_field(source, name, edit)
}

/// The named preset's part list under `key`: `None` when no preset has that
/// name, `Some(None)` when it sets none.
fn preset_part_list(source: &str, name: &str, key: PresetKey) -> Option<Option<Vec<String>>> {
    anyfix_edit::has_preset(source, name).then(|| anyfix_edit::part_list_of(source, name, key))
}

/// A part pill clicked under a preset: which part, and whether it now shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartShown {
    pub abbreviation: String,
    pub shown: bool,
}

/// Shows or hides one part in the preset's `visible_parts`, rewritten in
/// `all_parts` order. A preset without `visible_parts` starts from every
/// part. Returns `source` unchanged when no preset is called `name`.
pub fn set_preset_part_shown(
    source: &str,
    name: &str,
    all_parts: &[String],
    part: &PartShown,
) -> String {
    let Some(visible) = preset_part_list(source, name, PresetKey::VisibleParts) else {
        return source.to_owned();
    };
    let parts = all_parts
        .iter()
        .filter(|abbreviation| {
            if **abbreviation == part.abbreviation {
                part.shown
            } else {
                visible
                    .as_ref()
                    .is_none_or(|visible| visible.contains(abbreviation))
            }
        })
        .cloned()
        .collect();
    update_preset_field(source, name, &PresetEdit::VisibleParts(parts))
}

/// A solo button clicked under a preset: which part, and whether it is now
/// soloed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartSoloed {
    pub abbreviation: String,
    pub soloed: bool,
}

/// Solos or unsolos one part in the preset's `soloed_parts`, rewritten in
/// `all_parts` order; the line goes away once nothing is soloed. Returns
/// `source` unchanged when no preset is called `name`.
pub fn set_preset_part_soloed(
    source: &str,
    name: &str,
    all_parts: &[String],
    part: &PartSoloed,
) -> String {
    let Some(soloed) = preset_part_list(source, name, PresetKey::SoloedParts) else {
        return source.to_owned();
    };
    let soloed = soloed.unwrap_or_default();
    let parts = all_parts
        .iter()
        .filter(|abbreviation| {
            if **abbreviation == part.abbreviation {
                part.soloed
            } else {
                soloed.contains(abbreviation)
            }
        })
        .cloned()
        .collect();
    update_preset_field(source, name, &PresetEdit::SoloedParts(parts))
}

/// Adds a preset holding `preset`'s parts, solos and notation.
pub fn add_preset_section(source: &str, preset: NewPreset) -> String {
    anyfix_edit::add_preset_section(source, preset)
}

#[cfg(test)]
#[path = "preset_edit_tests.rs"]
mod preset_edit_tests;
