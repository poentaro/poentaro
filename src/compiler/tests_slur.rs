use crate::compiler::compile;
use crate::grouper::group;
use crate::parser::parse;

fn score_from(source: &str) -> crate::ast::grouped::Score {
    let doc = parse(source, "test", &[]).unwrap();
    group(doc).unwrap()
}

fn notes_doc(score_content: &str) -> String {
    format!(
        r#"metadata {{ title "t", author "a" }},

parts ["S": notes],

score """
    {score_content}
    """,
"#
    )
}

fn chord_doc(score_content: &str) -> String {
    format!(
        r#"metadata {{ title "t", author "a" }},

parts ["C": chords],

score """
    {score_content}
    """,
"#
    )
}

#[test]
fn three_same_pitch_notes_in_slur_emits_one_slur_arc() {
    // "(555)" — three quarter notes of the same pitch under a slur group.
    // A slur draws one arc from first to last: col 4→12.
    // col 0=note 1, col 4=first 5, col 8=second 5, col 12=third 5.
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1 (555)\n"));
    let result = compile(&score);
    assert_eq!(
        result.slur_spans.len(),
        1,
        "expected 1 slur arc for (555), got: {:?}",
        result.slur_spans
    );
    assert!(
        result
            .slur_spans
            .iter()
            .any(|s| s.from_column == 4 && s.to_column == 12),
        "expected arc col 4→12, got: {:?}",
        result.slur_spans
    );
}

#[test]
fn same_measure_slur_emits_slur_span() {
    // "(4 5)" open on note 4 (col 0), close on note 5 (col 4).
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] (4 5) 0 0\n"));
    let result = compile(&score);
    assert!(
        result.slur_spans.iter().any(|s| {
            s.part_index == 0
                && s.from_measure == 0
                && s.from_column == 0
                && s.to_measure == 0
                && s.to_column == 4
        }),
        "expected SlurSpan (measure=0, col=0) → (measure=0, col=4), got: {:?}",
        result.slur_spans
    );
}

#[test]
fn cross_measure_slur_emits_single_slur_span() {
    // Bar 1: "1 2 3 (4" — slur opens on note 4 at col 12.
    // Bar 2: "5) 6 7 1" — slur closes on note 5 at col 0.
    let score = score_from(&notes_doc(concat!(
        "time=4/4 key=C4 bpm=120\n",
        "[S] 1 2 3 (4\n",
        "\n",
        "[S] 5) 6 7 1\n",
    )));
    let result = compile(&score);
    assert!(
        result.slur_spans.iter().any(|s| {
            s.part_index == 0
                && s.from_measure == 0
                && s.from_column == 12
                && s.to_measure == 1
                && s.to_column == 0
        }),
        "expected SlurSpan (measure=0, col=12) → (measure=1, col=0), got: {:?}",
        result.slur_spans
    );
    assert!(
        result
            .slur_spans
            .iter()
            .all(|s| s.from_column != 16 && s.to_column != 16),
        "no slur span should touch barline col 16, got: {:?}",
        result.slur_spans
    );
}

#[test]
fn cross_measure_same_pitch_slur_emits_single_slur_span() {
    // Bar 1: "1 2 3 (4" — slur opens on note 4 at col 12.
    // Bar 2: "4) 5 6 7" — slur closes on note 4 at col 0.
    let score = score_from(&notes_doc(concat!(
        "time=4/4 key=C4 bpm=120\n",
        "[S] 1 2 3 (4\n",
        "\n",
        "[S] 4) 5 6 7\n",
    )));
    let result = compile(&score);
    assert!(
        result.slur_spans.iter().any(|s| {
            s.part_index == 0
                && s.from_measure == 0
                && s.from_column == 12
                && s.to_measure == 1
                && s.to_column == 0
        }),
        "expected SlurSpan (measure=0, col=12) → (measure=1, col=0), got: {:?}",
        result.slur_spans
    );
}

#[test]
fn cross_measure_slur_closing_on_extension_dash() {
    // Bar 1: "1 2 3 (4" — slur opens on note 4 at col 12.
    // Bar 2: "5 -) - -" — slur closes at the extension dash at col 4.
    let score = score_from(&notes_doc(concat!(
        "time=4/4 key=C4 bpm=120\n",
        "[S] 1 2 3 (4\n",
        "\n",
        "[S] 5 -) - -\n",
    )));
    let result = compile(&score);
    assert!(
        result.slur_spans.iter().any(|s| {
            s.part_index == 0
                && s.from_measure == 0
                && s.from_column == 12
                && s.to_measure == 1
                && s.to_column == 4
        }),
        "expected SlurSpan (measure=0, col=12) → (measure=1, col=4), got: {:?}",
        result.slur_spans
    );
    assert!(
        result.slur_spans.iter().all(|s| s.to_column != 16),
        "no slur span should end at barline col 16"
    );
}

