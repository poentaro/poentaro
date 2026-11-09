use crate::compiler::{compile, types::*};
use crate::grouper::group;
use crate::parser::parse;

fn score_from(source: &str) -> crate::ast::grouped::Score {
    let doc = parse(source, "test", &[]).unwrap();
    group(doc).unwrap()
}

/// Minimal one-part (notes) document. `score_content` is everything after `score\n`.
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

/// Chord-part document with one track.
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
fn single_quarter_note_produces_one_note_head_element() {
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    assert!(!blocks.is_empty());
    let row = &blocks[0].rows[0];
    let note_heads: Vec<_> = row
        .elements
        .iter()
        .filter(|e| matches!(e.content, ElementContent::NoteHead { .. }))
        .collect();
    assert_eq!(note_heads.len(), 1);
}

#[test]
fn bar_line_is_last_element_in_row() {
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let row = &blocks[0].rows[0];
    let last = row.elements.last().unwrap();
    assert_eq!(last.content, ElementContent::BarLine);
}

#[test]
fn bpm_decoration_on_first_measure() {
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=100\n[S] 1\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let has_bpm = blocks[0]
        .decorations
        .iter()
        .any(|d| matches!(d, Decoration::DirectiveLine { bpm: Some(100), .. }));
    assert!(has_bpm);
}

#[test]
fn two_measures_produce_two_blocks() {
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1\n\n[S] 2\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    assert_eq!(blocks.len(), 2);
}

#[test]
fn eighth_notes_produce_underline_elements() {
    // 2_ means eighth note (duration=2 quarter-beats) in `.poentaro` syntax
    // Two eighth notes fill one beat; padded with rests to complete 4/4
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 2_ 2_ 0 0 0\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let row = &blocks[0].rows[0];
    let underlines: Vec<_> = row
        .elements
        .iter()
        .filter(|e| matches!(e.content, ElementContent::Underline { .. }))
        .collect();
    assert!(!underlines.is_empty(), "expected at least one underline");
}

#[test]
fn time_signature_appears_as_decoration() {
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let has_ts = blocks[0].decorations.iter().any(|d| {
        matches!(
            d,
            Decoration::DirectiveLine {
                time_signature: Some((4, 4)),
                ..
            }
        )
    });
    assert!(has_ts);
}

#[test]
fn bar_number_decoration_without_label() {
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1\n\n[S] 2\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let bar1 = blocks[0].decorations.first().unwrap();
    assert!(
        matches!(
            bar1,
            Decoration::DirectiveLine {
                bar_number: Some(1),
                ..
            }
        ),
        "first measure should have bar_number=1"
    );
    let bar2 = blocks[1].decorations.first().unwrap();
    assert!(
        matches!(
            bar2,
            Decoration::DirectiveLine {
                bar_number: Some(2),
                ..
            }
        ),
        "second measure should have bar_number=2"
    );
}

#[test]
fn section_label_measure_still_has_bar_number() {
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120 label=\"Verse 1\"\n[S] 1\n",
    ));
    let result = compile(&score);
    let blocks = result.blocks;
    let dec = blocks[0].decorations.first().unwrap();
    assert!(
        matches!(
            dec,
            Decoration::DirectiveLine {
                bar_number: Some(_),
                ..
            }
        ),
        "labeled measure should still show its bar number"
    );
    assert!(
        matches!(dec, Decoration::DirectiveLine { label: Some(_), .. }),
        "labeled measure should have a label"
    );
}

#[test]
fn rest_produces_rest_element() {
    // A lone `0` fills the whole 4/4 measure by extending its duration
    // (equivalent to `0---`), matching how a lone note pads with dashes.
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 0\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let row = &blocks[0].rows[0];
    let rests: Vec<_> = row
        .elements
        .iter()
        .filter(|e| matches!(e.content, ElementContent::Rest { .. }))
        .collect();
    assert_eq!(rests.len(), 1);
}

#[test]
fn bar_line_column_equals_total_duration() {
    // "1 2 3 4" = four quarter notes, each duration=4 → total 16 quarter-beats
    // Bar line should appear at column 16
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1 2 3 4\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let row = &blocks[0].rows[0];
    let bar_line = row
        .elements
        .iter()
        .find(|e| matches!(e.content, ElementContent::BarLine))
        .unwrap();
    assert_eq!(
        bar_line.column, 16,
        "bar line should be at column 16 for four quarter notes"
    );
}

#[test]
fn not_mentioned_chord_part_is_omitted_when_other_parts_have_notes() {
    // B (chord) is not mentioned in this key-based measure, so it gets rest-filled.
    // Because A and C have actual notes, B should be omitted from the rendered rows.
    let score = score_from(
        r#"metadata { title "t", author "a" },

parts ["A": notes, "B": chords, "C": notes],

score """
    time=4/4 key=C4 bpm=120
    [A] 1 2 3 4
    la la la la
    [C] 1
    """,
"#,
    );
    let result = compile(&score);
    let blocks = result.blocks;
    assert_eq!(
        blocks[0].rows.len(),
        2,
        "B (rest-filled) should be omitted when A and C have notes"
    );
    assert_eq!(blocks[0].rows[0].label, "A", "first row label should be A");
    assert_eq!(blocks[0].rows[1].label, "C", "second row label should be C");
}

