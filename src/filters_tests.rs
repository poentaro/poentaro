use super::*;
use crate::part_info::PartInfo;

fn part(abbreviation: &str, display_name: &str) -> PartInfo {
    PartInfo {
        abbreviation: abbreviation.to_string(),
        display_name: display_name.to_string(),
        sounds: true,
    }
}

fn lyric_part(abbreviation: &str, display_name: &str) -> PartInfo {
    PartInfo {
        sounds: false,
        ..part(abbreviation, display_name)
    }
}

fn names(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}

fn melody_verse_and_chords() -> Vec<PartInfo> {
    vec![
        part("M", "Melody"),
        lyric_part("v1", "Verse 1"),
        part("C", "Chords"),
    ]
}

#[test]
fn default_state_shows_everything() {
    let resolved = resolve_part_visibility(
        &melody_verse_and_chords(),
        &PartToggleState::default(),
        None,
    );

    assert_eq!(resolved, ResolvedPartVisibility::default());
}

#[test]
fn hiding_a_part_removes_only_that_part() {
    let state = PartToggleState {
        hidden_parts: names(&["M"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_verse_and_chords(), &state, None);

    assert_eq!(resolved.rendered_tracks, Some(names(&["v1", "C"])));
    assert_eq!(resolved.sounding_tracks, Some(names(&["C"])));
    assert_eq!(resolved.sounding_display_names, Some(names(&["Chords"])));
}

#[test]
fn hiding_a_lyric_part_is_the_same_as_hiding_any_other_part() {
    let state = PartToggleState {
        hidden_parts: names(&["v1"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_verse_and_chords(), &state, None);

    assert_eq!(resolved.rendered_tracks, Some(names(&["M", "C"])));
}

#[test]
fn a_lyric_part_never_counts_as_sounding() {
    let hidden = PartToggleState {
        hidden_parts: names(&["v1"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_verse_and_chords(), &hidden, None);

    assert_eq!(resolved.sounding_tracks, None);
    assert_eq!(resolved.sounding_display_names, None);
}

#[test]
fn soloing_a_part_shows_only_soloed_parts() {
    let state = PartToggleState {
        soloed_parts: names(&["v1", "C"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_verse_and_chords(), &state, None);

    assert_eq!(resolved.rendered_tracks, Some(names(&["v1", "C"])));
}

#[test]
fn solo_wins_over_hide() {
    let state = PartToggleState {
        hidden_parts: names(&["C"]),
        soloed_parts: names(&["C"]),
    };

    let resolved = resolve_part_visibility(&melody_verse_and_chords(), &state, None);

    assert_eq!(resolved.rendered_tracks, Some(names(&["C"])));
}

#[test]
fn soloing_an_unknown_part_is_ignored() {
    let state = PartToggleState {
        soloed_parts: names(&["nope"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(&melody_verse_and_chords(), &state, None);

    assert_eq!(resolved, ResolvedPartVisibility::default());
}

fn preset(visible_parts: Option<&[&str]>) -> ChosenPreset {
    ChosenPreset {
        name: "Choir".to_string(),
        visible_parts: visible_parts.map(names),
        soloed_parts: Vec::new(),
    }
}

#[test]
fn preset_shows_only_its_visible_parts_and_silences_the_rest() {
    let resolved = resolve_part_visibility(
        &melody_verse_and_chords(),
        &PartToggleState::default(),
        Some(&preset(Some(&["M", "v1"]))),
    );

    assert_eq!(resolved.rendered_tracks, Some(names(&["M", "v1"])));
    assert_eq!(resolved.sounding_tracks, Some(names(&["M"])));
    assert_eq!(resolved.preset_name.as_deref(), Some("Choir"));
}

#[test]
fn preset_without_visible_parts_shows_everything() {
    let resolved = resolve_part_visibility(
        &melody_verse_and_chords(),
        &PartToggleState::default(),
        Some(&preset(None)),
    );

    assert_eq!(resolved.rendered_tracks, None);
    assert_eq!(resolved.preset_name.as_deref(), Some("Choir"));
}

#[test]
fn hiding_narrows_the_preset_parts() {
    let state = PartToggleState {
        hidden_parts: names(&["v1"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(
        &melody_verse_and_chords(),
        &state,
        Some(&preset(Some(&["M", "v1"]))),
    );

    assert_eq!(resolved.rendered_tracks, Some(names(&["M"])));
    assert_eq!(resolved.hidden_parts, names(&["v1", "C"]));
}

#[test]
fn hidden_parts_are_the_toggle_state_without_a_preset_and_ignore_solo() {
    let state = PartToggleState {
        hidden_parts: names(&["v1", "gone"]),
        soloed_parts: names(&["M"]),
    };

    let resolved = resolve_part_visibility(&melody_verse_and_chords(), &state, None);

    assert_eq!(resolved.hidden_parts, names(&["v1"]));
}

#[test]
fn solo_wins_over_the_preset() {
    let state = PartToggleState {
        soloed_parts: names(&["C"]),
        ..Default::default()
    };

    let resolved = resolve_part_visibility(
        &melody_verse_and_chords(),
        &state,
        Some(&preset(Some(&["M"]))),
    );

    assert_eq!(resolved.rendered_tracks, Some(names(&["C"])));
}

#[test]
fn preset_solo_shows_only_its_soloed_parts() {
    let chosen = ChosenPreset {
        soloed_parts: names(&["C"]),
        ..preset(Some(&["M", "v1"]))
    };

    let resolved = resolve_part_visibility(
        &melody_verse_and_chords(),
        &PartToggleState::default(),
        Some(&chosen),
    );

    assert_eq!(resolved.rendered_tracks, Some(names(&["C"])));
    assert_eq!(resolved.soloed_parts, names(&["C"]));
    assert_eq!(resolved.hidden_parts, names(&["C"]));
}

#[test]
fn soloed_parts_join_the_toggles_and_the_preset_in_parts_order() {
    let state = PartToggleState {
        soloed_parts: names(&["C", "gone"]),
        ..Default::default()
    };
    let chosen = ChosenPreset {
        soloed_parts: names(&["M"]),
        ..preset(None)
    };

    let resolved = resolve_part_visibility(&melody_verse_and_chords(), &state, Some(&chosen));

    assert_eq!(resolved.soloed_parts, names(&["M", "C"]));
    assert_eq!(resolved.rendered_tracks, Some(names(&["M", "C"])));
}

#[test]
fn a_preset_exports_its_soloed_parts_over_its_visible_parts() {
    let chosen = ChosenPreset {
        soloed_parts: names(&["C"]),
        ..preset(Some(&["M"]))
    };

    assert_eq!(chosen.exported_parts(), names(&["C"]));
    assert_eq!(preset(Some(&["M"])).exported_parts(), names(&["M"]));
    assert!(preset(None).exported_parts().is_empty());
}
