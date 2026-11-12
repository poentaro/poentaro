use super::{PartMode, PartSettings};
use crate::ast::parsed::RangeInstrument;

fn settings(mode: PartMode, program: Option<u8>, volume: u8, octave_offset: i8) -> PartSettings {
    PartSettings {
        mode,
        range_instrument: None,
        range_lowest: None,
        range_highest: None,
        program,
        volume,
        octave_offset,
    }
}

fn follow(target: &str) -> PartMode {
    PartMode::Follow {
        target: target.to_owned(),
    }
}

fn source_with_parts(parts_body: &str) -> String {
    format!("parts [\n  {parts_body},\n],\n\nscore \"\",\n")
}

fn update_part_declaration(
    source: &str,
    abbreviation: &str,
    settings: &PartSettings,
) -> Option<String> {
    super::update_part_declaration(source, abbreviation, settings)
}

fn assert_contains(result: &str, expected: &str) {
    assert!(
        result.contains(expected),
        "expected {expected:?} in:\n{result}"
    );
}

#[test]
fn test_basic_chord_to_notes() {
    let source = source_with_parts("\"main\" : chords");
    let result =
        update_part_declaration(&source, "main", &settings(PartMode::Notes, None, 100, 0)).unwrap();
    assert_contains(&result, "\"main\": notes");
}

