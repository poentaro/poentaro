use crate::compiler::{compile, types::*};
use crate::grouper::group;
use crate::parser::parse;

fn score_from(source: &str) -> crate::ast::grouped::Score {
    let doc = parse(source, "test", &[]).unwrap();
    group(doc).unwrap()
}

/// Lyrics-part document with one track.
fn lyrics_doc(score_content: &str) -> String {
    format!(
        r#"metadata {{ title "t", author "a" }},

parts ["S": notes, "Sv": lyrics [S]],

score """
    {score_content}
    """,
"#
    )
}

#[test]
fn cross_measure_tilde_tie_does_not_consume_lyric_slot_for_continuation_note() {
    // Bar 1: "1 2 3 4~" has 4 lyric slots → "ha ta ba na"
    // Bar 2: "4 5 6 7" → note 4 is a tie continuation, only 3 lyric slots → "sa da ko"
    // "sa" must be assigned to note 5 (column 4), not the tied note 4 (column 0).
    let score = score_from(&lyrics_doc(concat!(
        "time=4/4 key=C4 bpm=120\n",
        "[S] 1 2 3 4~\n",
        "[Sv] ha ta ba na\n",
        "\n",
        "[S] 4 5 6 7\n",
        "[Sv] sa da ko\n",
    )));
    let result = compile(&score);
    let blocks = result.blocks;
    let bar2 = &blocks[1].rows[1];
    // "sa" should be at column 4 (note 5, after the tied note 4 at column 0)
    let lyrics: Vec<_> = bar2
        .elements
        .iter()
        .filter_map(|e| {
            if let ElementContent::Lyric { text, .. } = &e.content {
                Some((e.column, text.as_str()))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        lyrics,
        vec![(4, "sa"), (8, "da"), (12, "ko")],
        "lyrics should be assigned to notes 5, 6, 7 (columns 4, 8, 12), not to the tied continuation note 4"
    );
}

#[test]
fn lyrics_underflow_errors_propagate_to_measure_block() {
    // 4 notes but only 2 syllables → block should have errors
    let source = lyrics_doc("time=4/4 key=C4 bpm=120\n[S] 1 2 3 4\n[Sv] a b\n");
    let score = score_from(&source);
    let result = compile(&score);
    assert_eq!(result.blocks.len(), 1);
    assert_eq!(result.blocks[0].diagnostics.len(), 1);
    assert!(result.blocks[0].diagnostics[0]
        .message()
        .contains("underflow"));
}

#[test]
fn matching_lyrics_produce_no_block_errors() {
    let source = lyrics_doc("time=4/4 key=C4 bpm=120\n[S] 1 2 3 4\n[Sv] a b c d\n");
    let score = score_from(&source);
    let result = compile(&score);
    assert!(result.blocks[0].diagnostics.is_empty());
}

#[test]
fn lyrics_underflow_in_first_measure_only() {
    // Measure 1: 4 notes but only 2 syllables → underflow
    // Measure 2: 4 notes and 4 syllables → no error
    let source = lyrics_doc(concat!(
        "time=4/4 key=C4 bpm=120\n",
        "[S] 1 2 3 4\n",
        "[Sv] a b\n",
        "\n",
        "[S] 5 6 7 1\n",
        "[Sv] c d e f\n",
    ));
    let score = score_from(&source);
    let result = compile(&score);
    assert_eq!(result.blocks.len(), 2);
    assert_eq!(result.blocks[0].diagnostics.len(), 1);
    assert!(result.blocks[0].diagnostics[0]
        .message()
        .contains("underflow"));
    assert!(result.blocks[1].diagnostics.is_empty());
}

#[test]
fn malformed_parts_line_is_recoverable_and_valid_part_still_renders() {
    use crate::error::RecoverableErrorKind;

    let source = r#"metadata { title "t", author "a" },

parts [ no_kind, "Melody" : notes ],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    """,
"#;
    let doc = parse(source, "test", &[]).expect("malformed parts line must not abort parsing");
    assert_eq!(doc.declarations.len(), 1, "valid declaration must survive");
    assert_eq!(doc.declarations[0].abbreviation, "Melody");
    assert_eq!(doc.parts_parse_errors.len(), 1);
    assert!(
        matches!(
            doc.parts_parse_errors[0].kind,
            RecoverableErrorKind::AnyfixInvalid { .. }
        ),
        "expected AnyfixInvalid error, got: {:?}",
        doc.parts_parse_errors[0].kind
    );
    let doc_error = doc.parts_parse_errors[0].message();
    let score = group(doc).unwrap();
    assert!(
        score
            .document_diagnostics
            .iter()
            .any(|d| d.message() == doc_error),
        "malformed-part error must appear in document_diagnostics"
    );
}

#[test]
fn all_parts_invalid_renders_empty_document_with_error() {
    use crate::error::RecoverableErrorKind;

    let source = r#"metadata { title "t", author "a" },

parts [],

score """
    time=4/4 key=C4 bpm=120
    1 2 3 4
    """,
"#;
    let doc = parse(source, "test", &[]).expect("all-invalid parts must not abort parsing");
    assert!(
        doc.declarations.is_empty(),
        "no valid declarations expected"
    );
    assert!(
        doc.parts_parse_errors
            .iter()
            .any(|e| matches!(e.kind, RecoverableErrorKind::PartsEmptySection)),
        "PartsEmptySection error must be collected"
    );
    let score = group(doc).unwrap();
    assert!(
        score
            .document_diagnostics
            .iter()
            .any(|d| d.message().contains("at least one part")),
        "empty-section error must appear in document_diagnostics"
    );
}

#[test]
fn tie_into_a_measure_without_lyrics_does_not_swallow_a_later_syllable() {
    // Bar 1's tie ends in bar 2, which has no lyrics (its lyric row is hidden).
    // Bar 3 must start fresh: "A" goes on its first note, not its second.
    let score = score_from(&lyrics_doc(concat!(
        "time=4/4 key=C4 bpm=120\n",
        "[S] 6,~---\n",
        "[Sv] a\n",
        "\n",
        "[S] 6, 5 5 5\n",
        "\n",
        "[S] 6, 3_2_ 1 6,\n",
        "[Sv] A B C D E\n",
    )));
    let result = compile(&score);
    let bar3 = &result.blocks[2];
    let lyrics: Vec<_> = bar3.rows[1]
        .elements
        .iter()
        .filter_map(|e| match &e.content {
            ElementContent::Lyric { text, note_id } => Some((e.column, text.as_str(), *note_id)),
            _ => None,
        })
        .collect();
    // Bar 1 holds note 0, bar 2 notes 1..=4, so bar 3 starts at note 5.
    assert_eq!(
        lyrics,
        vec![
            (0, "A", 5),
            (4, "B", 6),
            (6, "C", 7),
            (8, "D", 8),
            (12, "E", 9)
        ]
    );
}
