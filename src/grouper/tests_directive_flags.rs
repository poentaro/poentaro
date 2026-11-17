use super::*;

#[test]
fn merge_duplicate_measures_across_parts_directive_propagates_and_persists() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120 merge_duplicate_measures_across_parts=no
    [Melody] 1 2 3 4

    [Melody] 5 6 7 1
    """,
"#,
    );
    assert!(!score.measures[0].merge_duplicate_measures_across_parts);
    assert!(
        !score.measures[1].merge_duplicate_measures_across_parts,
        "sticky: stays disabled on the next measure with no override"
    );
}

#[test]
fn merge_duplicate_measures_across_parts_can_be_re_enabled_after_being_disabled() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120 merge_duplicate_measures_across_parts=no
    [Melody] 1 2 3 4

    merge_duplicate_measures_across_parts=yes
    [Melody] 5 6 7 1
    """,
"#,
    );
    assert!(!score.measures[0].merge_duplicate_measures_across_parts);
    assert!(score.measures[1].merge_duplicate_measures_across_parts);
}

#[test]
fn invalid_hide_resting_parts_directive_value_attaches_diagnostic_to_measure() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120 hide_resting_parts=maybe
    [Melody] 1 2 3 4
    """,
"#,
    );
    assert!(
        !score.measures[0].diagnostics.is_empty(),
        "an invalid hide_resting_parts value should be a recoverable error"
    );
}
