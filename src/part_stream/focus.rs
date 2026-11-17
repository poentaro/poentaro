//! What the part focus editor shows around each measure of the focused
//! part's stream: the other lines its measure group writes, as read-only
//! **ghost lines**, and its section label.

use crate::filters::{resolve_part_visibility, PartToggleState};
use crate::part_info::PartInfo;

use super::document::{GroupLine, ScoreDocument};
use super::editable::editable_part;
use super::{find_part, PartStreamError};

/// One measure of the score, around the focused part's line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusMeasure {
    pub section_label: Option<String>,
    /// Directive and comment lines, and parts declared before the focused one.
    pub lines_above: Vec<String>,
    /// Parts declared after the focused one.
    pub lines_below: Vec<String>,
}

/// The ghost lines of every measure (index = stream measure index), as the
/// source writes them: a measure that doesn't write a part has no line for
/// it. Only parts the toggles draw are kept.
pub fn part_focus_measures(
    source: &str,
    abbreviation: &str,
    toggles: &PartToggleState,
) -> Result<Vec<FocusMeasure>, PartStreamError> {
    let document = ScoreDocument::load(source)?;
    let focused = find_part(&document, abbreviation)?;
    editable_part(&document, focused)?;
    let parts: Vec<PartInfo> = document
        .declarations
        .iter()
        .map(|declaration| PartInfo {
            abbreviation: declaration.abbreviation.clone(),
            display_name: declaration.display_name.clone(),
            sounds: declaration.kind.sounds(),
        })
        .collect();
    let rendered = resolve_part_visibility(&parts, toggles, None).rendered_tracks;
    let is_drawn = |part_index: usize| {
        rendered.as_ref().is_none_or(|tracks| {
            parts
                .get(part_index)
                .is_some_and(|part| tracks.contains(&part.abbreviation))
        })
    };
    Ok(document
        .measures
        .iter()
        .map(|measure| {
            let (lines_above, lines_below) = measure
                .lines
                .iter()
                .filter(|line| line.part_index != Some(focused))
                .filter(|line| line.part_index.is_none_or(is_drawn))
                .partition::<Vec<_>, _>(|line| line.part_index.is_none_or(|index| index < focused));
            let texts =
                |lines: Vec<&GroupLine>| lines.iter().map(|line| line.text.clone()).collect();
            FocusMeasure {
                section_label: measure.section_label.clone(),
                lines_above: texts(lines_above),
                lines_below: texts(lines_below),
            }
        })
        .collect())
}

#[cfg(test)]
#[path = "focus_tests.rs"]
mod tests;
