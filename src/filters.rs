use crate::ast::grouped::Score;

/// Retain only parts whose names appear in `enabled_tracks`.
///
/// `None` keeps every part. `Some([])` removes every part.
pub fn apply_track_filter(score: &mut Score, enabled_tracks: Option<&[String]>) {
    let Some(tracks) = enabled_tracks else {
        return;
    };
    for measure in &mut score.measures {
        measure.parts.retain(|part| {
            part.name()
                .as_ref()
                .is_some_and(|name| tracks.contains(name))
        });
    }
}

/// Retain only parts whose names appear in `tracks`. No-op when `tracks` is empty.
pub fn filter_tracks(score: &mut Score, tracks: &[String]) {
    if tracks.is_empty() {
        return;
    }
    apply_track_filter(score, Some(tracks));
}

/// The part-toggle UI's raw show/hide/solo state, as abbreviations. Every part
/// — notes, chords, percussion or lyrics — is toggled the same way;
/// [`resolve_part_visibility`] turns this into what is drawn and what sounds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PartToggleState {
    pub hidden_parts: Vec<String>,
    pub soloed_parts: Vec<String>,
}

/// The `preset` chosen in the preset menu, as far as part visibility and
/// rendering need it (see [`crate::chosen_preset_from_source`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChosenPreset {
    pub name: String,
    /// The preset's `visible_parts`; `None` when it leaves every part shown.
    pub visible_parts: Option<Vec<String>>,
    /// The preset's `soloed_parts`, added to the toggles' own solos.
    pub soloed_parts: Vec<String>,
}

impl ChosenPreset {
    /// The parts an export under this preset keeps: its `soloed_parts` when
    /// it has any, else its `visible_parts`. Empty means every part.
    pub fn exported_parts(self) -> Vec<String> {
        if self.soloed_parts.is_empty() {
            self.visible_parts.unwrap_or_default()
        } else {
            self.soloed_parts
        }
    }
}

/// What [`resolve_part_visibility`] decided for a score's parts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedPartVisibility {
    /// Parts drawn, for [`apply_track_filter`] and the part-list legend.
    /// `None` when every part is drawn.
    pub rendered_tracks: Option<Vec<String>>,
    /// Parts that sound in playback. `None` when every part sounds.
    pub sounding_tracks: Option<Vec<String>>,
    /// Display names of `sounding_tracks`, for export filenames.
    pub sounding_display_names: Option<Vec<String>>,
    /// The chosen preset, whose `metadata` overrides the render applies.
    /// `None` renders the song's own `metadata`.
    pub preset_name: Option<String>,
    /// Parts whose pill is off, in `parts` order: hidden ones and those the
    /// preset's `visible_parts` leaves out. Solo doesn't change it.
    pub hidden_parts: Vec<String>,
    /// Parts whose solo button is on, in `parts` order: soloed in the
    /// toggles or in the preset's `soloed_parts`.
    pub soloed_parts: Vec<String>,
}

/// Resolves the raw toggle state against `parts`, under the chosen preset.
///
/// A part shows when the preset's `visible_parts` lists it (or the preset sets
/// none) and it isn't hidden. Solo wins over both: once any part is soloed,
/// in the toggles or in the preset's `soloed_parts`, only soloed parts show.
/// Solo entries naming no part are ignored.
pub fn resolve_part_visibility(
    parts: &[crate::part_info::PartInfo],
    state: &PartToggleState,
    preset: Option<&ChosenPreset>,
) -> ResolvedPartVisibility {
    let contains =
        |names: &[String], part: &crate::part_info::PartInfo| names.contains(&part.abbreviation);
    let is_hidden = |part: &crate::part_info::PartInfo| {
        contains(&state.hidden_parts, part)
            || preset
                .and_then(|preset| preset.visible_parts.as_deref())
                .is_some_and(|visible| !contains(visible, part))
    };
    let is_soloed = |part: &crate::part_info::PartInfo| {
        contains(&state.soloed_parts, part)
            || preset.is_some_and(|preset| contains(&preset.soloed_parts, part))
    };
    let soloed_parts: Vec<String> = parts
        .iter()
        .filter(|part| is_soloed(part))
        .map(|part| part.abbreviation.clone())
        .collect();
    let any_solo = !soloed_parts.is_empty();
    let shown: Vec<_> = parts
        .iter()
        .filter(|part| {
            if any_solo {
                is_soloed(part)
            } else {
                !is_hidden(part)
            }
        })
        .collect();
    let or_none_when_all =
        |map: fn(&crate::part_info::PartInfo) -> String,
         of_kind: fn(&crate::part_info::PartInfo) -> bool| {
            let all = parts.iter().filter(|part| of_kind(part)).count();
            let kept: Vec<String> = shown
                .iter()
                .filter(|part| of_kind(part))
                .map(|part| map(part))
                .collect();
            (kept.len() != all).then_some(kept)
        };

    ResolvedPartVisibility {
        rendered_tracks: or_none_when_all(|part| part.abbreviation.clone(), |_| true),
        sounding_tracks: or_none_when_all(|part| part.abbreviation.clone(), |part| part.sounds),
        sounding_display_names: or_none_when_all(
            |part| part.display_name.clone(),
            |part| part.sounds,
        ),
        preset_name: preset.map(|preset| preset.name.clone()),
        hidden_parts: parts
            .iter()
            .filter(|part| is_hidden(part))
            .map(|part| part.abbreviation.clone())
            .collect(),
        soloed_parts,
    }
}

#[cfg(test)]
#[path = "filters_tests.rs"]
mod tests;
