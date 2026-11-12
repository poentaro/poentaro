//! `sequence [Verse WITH ([w] 2, [M] 2)]` plays the parts' second lines in
//! that occurrence, and a part's alternative lines play nowhere else.

use crate::midi::expand_navigation;
use crate::sing::{sung_notes, SungSyllable};

const SCORE: &str = "time=2/4 key=C4 bpm=60 label=\"Verse\"\n[M] 1 2\n[M] 3 4\n[w] 一 二\n[w] 三 四\n\n[M] 5 6\n[w] 五 六\n";

fn sung(sequence: &str) -> Vec<(u8, String)> {
    let source = format!(
        "parts [\"Melody\" [M]: notes {{ sound: chimera soprano }}, \"Words\" [w]: lyrics [M]],\n{sequence}score \"\"\"\n{SCORE}\"\"\",\n"
    );
    let score = crate::compile(&source, "test.poentaro", &[]).expect("compiles");
    let played = expand_navigation(&score).expect("expands");
    sung_notes(&played, "M")
        .expect("sung notes")
        .into_iter()
        .map(|note| {
            let syllable = match note.syllable {
                SungSyllable::Word(word) => word,
                other => format!("{other:?}"),
            };
            (note.midi, syllable)
        })
        .collect()
}

fn pairs(expected: &[(u8, &str)]) -> Vec<(u8, String)> {
    expected
        .iter()
        .map(|(midi, syllable)| (*midi, syllable.to_string()))
        .collect()
}

#[test]
fn without_a_sequence_each_part_plays_its_first_line() {
    assert_eq!(
        sung(""),
        pairs(&[(60, "一"), (62, "二"), (67, "五"), (69, "六")])
    );
}

#[test]
fn a_chosen_line_plays_where_written_and_the_first_line_elsewhere() {
    assert_eq!(
        sung("sequence [Verse, Verse WITH ([w] 2, [M] 2)],\n"),
        pairs(&[
            (60, "一"),
            (62, "二"),
            (67, "五"),
            (69, "六"),
            (64, "三"),
            (65, "四"),
            (67, "五"),
            (69, "六"),
        ])
    );
}

#[test]
fn a_line_the_part_never_writes_falls_back_to_its_first_line() {
    let source = format!(
        "parts [\"Melody\" [M]: notes, \"Words\" [w]: lyrics [M]],\nsequence [Verse WITH [w] 3],\nscore \"\"\"\n{SCORE}\"\"\",\n"
    );
    let score = crate::compile(&source, "test.poentaro", &[]).expect("compiles");
    assert!(score.document_diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        crate::error::Diagnostic::Warning(warning)
            if warning.message == "part [w] has no line 3 in any measure; playing its line 1"
    )));
    let spans = score.sequence.expect("sequence");
    assert!(spans[0].chosen_lines.is_empty());
}