#[test]
fn not_mentioned_part_is_kept_when_hide_resting_parts_is_disabled() {
    let score = score_from(
        r#"metadata { title "t", author "a", hide resting parts: no },

parts ["A": notes, "B": chords, "C": notes],

score """
    time=4/4 key=C4 bpm=120
    [A] 1 2 3 4
    la la la la
    [C] 1
    """,
"#,
    );
    let result = compile(&score);
    let blocks = result.blocks;
    assert_eq!(
        blocks[0].rows.len(),
        3,
        "B (rest-filled) should be kept when hide_resting_parts is disabled"
    );
}

#[test]
fn extended_note_produces_note_dash_at_each_extra_beat() {
    // "1- 2-" = two half notes filling a 4/4 measure (8+8=16 quarter-beats).
    // Each half note should produce one NoteDash at the beat following the note head.
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1- 2-\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let row = &blocks[0].rows[0];
    let dashes: Vec<_> = row
        .elements
        .iter()
        .filter(|e| matches!(e.content, ElementContent::NoteDash { .. }))
        .collect();
    assert_eq!(
        dashes.len(),
        2,
        "two half notes should produce two NoteDash elements"
    );
    assert_eq!(dashes[0].column, 4, "first NoteDash should be at column 4");
    assert_eq!(
        dashes[1].column, 12,
        "second NoteDash should be at column 12"
    );
}

#[test]
fn extended_chord_produces_note_dash_at_each_extra_beat() {
    // "1 - - -" = a whole-note chord filling a 4/4 measure.
    // The three `-` tokens should each produce a NoteDash at columns 4, 8, 12.
    let score = score_from(&chord_doc("time=4/4 key=C4 bpm=120\n[C] 1 - - -\n"));
    let result = compile(&score);
    let row = &result.blocks[0].rows[0];
    let dashes: Vec<_> = row
        .elements
        .iter()
        .filter(|e| matches!(e.content, ElementContent::NoteDash { .. }))
        .collect();
    assert_eq!(
        dashes.len(),
        3,
        "three `-` tokens should produce three NoteDash elements"
    );
    assert_eq!(dashes[0].column, 4, "first dash at column 4");
    assert_eq!(dashes[1].column, 8, "second dash at column 8");
    assert_eq!(dashes[2].column, 12, "third dash at column 12");
}

#[test]
fn dotted_chord_produces_chord_symbol_with_dotted_flag_set() {
    let score = score_from(&chord_doc("time=4/4 key=C4 bpm=120\n[C] 1. 3\n"));
    let result = compile(&score);
    let row = &result.blocks[0].rows[0];
    let chord_symbols: Vec<_> = row
        .elements
        .iter()
        .filter_map(|e| match &e.content {
            ElementContent::ChordSymbol { text, dotted, .. } => Some((text.clone(), *dotted)),
            _ => None,
        })
        .collect();
    assert_eq!(
        chord_symbols,
        vec![("1".to_string(), true), ("3".to_string(), false)]
    );
}

#[test]
fn dotted_note_extended_with_dotted_beats_produces_note_dash_at_each_extra_beat() {
    // In 9/8, "1. -. -." is a dotted quarter (6 quarter-beats) plus two dotted-beat
    // extensions (6 + 6), filling the whole 18-quarter-beat measure.
    // Each `-.` extension should produce a NoteDash, same as plain `-` does for
    // undotted notes — but the note head itself is dotted, which currently makes
    // `compile_unit`'s `if !unit.dotted` guard skip the dash-emission loop entirely.
    let score = score_from(&notes_doc("time=9/8 key=C4 bpm=120\n[S] 1. -. -.\n"));
    let result = compile(&score);
    let row = &result.blocks[0].rows[0];
    let dashes: Vec<_> = row
        .elements
        .iter()
        .filter(|e| matches!(e.content, ElementContent::NoteDash { .. }))
        .collect();
    assert_eq!(
        dashes.len(),
        2,
        "two `-.` tokens should produce two NoteDash elements"
    );
    assert_eq!(dashes[0].column, 6, "first dash at column 6");
    assert_eq!(dashes[1].column, 12, "second dash at column 12");
}

#[test]
fn compound_meter_nine_eighth_groups_underlines_three_by_three() {
    // In 9/8 (a compound meter), the beat is a dotted quarter = 3 eighth notes, so
    // nine eighth notes filling the measure should beam into 3 groups of 3 — not
    // the 2-by-2 (quarter-note) grouping used for simple meters like 4/4.
    let score = score_from(&notes_doc(
        "time=9/8 key=C4 bpm=120\n[S] 1_1_1_ 2_2_2_ 3_3_3_\n",
    ));
    let result = compile(&score);
    let row = &result.blocks[0].rows[0];
    let underline_starts: Vec<u32> = row
        .elements
        .iter()
        .filter_map(|e| match e.content {
            ElementContent::Underline { level: 0, .. } => Some(e.column),
            _ => None,
        })
        .collect();
    assert_eq!(
        underline_starts,
        vec![0, 6, 12],
        "expected three beam groups of 3 eighth notes each (columns 0, 6, 12)"
    );
}

#[test]
fn note_head_column_is_zero_indexed() {
    // First note in measure should be at column 0
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let row = &blocks[0].rows[0];
    let note_head = row
        .elements
        .iter()
        .find(|e| matches!(e.content, ElementContent::NoteHead { .. }))
        .unwrap();
    assert_eq!(note_head.column, 0);
}