#[test]
fn test_notes_to_follow() {
    let source = source_with_parts(r#""Alto" [A] : notes"#);
    let result =
        update_part_declaration(&source, "A", &settings(follow("M"), None, 100, 0)).unwrap();
    assert_contains(&result, r#""Alto" [A]: follow [M]"#);
}

#[test]
fn test_set_sound() {
    let source = source_with_parts(r#""Piano" [P] : notes"#);
    let result =
        update_part_declaration(&source, "P", &settings(PartMode::Notes, Some(0), 100, 0)).unwrap();
    assert_contains(&result, r#""Piano" [P]: notes { sound 0 }"#);
}

#[test]
fn test_change_sound() {
    let source = source_with_parts(r#""Piano" [P] : notes { sound : 0 }"#);
    let result =
        update_part_declaration(&source, "P", &settings(PartMode::Notes, Some(52), 100, 0))
            .unwrap();
    assert_contains(&result, r#""Piano" [P]: notes { sound 52 }"#);
}

#[test]
fn test_remove_sound() {
    let source = source_with_parts(r#""Piano" [P] : notes { sound : 0 }"#);
    let result =
        update_part_declaration(&source, "P", &settings(PartMode::Notes, None, 100, 0)).unwrap();
    assert_contains(&result, r#""Piano" [P]: notes]"#);
}

#[test]
fn test_mode_change_preserves_sound() {
    let source = source_with_parts(r#""Piano" [P] : notes { sound : 0 }"#);
    let result =
        update_part_declaration(&source, "P", &settings(PartMode::Chords, Some(0), 100, 0))
            .unwrap();
    assert_contains(&result, r#""Piano" [P]: chords { sound 0 }"#);
}

#[test]
fn test_no_match_returns_none() {
    let source = source_with_parts("\"main\" : notes");
    let result = update_part_declaration(
        &source,
        "NOMATCH",
        &settings(PartMode::Chords, None, 100, 0),
    );
    assert!(result.is_none());
}

#[test]
fn test_multi_part_only_target_changes() {
    let source = source_with_parts("\"Melody\" [M] : notes,\n  \"Alto\" [A] : notes");
    let result =
        update_part_declaration(&source, "A", &settings(PartMode::Chords, None, 100, 0)).unwrap();
    assert_contains(&result, r#""Melody" [M]: notes,"#);
    assert_contains(&result, r#""Alto" [A]: chords"#);
}

#[test]
fn test_set_octave_on_notes() {
    let source = source_with_parts(r#""Melody" [M] : notes"#);
    let result =
        update_part_declaration(&source, "M", &settings(PartMode::Notes, None, 100, -1)).unwrap();
    assert_contains(&result, r#""Melody" [M]: notes { octave -1 }"#);
}

#[test]
fn test_set_octave_on_follow() {
    let source = source_with_parts("\"Melody\" [M] : notes,\n  \"Chords\" [C] : follow [M]");
    let result =
        update_part_declaration(&source, "C", &settings(follow("M"), None, 100, 1)).unwrap();
    assert_contains(&result, r#""Chords" [C]: follow [M] { octave: + 1 }"#);
}

#[test]
fn test_change_sound_on_follow_preserves_mode_and_target() {
    let source = source_with_parts(
        "\"Melody\" [M] : notes,\n  \"Chords\" [C] : follow [M] { volume : 60% }",
    );
    let result =
        update_part_declaration(&source, "C", &settings(follow("M"), Some(40), 60, 0)).unwrap();
    assert_contains(
        &result,
        r#""Chords" [C]: follow [M] { sound 40, volume: 60 % }"#,
    );
}

#[test]
fn test_default_volume_and_octave_are_omitted() {
    let source = source_with_parts(r#""Melody" [M] : notes { volume : 60%, octave : +1 }"#);
    let result =
        update_part_declaration(&source, "M", &settings(PartMode::Notes, None, 100, 0)).unwrap();
    assert_contains(&result, r#""Melody" [M]: notes]"#);
}

#[test]
fn test_out_of_range_volume_and_octave_are_written_clamped() {
    let source = source_with_parts(r#""Melody" [M] : notes"#);
    let result =
        update_part_declaration(&source, "M", &settings(PartMode::Notes, None, 150, -7)).unwrap();
    assert_contains(&result, r#""Melody" [M]: notes { octave -4 }"#);
}

#[test]
fn test_set_percussion_key() {
    let source = source_with_parts(r#""Drums" [D] : percussion"#);
    let result = update_part_declaration(
        &source,
        "D",
        &settings(PartMode::Percussion, Some(38), 100, 0),
    )
    .unwrap();
    assert_contains(&result, r#""Drums" [D]: percussion { sound 38 }"#);
}

#[test]
fn writes_the_instrument_before_the_sound() {
    let source = source_with_parts(r#""Violin" [V] : notes"#);
    let result = update_part_declaration(
        &source,
        "V",
        &PartSettings {
            range_instrument: Some(RangeInstrument::Violin),
            ..settings(PartMode::Notes, Some(40), 80, 0)
        },
    )
    .unwrap();
    assert_contains(
        &result,
        r#""Violin" [V]: notes { instrument: violin, sound 40, volume: 80 % }"#,
    );
}

#[test]
fn clearing_the_instrument_drops_it_from_the_part() {
    let source = source_with_parts(r#""Soprano" [S] : notes { instrument : soprano }"#);
    let result =
        update_part_declaration(&source, "S", &settings(PartMode::Notes, None, 100, 0)).unwrap();
    assert_contains(&result, r#""Soprano" [S]: notes]"#);
}

fn with_range(settings: PartSettings, lowest: Option<u8>, highest: Option<u8>) -> PartSettings {
    PartSettings {
        range_lowest: lowest,
        range_highest: highest,
        ..settings
    }
}

#[test]
fn writes_the_range_after_the_instrument_quoting_sharps() {
    let source = source_with_parts(r#""Tenor" [T] : notes"#);
    let settings = PartSettings {
        range_instrument: Some(RangeInstrument::Tenor),
        ..settings(PartMode::Notes, Some(52), 100, 0)
    };
    let result =
        update_part_declaration(&source, "T", &with_range(settings, Some(48), Some(66))).unwrap();
    assert_contains(
        &result,
        r#""Tenor" [T]: notes { instrument: tenor, range: C3 to "F#4", sound 52 }"#,
    );
}

#[test]
fn a_picked_bound_takes_the_other_from_the_instrument() {
    let source = source_with_parts(r#""Tenor" [T] : notes { instrument : tenor }"#);
    let settings = PartSettings {
        range_instrument: Some(RangeInstrument::Tenor),
        ..settings(PartMode::Notes, None, 100, 0)
    };
    let result =
        update_part_declaration(&source, "T", &with_range(settings, None, Some(65))).unwrap();
    assert_contains(
        &result,
        r#""Tenor" [T]: notes { instrument: tenor, range: C3 to F4 }"#,
    );
}

#[test]
fn a_range_read_back_is_written_unchanged() {
    let source = source_with_parts(r#""Erhu" [E] : notes { range : D4 to A6 }"#);
    let declaration = crate::list_part_declarations_from_source(&source, "test.poentaro", &[])
        .unwrap()
        .remove(0);
    assert_eq!(
        (
            declaration.settings.range_lowest,
            declaration.settings.range_highest
        ),
        (Some(62), Some(93))
    );
    let result = update_part_declaration(&source, "E", &declaration.settings).unwrap();
    assert_contains(&result, r#""Erhu" [E]: notes { range: D4 to A6 }"#);
}

#[test]
fn switching_to_chords_drops_the_written_range() {
    let source = source_with_parts(r#""Tenor" [T] : notes { range : C3 to G4 }"#);
    let settings = with_range(settings(PartMode::Chords, None, 100, 0), Some(48), Some(67));
    let result = update_part_declaration(&source, "T", &settings).unwrap();
    assert_contains(&result, r#""Tenor" [T]: chords]"#);
}

#[test]
fn default_range_labels_name_the_instruments_bounds() {
    let source = source_with_parts(
        r#""Tenor" [T] : notes { instrument : tenor }, "Copy" [C] : follow [T], "Free" [F] : notes"#,
    );
    let labels = crate::list_part_declarations_from_source(&source, "test.poentaro", &[])
        .unwrap()
        .into_iter()
        .map(|declaration| declaration.default_range_labels.lowest)
        .collect::<Vec<_>>();
    assert_eq!(labels, ["C3 (tenor)", "as target", "none"]);
}
