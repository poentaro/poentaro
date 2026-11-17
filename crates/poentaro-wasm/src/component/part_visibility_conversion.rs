use super::*;

pub(super) fn part_info_from_wit(part: Part) -> poentaro::PartInfo {
    poentaro::PartInfo {
        abbreviation: part.abbreviation,
        display_name: part.display_name,
        sounds: part.sounds,
    }
}

pub(super) fn part_toggle_state_from_wit(state: PartToggleState) -> poentaro::PartToggleState {
    poentaro::PartToggleState {
        hidden_parts: state.hidden_parts,
        soloed_parts: state.soloed_parts,
    }
}

pub(super) fn resolved_part_visibility_from_wit(
    visibility: ResolvedPartVisibility,
) -> poentaro::ResolvedPartVisibility {
    poentaro::ResolvedPartVisibility {
        rendered_tracks: visibility.rendered_tracks,
        sounding_tracks: visibility.sounding_tracks,
        sounding_display_names: visibility.sounding_display_names,
        preset_name: visibility.preset_name,
        hidden_parts: visibility.hidden_parts,
        soloed_parts: visibility.soloed_parts,
    }
}

pub(super) fn resolved_part_visibility_to_wit(
    visibility: poentaro::ResolvedPartVisibility,
) -> ResolvedPartVisibility {
    ResolvedPartVisibility {
        rendered_tracks: visibility.rendered_tracks,
        sounding_tracks: visibility.sounding_tracks,
        sounding_display_names: visibility.sounding_display_names,
        preset_name: visibility.preset_name,
        hidden_parts: visibility.hidden_parts,
        soloed_parts: visibility.soloed_parts,
    }
}
