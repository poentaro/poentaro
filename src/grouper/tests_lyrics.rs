use super::group;
use crate::parser;

#[test]
fn lyrics_overflow_recovers_with_error_on_measure() {
    // 2 notes but 4 syllables → should not Err, should attach error to measure
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 0 0
    [Melodyv1] a b c d e f
    """,
"#;
    let doc = parser::parse(input, "test.poentaro", &[]).unwrap();
    let score = group(doc).expect("overflow must not abort grouping");
    assert_eq!(score.measures.len(), 1);
    assert_eq!(score.measures[0].diagnostics.len(), 1);
    assert!(
        score.measures[0].diagnostics[0]
            .message()
            .contains("overflow"),
        "error message should mention overflow, got: {}",
        score.measures[0].diagnostics[0].message()
    );
}

#[test]
fn lyrics_underflow_recovers_with_error_on_measure() {
    // 4 notes but only 2 syllables → should not Err, should attach error to measure
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b
    """,
"#;
    let doc = parser::parse(input, "test.poentaro", &[]).unwrap();
    let score = group(doc).expect("underflow must not abort grouping");
    assert_eq!(score.measures.len(), 1);
    assert_eq!(score.measures[0].diagnostics.len(), 1);
    assert!(
        score.measures[0].diagnostics[0]
            .message()
            .contains("underflow"),
        "error message should mention underflow, got: {}",
        score.measures[0].diagnostics[0].message()
    );
}

#[test]
fn lyrics_underflow_error_span_covers_lyrics_line_not_notes() {
    // 4 notes but only 2 syllables → underflow error span must point at the
    // lyrics line ("a b"), not the notes line ("1 2 3 4").
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b
    """,
"#;
    let doc = parser::parse(input, "test.poentaro", &[]).unwrap();
    let score = group(doc).expect("underflow must not abort grouping");

    let lyrics_line_offset = input.find("a b").unwrap();
    let notes_line_offset = input.find("1 2 3 4").unwrap();

    let error = &score.measures[0].diagnostics[0];
    assert!(
        error.span().start >= lyrics_line_offset,
        "underflow span should start at the lyrics line (offset {}), not before it (notes are at {}); got span.start={}",
        lyrics_line_offset,
        notes_line_offset,
        error.span().start,
    );
    assert!(
        error.span().end >= lyrics_line_offset,
        "underflow span should cover the lyrics line; got span.end={}",
        error.span().end,
    );
}

#[test]
fn measures_without_lyrics_underflow_have_no_errors() {
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#;
    let doc = parser::parse(input, "test.poentaro", &[]).unwrap();
    let score = group(doc).unwrap();
    assert!(score.measures[0].diagnostics.is_empty());
}

#[test]
fn explicit_no_lyrics_marker_produces_no_paired_syllables() {
    // `_` means "zero syllables for this measure" (syntax.md), so a measure
    // with real notes paired against `_` must not get any placeholder
    // syllables — otherwise it would render as a spurious empty lyric row.
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] _
    """,
"#;
    let doc = parser::parse(input, "test.poentaro", &[]).unwrap();
    let score = group(doc).unwrap();
    assert!(score.measures[0].diagnostics.is_empty());
    let melody = score.measures[0].parts[0].slice();
    assert!(
        melody.lyrics.iter().all(|verse| verse.syllables.is_empty()),
        "measure paired against `_` should have no syllables in any verse, got: {:?}",
        melody
            .lyrics
            .iter()
            .map(|l| &l.syllables)
            .collect::<Vec<_>>()
    );
}

#[test]
fn follow_part_with_no_lyric_line_gets_no_implicit_syllables() {
    // Neither `m` nor `h` ever writes a lyric line for this measure, so both
    // get the implicit `_` fill (desugar's `implicit_fill`), and `h` (which
    // `follow[m]`s) copies that resolved `_` through the follow chain. This
    // must not produce placeholder syllables for either part — this is the
    // exact path that caused an orphan empty lyric row to appear between
    // unrelated parts in a real score.
    let input = r#"metadata { title "t", author "a" },

parts ["Melody" [m]: notes, "Harmony" [h]: follow [m]],

score """
    time=4/4 key=C4 bpm=120
    [m] 1 2 3 4
    [h] 5 6 7 1
    """,
"#;
    let doc = parser::parse(input, "test.poentaro", &[]).unwrap();
    let score = group(doc).unwrap();
    let harmony = score.measures[0]
        .parts
        .iter()
        .find(|p| p.name().map(|n| n.as_str()) == Some("h"))
        .expect("Harmony part should exist")
        .slice();
    assert!(
        harmony
            .lyrics
            .iter()
            .all(|verse| verse.syllables.is_empty()),
        "follow part with no lyric line of its own should have no syllables, got: {:?}",
        harmony
            .lyrics
            .iter()
            .map(|l| &l.syllables)
            .collect::<Vec<_>>()
    );
}

#[test]
fn cross_measure_tilde_tie_closing_note_does_not_consume_syllable() {
    // 4~ ends measure 1; the 4 opening measure 2 is a tie continuation and
    // must not consume a lyric syllable. Measure 2 has notes 4 (tied), 5, 6, 7,
    // so "ha ko da" (3 syllables) is exactly sufficient — no underflow.
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4~
    [Melodyv1] la la la la

    [Melody] 4 5 6 7
    [Melodyv1] ha ko da
    """,
"#;
    let doc = parser::parse(input, "test.poentaro", &[]).unwrap();
    let score = group(doc).unwrap();
    assert_eq!(score.measures.len(), 2);
    assert!(
        score.measures[1].diagnostics.is_empty(),
        "measure 2 must have no underflow error, got: {:?}",
        score.measures[1].diagnostics
    );
}

#[test]
fn tie_state_does_not_leak_across_a_lyric_less_measure() {
    // Measure 1 ends on a genuine tie (`2~`). Measure 2 has no lyric line of
    // its own and does not tie into measure 3 — its last note (`3`) has no
    // `~`. Measure 3 has its own lyrics and should pair 1:1 against its own
    // notes (`4`->`c`, `5`->`d`), since the tie from measure 1 was already
    // resolved by measure 2's own (lyric-less) notes and must not leak
    // through untouched into measure 3.
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2~
    [Melodyv1] a b

    [Melody] 2 3

    [Melody] 4 5
    [Melodyv1] c d
    """,
"#;
    let doc = parser::parse(input, "test.poentaro", &[]).unwrap();
    let score = group(doc).unwrap();
    assert_eq!(score.measures.len(), 3);
    assert!(
        score.measures[2].diagnostics.is_empty(),
        "measure 3 must not report a lyrics overflow: the tie from measure 1 \
         is resolved by measure 2's own untied notes and must not leak \
         through the lyric-less measure 2, got: {:?}",
        score.measures[2].diagnostics
    );
}

#[test]
fn cross_measure_slur_note_consumes_syllable() {
    // (5 ... 5) is a slur, not a tie — the closing 5) still plays as its own
    // note and must consume a lyric syllable. Measure 2 has notes 5, 6, 7, 0 (rest),
    // so "hi ha ho" (3 syllables) is exactly sufficient — no underflow.
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 (5
    [Melodyv1] fa fo fi fu

    [Melody] 5) 6 7 0
    [Melodyv1] hi ha ho
    """,
"#;
    let doc = parser::parse(input, "test.poentaro", &[]).unwrap();
    let score = group(doc).unwrap();
    assert_eq!(score.measures.len(), 2);
    assert!(
        score.measures[1].diagnostics.is_empty(),
        "measure 2 must have no underflow error, got: {:?}",
        score.measures[1].diagnostics
    );
}
