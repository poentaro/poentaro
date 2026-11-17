//! `字(pinyin)` in lyric lines: tokenising, rendering and singing.

use crate::parser::lyrics::tokenize_bracketed_lyrics;
use crate::utils::tokenize_lyrics_with_errors;

fn readings(line: &str) -> Vec<(String, Option<String>)> {
    tokenize_lyrics_with_errors(line, 0)
        .syllables
        .into_iter()
        .map(|s| (s.text, s.pinyin.map(|p| p.marked())))
        .collect()
}

fn source(lyrics: &str) -> String {
    format!(
        "parts [\"Melody\" [M]: notes {{ sound: chimera soprano }}, \"Lyrics\" [L]: lyrics [M]],\n\
         score \"\"\"\ntime=4/4 key=C4 bpm=60\n[M] 1 2 3 4\n[L] {lyrics}\n\"\"\",\n"
    )
}

#[test]
fn a_reading_attached_to_a_character_is_part_of_its_one_syllable() {
    assert_eq!(
        readings("银行(háng) 长（zhang3）"),
        vec![
            ("银".to_string(), None),
            ("行".to_string(), Some("háng".to_string())),
            ("长".to_string(), Some("zhǎng".to_string())),
        ]
    );
}

#[test]
fn the_syllable_span_covers_only_the_character() {
    let tokens = tokenize_lyrics_with_errors("行(háng)", 10);
    assert_eq!(tokens.syllables[0].span, crate::error::Span::new(10, 13));
}

#[test]
fn text_that_is_not_pinyin_is_an_error_and_the_reading_is_ignored() {
    let tokens = tokenize_lyrics_with_errors("行(hello) 走", 0);
    assert_eq!(
        tokens
            .syllables
            .iter()
            .map(|s| (s.text.as_str(), s.pinyin.is_some()))
            .collect::<Vec<_>>(),
        vec![("行", false), ("走", false)]
    );
    assert_eq!(
        tokens
            .errors
            .iter()
            .map(|e| e.message())
            .collect::<Vec<_>>(),
        vec!["`hello` is not a pinyin syllable".to_string()]
    );
}

#[test]
fn parentheses_not_attached_to_a_chinese_character_stay_lyric_text() {
    assert_eq!(
        readings("(la) oh(hang) 行 (háng)"),
        vec![
            ("(la)".to_string(), None),
            ("oh(hang)".to_string(), None),
            ("行".to_string(), None),
            ("(háng)".to_string(), None),
        ]
    );
}

#[test]
fn under_a_chord_part_a_reading_is_not_a_bracketed_item() {
    let lyrics = tokenize_bracketed_lyrics("(~) 银行(háng) 我(陪)", 0);
    assert!(lyrics.errors.is_empty(), "{:?}", lyrics.errors);
    assert_eq!(
        lyrics
            .syllables
            .iter()
            .map(|s| (
                s.text.as_str(),
                s.bracketed,
                s.pinyin.as_ref().map(|p| p.marked())
            ))
            .collect::<Vec<_>>(),
        vec![
            ("~", true, None),
            ("银", false, None),
            ("行", false, Some("háng".to_string())),
            ("我", false, None),
            ("陪", true, None),
        ]
    );
}

#[test]
fn the_lyrics_notes_tally_counts_the_character_once() {
    let score =
        crate::compile(&source("去 银 行(hang2) 了"), "test.poentaro", &[]).expect("compiles");
    assert!(
        crate::collect_measure_diagnostics(&score).is_empty(),
        "{:?}",
        crate::collect_measure_diagnostics(&score)
    );
}

#[test]
fn an_invalid_reading_is_reported_for_the_document() {
    let score =
        crate::compile(&source("去 银 行(xyz) 了"), "test.poentaro", &[]).expect("compiles");
    let messages = crate::collect_measure_diagnostics(&score)
        .iter()
        .map(|d| format!("{d:?}"))
        .collect::<Vec<_>>();
    assert!(messages.iter().any(|m| m.contains("xyz")), "{messages:?}");
}

#[test]
fn the_reading_is_drawn_above_its_character_in_tone_mark_form() {
    let output = crate::render_svgs_from_source(&source("去 银 行(hang2) 了"), "t.poentaro", &[])
        .expect("renders");
    let svg = output.svgs.join("");
    assert!(svg.contains(">háng<"), "no ruby in the SVG");
    assert!(!svg.contains("hang2"));
}

#[cfg(feature = "midi")]
#[test]
fn the_singer_reads_the_character_with_the_given_reading() {
    let score =
        crate::compile(&source("去 银 行(háng) 了"), "test.poentaro", &[]).expect("compiles");
    use crate::sing::SungSyllable;
    let notes = crate::sing::sung_notes(&score, "M").expect("sung notes");
    assert_eq!(
        notes.iter().map(|n| n.syllable.clone()).collect::<Vec<_>>(),
        vec![
            SungSyllable::Word("去".to_string()),
            SungSyllable::Word("银".to_string()),
            SungSyllable::Pinyin("hang2".to_string()),
            SungSyllable::Word("了".to_string()),
        ]
    );
}
