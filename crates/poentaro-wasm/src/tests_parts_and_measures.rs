use super::*;
use crate::responses::{get_measure_at_offset_response, list_measure_spans_response};
use crate::types::{ListMeasureSpansResponse, ListPartsResponse, MeasureAtOffsetResponse};

#[test]
fn list_parts_response_returns_declarations() {
    let input = r#"metadata { title "t", author "a" },

parts ["Soprano": notes, "Alto": notes],

score """
    time=4/4 key=C4 bpm=120
    [Soprano] 1 2 3 4
    [Alto] 5 6 7 1
    """,
"#;
    let resp = part_declarations::list_parts_response(input, &[]);
    match resp {
        ListPartsResponse::Ok { parts, .. } => {
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0].abbreviation, "Soprano");
            assert_eq!(parts[1].abbreviation, "Alto");
        }
        ListPartsResponse::Err { diagnostics } => {
            panic!("expected ok: {}", diagnostics[0].message);
        }
    }
}

#[test]
fn get_measure_at_offset_ok_for_note_in_measure() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    """,
"#;
    let byte_offset = source.find("1 2 3 4").unwrap();
    let resp = get_measure_at_offset_response(source, byte_offset);
    match resp {
        MeasureAtOffsetResponse::Ok { measure_index } => assert_eq!(measure_index, 0),
        MeasureAtOffsetResponse::NotInMeasure => panic!("expected Ok"),
    }
}

#[test]
fn get_measure_at_offset_not_in_measure_for_header() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    """,
"#;
    let resp = get_measure_at_offset_response(source, 0);
    assert!(
        matches!(resp, MeasureAtOffsetResponse::NotInMeasure),
        "expected NotInMeasure"
    );
}

#[test]
fn list_measure_spans_returns_one_span_per_measure() {
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    [Melody] 1 2 3 4

    [Melody] 5 6 7 1
    """,
"#;
    let resp = list_measure_spans_response(input);
    match resp {
        ListMeasureSpansResponse::Ok { spans, .. } => {
            assert_eq!(spans.len(), 2);
            assert!(spans[0].start < spans[1].start);
            // No separate directive line precedes the first measure, so view_zone_start
            // is on the same source line as start (the [Melody] prefix is on that line).
            let count_newlines = |s: &str, end: usize| s[..end].matches('\n').count();
            assert_eq!(
                count_newlines(input, spans[0].view_zone_start),
                count_newlines(input, spans[0].start),
            );
        }
        ListMeasureSpansResponse::Err => panic!("expected ok"),
    }
}

#[test]
fn list_measure_spans_view_zone_start_includes_directive_line() {
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    bpm=60
    [Melody] 1 2 3 4
    """,
"#;
    let directive_offset = input.find("bpm=60").unwrap();
    let notes_offset = input.find("1 2 3 4").unwrap();
    let resp = list_measure_spans_response(input);
    match resp {
        ListMeasureSpansResponse::Ok { spans, .. } => {
            assert_eq!(spans.len(), 1);
            assert_eq!(spans[0].view_zone_start, directive_offset);
            assert_eq!(spans[0].start, notes_offset);
        }
        ListMeasureSpansResponse::Err => panic!("expected ok"),
    }
}

#[test]
fn list_measure_spans_returns_empty_for_invalid_source() {
    // Missing sections are recoverable; the response is Ok with no spans.
    let resp = list_measure_spans_response("not valid poentaro");
    match resp {
        ListMeasureSpansResponse::Ok { spans, .. } => assert!(spans.is_empty()),
        ListMeasureSpansResponse::Err => {}
    }
}

#[test]
fn list_measure_spans_returns_sequence_entries_in_playback_order_with_repeats() {
    // `sequence` replays the "Chorus" span twice, so the resolved entries
    // must reflect playback order (Verse, Chorus, Chorus), not written order
    // (Verse, Chorus), and each occurrence must carry its own measure range.
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

sequence [Verse, Chorus, Chorus],

score """
    time=4/4 key=C4 bpm=120 label="Verse"
    [Melody] 1 2 3 4

    label="Chorus"
    [Melody] 5 6 7 1
    """,
"#;
    let resp = list_measure_spans_response(input);
    match resp {
        ListMeasureSpansResponse::Ok {
            sequence_entries, ..
        } => {
            let labels: Vec<&str> = sequence_entries.iter().map(|e| e.label.as_str()).collect();
            assert_eq!(labels, vec!["Verse", "Chorus", "Chorus"]);
            assert_eq!(sequence_entries[0].start_measure_index, 0);
            assert_eq!(sequence_entries[0].end_measure_index, 0);
            assert_eq!(sequence_entries[1].start_measure_index, 1);
            assert_eq!(sequence_entries[1].end_measure_index, 1);
            assert_eq!(sequence_entries[2].start_measure_index, 1);
            assert_eq!(sequence_entries[2].end_measure_index, 1);
        }
        ListMeasureSpansResponse::Err => panic!("expected ok"),
    }
}

#[test]
fn list_measure_spans_sequence_entry_label_includes_part_omission_suffix() {
    // `Y(-b)` omits part "b" from that one occurrence's playback; the entry's
    // label must carry that suffix so the sequence jump toolbar can show
    // "Y(-b)" distinctly from the plain "Y" entries that follow it.
    let input = r#"metadata { title "t", author "a" },

parts ["a": notes, "b": notes],

sequence ["X", "Y" WITHOUT [b], "Y", "X"],

score """
    time=4/4 key=C4 bpm=120 label="X"
    [a] 1

    label="Y"
    [a] 2
    [b] 5
    """,
"#;
    let resp = list_measure_spans_response(input);
    match resp {
        ListMeasureSpansResponse::Ok {
            sequence_entries, ..
        } => {
            let labels: Vec<&str> = sequence_entries.iter().map(|e| e.label.as_str()).collect();
            assert_eq!(labels, vec!["X", "Y(-b)", "Y", "X"]);
        }
        ListMeasureSpansResponse::Err => panic!("expected ok"),
    }
}

#[test]
fn list_measure_spans_sequence_entry_label_includes_part_only_suffix() {
    // `Y(a)` plays only part "a" on that one occurrence; the entry's label
    // must carry that suffix, written without a leading dash so it reads
    // distinctly from an omit suffix like `Y(-b)`.
    let input = r#"metadata { title "t", author "a" },

parts ["a": notes, "b": notes],

sequence ["X", "Y" ONLY [a], "Y", "X"],

score """
    time=4/4 key=C4 bpm=120 label="X"
    [a] 1

    label="Y"
    [a] 2
    [b] 5
    """,
"#;
    let resp = list_measure_spans_response(input);
    match resp {
        ListMeasureSpansResponse::Ok {
            sequence_entries, ..
        } => {
            let labels: Vec<&str> = sequence_entries.iter().map(|e| e.label.as_str()).collect();
            assert_eq!(labels, vec!["X", "Y(a)", "Y", "X"]);
        }
        ListMeasureSpansResponse::Err => panic!("expected ok"),
    }
}

#[test]
fn list_measure_spans_returns_empty_sequence_entries_without_sequence_section() {
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    [Melody] 1 2 3 4
    """,
"#;
    let resp = list_measure_spans_response(input);
    match resp {
        ListMeasureSpansResponse::Ok {
            sequence_entries, ..
        } => assert!(sequence_entries.is_empty()),
        ListMeasureSpansResponse::Err => panic!("expected ok"),
    }
}