#[test]
fn three_measure_slur_emits_single_slur_span() {
    // Bar 1: "(1 2 3 4" — slur opens on note 1 at col 0, multiple notes in slur.
    // Bar 2: "5 6 7 1" — all notes in slur continue.
    // Bar 3: "2) 3 4 5" — slur closes on note 2 at col 0.
    let score = score_from(&notes_doc(concat!(
        "time=4/4 key=C4 bpm=120\n",
        "[S] (1 2 3 4\n",
        "\n",
        "[S] 5 6 7 1\n",
        "\n",
        "[S] 2) 3 4 5\n",
    )));
    let result = compile(&score);
    assert!(
        result.slur_spans.iter().any(|s| {
            s.part_index == 0
                && s.from_measure == 0
                && s.from_column == 0
                && s.to_measure == 2
                && s.to_column == 0
        }),
        "expected SlurSpan (measure=0, col=0) → (measure=2, col=0), got: {:?}",
        result.slur_spans
    );
}

#[test]
fn cross_measure_arc_dropped_when_target_measure_has_error() {
    // Bar 1: "1 2 3 (4" — slur group opens on note 4 at col 12.
    // Bar 2: "- 5) 6 7 1" — lone '-' triggers ExtensionNoPrecedingEvent (has_error=true),
    //   so the pending slur open is cleared before bar 2 compiles.
    //   Note 5 would normally close the group but no open exists after the reset.
    // Expected: no slur span connecting bar 0 to bar 1.
    let score = score_from(&notes_doc(concat!(
        "time=4/4 key=C4 bpm=120\n",
        "[S] 1 2 3 (4\n",
        "\n",
        "[S] - 5) 6 7 1\n",
    )));
    let result = compile(&score);
    assert!(
        result
            .slur_spans
            .iter()
            .all(|s| !(s.from_measure == 0 && s.to_measure == 1)),
        "cross-measure arc into errored measure should be dropped, got: {:?}",
        result.slur_spans
    );
}

#[test]
fn cross_measure_chord_slur_emits_single_slur_span() {
    // Bar 1: "1 2 3 (4m" — slur opens on chord 4m at col 12.
    // Bar 2: "5) 6 7 1"  — slur closes on chord 5 at col 0.
    let score = score_from(&chord_doc(concat!(
        "time=4/4 key=C4 bpm=120\n",
        "[C] 1 2 3 (4m\n",
        "\n",
        "[C] 5) 6 7 1\n",
    )));
    let result = compile(&score);
    assert!(
        result.slur_spans.iter().any(|s| {
            s.part_index == 0
                && s.from_measure == 0
                && s.from_column == 12
                && s.to_measure == 1
                && s.to_column == 0
        }),
        "expected SlurSpan (measure=0, col=12) → (measure=1, col=0), got: {:?}",
        result.slur_spans
    );
}

#[test]
fn three_measure_slur_with_single_note_middle_measure() {
    // Bar 1: "1 2 3 (4" — slur opens on note 4 at col 12.
    // Bar 2: "5 6 7 1" — single measure with all notes in slur continuation.
    // Bar 3: "2) 3 4 5" — slur closes on note 2 at col 0.
    let score = score_from(&notes_doc(concat!(
        "time=4/4 key=C4 bpm=120\n",
        "[S] 1 2 3 (4\n",
        "\n",
        "[S] 5 6 7 1\n",
        "\n",
        "[S] 2) 3 4 5\n",
    )));
    let result = compile(&score);
    assert!(
        result.slur_spans.iter().any(|s| {
            s.part_index == 0
                && s.from_measure == 0
                && s.from_column == 12
                && s.to_measure == 2
                && s.to_column == 0
        }),
        "expected SlurSpan (measure=0, col=12) → (measure=2, col=0), got: {:?}",
        result.slur_spans
    );
}

#[test]
fn tilde_cross_measure_chord_slur_emits_single_slur_span() {
    // [a] 3~---  => chord 3 tied (via ~) into 3 extensions, filling measure 1
    // [a] 3      => chord 3 closes the tie in measure 2
    let input = r#"metadata { title "", author "" },

parts ["Accompaniment" [a]: chords],

score """
    [a] 3~---

    [a] 3
    """,
"#;
    let doc = parse(input, "test", &[]).unwrap();
    let score = group(doc).unwrap();
    let result = compile(&score);
    assert!(
        result
            .slur_spans
            .iter()
            .any(|s| { s.part_index == 0 && s.from_measure == 0 && s.to_measure == 1 }),
        "expected a SlurSpan crossing from measure 0 to measure 1 for tilde chord tie, got: {:?}",
        result.slur_spans
    );
}
