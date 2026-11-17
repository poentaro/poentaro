#![allow(clippy::disallowed_macros)]
use poentaro::list_measure_spans_from_source;

const TWO_MEASURE_SOURCE: &str = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    [Melody] 1 2 3 4

    [Melody] 5 6 7 1
    """,
"#;

const DIRECTIVE_MEASURE_SOURCE: &str = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    bpm=60
    [Melody] 1 2 3 4
    """,
"#;

const LABEL_DIRECTIVE_SOURCE: &str = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    label="something "
    [Melody] 1 2 3 4
    """,
"#;

#[test]
fn returns_one_span_per_measure() {
    let spans = list_measure_spans_from_source(TWO_MEASURE_SOURCE, "test.poentaro")
        .unwrap()
        .spans;
    assert_eq!(spans.len(), 2);
}

#[test]
fn spans_are_ordered_by_source_position() {
    let spans = list_measure_spans_from_source(TWO_MEASURE_SOURCE, "test.poentaro")
        .unwrap()
        .spans;
    assert!(spans[0].start < spans[1].start);
}

#[test]
fn view_zone_start_is_at_or_before_content_start_without_directive() {
    // When there is no leading directive line, the view zone starts at the beginning
    // of the first data line (the [Abbrev] prefix), which is at or before the
    // note content start.
    let spans = list_measure_spans_from_source(TWO_MEASURE_SOURCE, "test.poentaro")
        .unwrap()
        .spans;
    assert!(spans[0].view_zone_start <= spans[0].start);
    assert!(spans[1].view_zone_start <= spans[1].start);
    // The view zone should start where the [Abbrev] prefix begins on the first data line.
    let first_line_start = TWO_MEASURE_SOURCE.find("[Melody] 1 2 3 4").unwrap();
    assert_eq!(spans[0].view_zone_start, first_line_start);
    let second_line_start = TWO_MEASURE_SOURCE.find("[Melody] 5 6 7 1").unwrap();
    assert_eq!(spans[1].view_zone_start, second_line_start);
}

#[test]
fn view_zone_start_includes_leading_directive_line() {
    let spans = list_measure_spans_from_source(DIRECTIVE_MEASURE_SOURCE, "test.poentaro")
        .unwrap()
        .spans;
    assert_eq!(spans.len(), 1);

    let directive_offset = DIRECTIVE_MEASURE_SOURCE.find("bpm=60").unwrap();
    let notes_offset = DIRECTIVE_MEASURE_SOURCE.find("1 2 3 4").unwrap();

    assert_eq!(spans[0].view_zone_start, directive_offset);
    assert_eq!(spans[0].start, notes_offset);
    assert!(spans[0].view_zone_start < spans[0].start);
}

#[test]
fn returns_empty_spans_on_source_with_no_sections() {
    // Section-structure errors are recoverable; a source with no section headers
    // produces an empty score (no measures), not an Err.
    let result = list_measure_spans_from_source("not valid poentaro", "test.poentaro").unwrap();
    assert!(result.spans.is_empty());
}

/// When the caret is on a directive line (e.g. `label="something "`), the measure
/// must still be detected. `start_line` must reach back to the directive line, not
/// only to the first note line.
#[test]
fn start_line_includes_label_directive_line() {
    let spans = list_measure_spans_from_source(LABEL_DIRECTIVE_SOURCE, "test.poentaro")
        .unwrap()
        .spans;
    assert_eq!(spans.len(), 1);

    let directive_line: usize = LABEL_DIRECTIVE_SOURCE
        .lines()
        .enumerate()
        .find(|(_, line)| line.trim_start().starts_with("label="))
        .map(|(i, _)| i + 1) // convert 0-indexed to 1-indexed
        .expect("label= line not found in LABEL_DIRECTIVE_SOURCE");

    assert_eq!(
        spans[0].start_line, directive_line,
        "start_line ({}) should be the directive line ({}) so that a caret on \
         `label=\"something \"` is detected as belonging to this measure",
        spans[0].start_line, directive_line,
    );
}

/// Regression: with multiple parts sharing a single notes row per group,
/// the `start_line`/`end_line` of each measure span must not overlap adjacent
/// measures.  Specifically, the notes line of group 3 (1-based line 23) must
/// belong to exactly one measure span — measure index 2 — and measure index 1
/// must end before that line.
#[test]
fn multipart_measure_spans_do_not_overlap_across_groups() {
    let source = r#"metadata { title "", author "" },

parts [
    "Chord": chords,
    "Alto 1 & Tenor (A1,T)": notes,
    "Alto 2 (A2)": notes,
    "Soprano 1 (S1)": notes,
    "Soprano 2 (S2)": notes,
],

score """
    bpm=80 key=C4 time=4/4 label="Verse 1"
    [Chord] 1 - - -
    [A1,T] 5_ 5_ 5_ 5= 5= 5_ 3_ 2_ (3_
    白陽旗旛在大道盛宏

    [Chord] 6m/3
    [A1,T] 3_) (1_1-) 0_ 1= 1=
    昌花花

    [Chord] 4
    [A1,T] 2. 3_ 4_ 3= 3= (2_1_)
    草擺動道音歌-
    """,
"#;

    // Line 23 (1-based) contains "2. 3_ 4_ 3= 3= (2_1_)" — the notes line of group 3.
    let caret_line: usize = 23;
    let spans = list_measure_spans_from_source(source, "test.poentaro")
        .unwrap()
        .spans;

    assert_eq!(spans.len(), 3, "expected exactly 3 measures");

    // Exactly one span should contain the caret line.
    let matching: Vec<usize> = spans
        .iter()
        .enumerate()
        .filter(|(_, s)| s.start_line <= caret_line && caret_line <= s.end_line)
        .map(|(i, _)| i)
        .collect();

    assert_eq!(
        matching,
        vec![2],
        "caret on line {caret_line} should match only measure index 2, got {:?} \
         (measure 1 end_line={}, measure 2 start_line={})",
        matching,
        spans[1].end_line,
        spans[2].start_line,
    );
}

const SEQUENCE_SOURCE: &str = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

sequence [Verse, Chorus, Chorus],

score """
    time=4/4 key=C4 bpm=120 label="Verse"
    [Melody] 1 2 3 4

    label="Chorus"
    [Melody] 5 6 7 1
    """,
"#;

/// `sequence` resolution must be exposed alongside the source spans, in
/// playback order (repeats included), so callers can bound playback to a
/// single occurrence without re-deriving it from written order.
#[test]
fn sequence_field_reflects_playback_order_with_repeats() {
    let result = list_measure_spans_from_source(SEQUENCE_SOURCE, "test.poentaro").unwrap();
    let sequence = result.sequence.expect("expected a resolved sequence");

    let labels: Vec<&str> = sequence.iter().map(|s| s.label.as_str()).collect();
    assert_eq!(labels, vec!["Verse", "Chorus", "Chorus"]);
    assert_eq!((sequence[0].start, sequence[0].end), (0, 0));
    assert_eq!((sequence[1].start, sequence[1].end), (1, 1));
    assert_eq!((sequence[2].start, sequence[2].end), (1, 1));
}

#[test]
fn sequence_field_is_none_without_sequence_section() {
    let result = list_measure_spans_from_source(TWO_MEASURE_SOURCE, "test.poentaro").unwrap();
    assert!(result.sequence.is_none());
}
