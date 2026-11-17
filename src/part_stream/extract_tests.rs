use crate::part_stream::{
    extract_part_stream, list_streamable_parts, PartStreamError, StreamablePartKind,
};

const TWO_PARTS: &str = r#"parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    [M] 1 2
    [B] 1 1 1 1

    [B] 1 1 1 1

    time=3/4
    [M]   5   6 7
    [B] 1 1 1

    [M] 1 - -
    [B] 1 1 1

    [M] 2 - -
    [B] 1 1 1
    """,
"#;

#[test]
fn materializes_implicit_padding_and_rest_fills_unwritten_measures() {
    let stream = extract_part_stream(TWO_PARTS, "M").unwrap();
    assert_eq!(stream.text, "1 2 - -\n0 0 0 0\n5 6 7\n1 - -\n2 - -");
    assert_eq!(stream.measure_starts, vec![0, 8, 16, 22, 28]);
}

#[test]
fn rest_fills_a_compound_meter_by_its_beat_capacity() {
    let source = r#"parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    time=6/8
    [B] 1_1_1_1_1_1_
    """,
"#;
    assert_eq!(extract_part_stream(source, "M").unwrap().text, "0 0 0");
}

#[test]
fn unknown_part_is_an_error() {
    assert_eq!(
        extract_part_stream(TWO_PARTS, "X"),
        Err(PartStreamError::UnknownPart {
            abbreviation: "X".to_string()
        })
    );
}

#[test]
fn beat_overflow_is_kept_as_written() {
    let source = r#"parts ["Melody" [M]: notes],

score """
    [M] 1 2 3 4   5

    [M] 6 7 1 2
    """,
"#;
    assert_eq!(list_streamable_parts(source)[0].ineligible_reason, None);
    assert_eq!(
        extract_part_stream(source, "M").unwrap().text,
        "1 2 3 4 5\n6 7 1 2"
    );
}

#[test]
fn unpaddable_beat_underflow_is_kept_as_written() {
    let source = r#"parts ["Melody" [M]: notes],

score """
    [M] 1 2 3_

    [M] 4_ 5 6 7
    """,
"#;
    assert_eq!(list_streamable_parts(source)[0].ineligible_reason, None);
    assert_eq!(
        extract_part_stream(source, "M").unwrap().text,
        "1 2 3_\n4_ 5 6 7"
    );
}

#[test]
fn a_misbarred_measure_whose_re_barring_errors_is_still_eligible() {
    let source = r#"parts ["Melody" [M]: notes],

score """
    [M] 1 2 3 4_ 5 -
    """,
"#;
    assert_eq!(list_streamable_parts(source)[0].ineligible_reason, None);
}

#[test]
fn inline_directive_makes_the_part_ineligible() {
    let source = r#"parts ["Melody" [M]: notes, "Bass" [B]: notes],

score """
    [M] 1 2 3 4

    [B] 1 1 1 1
    [M] 1 2 bpm=90 3 4
    """,
"#;
    let reason = list_streamable_parts(source)[0]
        .ineligible_reason
        .clone()
        .unwrap();
    assert!(reason.contains("measure 2"), "{reason}");
    assert!(reason.contains("inline directive"), "{reason}");
}

#[test]
fn a_measure_starting_with_an_extension_makes_the_part_ineligible() {
    let source = r#"parts ["Melody" [M]: notes],

score """
    [M] - 1 2 3
    """,
"#;
    let reason = list_streamable_parts(source)[0]
        .ineligible_reason
        .clone()
        .unwrap();
    assert!(
        reason.contains("`-` that has no note to extend"),
        "{reason}"
    );
}

#[test]
fn lists_every_part_with_its_kind_and_eligibility() {
    let source = r#"parts [
    "Melody" [M]: notes,
    "Words" [W]: lyrics [M],
    "Harmony" [H]: chords,
    "Drums" [D]: percussion,
    "Echo" [E]: follow [M],
],

score """
    [M] 1 2 3 4
    [W] a b c d
    [H] 1 4 5 1
    [D] x x x x
    """,
"#;
    let parts = list_streamable_parts(source);
    let summary: Vec<(&str, StreamablePartKind, bool)> = parts
        .iter()
        .map(|part| {
            (
                part.abbreviation.as_str(),
                part.kind,
                part.ineligible_reason.is_none(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        vec![
            ("M", StreamablePartKind::Notes, true),
            ("W", StreamablePartKind::Lyrics, true),
            ("H", StreamablePartKind::Chords, true),
            ("D", StreamablePartKind::Percussion, true),
            ("E", StreamablePartKind::Follow, true),
        ]
    );
    assert_eq!(parts[0].display_name, "Melody");
}
