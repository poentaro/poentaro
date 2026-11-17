//! Lyrics under a chord part (`lyrics[X]` where `X` is a `chords` part); see
//! "Lyrics under a chord part" in `syntax.md`. Each chord attack pairs with one
//! bracketed lyric item; unbracketed syllables flow between brackets.
//!
//! The tests that need layout/rendering stay `#[ignore]`d until it lands (tasks 5-6 of
//! `HANDOFF-chord-lyrics.md`): the pre-commit hook runs `cargo test
//! --workspace`, so failing tests cannot be committed un-ignored. Un-ignore
//! them as the feature is implemented; run with `cargo test chord_lyrics --
//! --ignored` to see them fail/pass meanwhile.
#![allow(clippy::disallowed_macros, clippy::panic)]
use poentaro::error::Diagnostic;
use poentaro::render_svgs_from_source;

fn source(chords: &str, lyrics: &str) -> String {
    format!(
        r#"metadata {{ title "t", author "a" }},

parts ["c": chords, "l": lyrics [c]],

score """
    time=4/4 key=C4 bpm=120
    [c] {chords}
    [l] {lyrics}
    """,
"#
    )
}

fn render(chords: &str, lyrics: &str) -> poentaro::RenderOutput {
    render_svgs_from_source(&source(chords, lyrics), "test.poentaro", &[]).unwrap_or_else(|err| {
        panic!(
            "a chord-lyrics mistake must be a diagnostic, not an abort: {}",
            err.message()
        )
    })
}

fn messages(output: &poentaro::RenderOutput) -> Vec<String> {
    output.diagnostics.iter().map(Diagnostic::message).collect()
}

fn error_count(output: &poentaro::RenderOutput) -> usize {
    output
        .diagnostics
        .iter()
        .filter(|d| matches!(d, Diagnostic::Error(_)))
        .count()
}

fn assert_no_diagnostics(output: &poentaro::RenderOutput) {
    assert!(
        output.diagnostics.is_empty(),
        "expected no diagnostics, got: {:?}",
        messages(output)
    );
}

fn assert_one_error(output: &poentaro::RenderOutput) {
    assert_eq!(
        error_count(output),
        1,
        "expected exactly one error diagnostic, got: {:?}",
        messages(output)
    );
    // The message must explain the bracket problem (the wording "bracket"
    // is the contract for tasks 3-4), not merely be some unrelated error.
    assert!(
        messages(output)
            .iter()
            .any(|m| m.to_lowercase().contains("bracket")),
        "error should mention brackets, got: {:?}",
        messages(output)
    );
}

#[test]
fn happy_path_compiles_cleanly_and_renders_brackets_as_ascii() {
    let output = render("1 -- 7", "(~) 只剩下鋼琴 （陪）我");
    assert_no_diagnostics(&output);
    let svg = output.svgs.concat();
    assert!(svg.contains("(~)"), "bracket must be drawn");
    assert!(svg.contains("(陪)"), "full-width bracket must render ASCII");
    assert!(!svg.contains('（') && !svg.contains('）'));
}

#[test]
fn ascii_brackets_are_accepted() {
    let output = render("1 -- 7", "(~) 只剩下鋼琴 (陪) 我");
    assert_no_diagnostics(&output);
    assert!(output.svgs.concat().contains("(陪)"));
}

#[test]
fn unbracketed_run_before_first_bracket_is_allowed_when_first_attack_is_not_on_beat_one() {
    let output = render("0 1 - -", "我 (愛) 你");
    assert_no_diagnostics(&output);
}

#[test]
fn fewer_brackets_than_chord_attacks_is_allowed() {
    // Two attacks (1 and 4) but one bracket.
    assert_no_diagnostics(&render("1 - 4 -", "(啦) 啦啦"));
}

#[test]
fn error_when_brackets_outnumber_chord_attacks() {
    // Two attacks (1 and 4) but three brackets.
    assert_one_error(&render("1 - 4 -", "(a) (b) (c)"));
}

#[test]
fn error_when_a_bracket_holds_several_syllables() {
    assert_one_error(&render("1 - - -", "（陪我）"));
}

#[test]
fn error_when_unbracketed_run_precedes_a_beat_one_attack() {
    assert_one_error(&render("1 - - -", "啦 (啦)"));
}

#[test]
fn chord_rest_is_not_an_attack_so_extra_bracket_is_an_error() {
    // Attacks: 1 and 4 (the `0` rest is not one); three brackets is too many.
    assert_one_error(&render("1 0 4 -", "(a) (b) (c)"));
}

#[test]
fn chord_rest_is_not_an_attack_so_matching_brackets_are_fine() {
    assert_no_diagnostics(&render("1 0 4 -", "(a) (b)"));
}

#[test]
fn every_syllable_gets_a_source_span_and_a_distinct_note_id() {
    let spans = poentaro::lyric_spans::list_lyric_spans_from_source(
        &source("1 - 4 -", "(~) 只剩下 (陪) 我"),
        "test.poentaro",
        None,
    )
    .unwrap_or_else(|err| panic!("{}", err.message()))
    .spans;
    assert_eq!(spans.len(), 6);
    let mut ids: Vec<usize> = spans.iter().map(|span| span.note_id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 6, "click targets are keyed by note id");
}

#[test]
fn bracketed_syllables_share_their_chords_note_ids() {
    let spans = poentaro::lyric_spans::list_lyric_spans_from_source(
        &source("1 - 4 -", "(啦) 啦 (啦)"),
        "test.poentaro",
        None,
    )
    .unwrap_or_else(|err| panic!("{}", err.message()))
    .spans;
    let bracketed_ids: Vec<usize> = spans.iter().map(|span| span.note_id).collect();
    // Chord events are note ids 0 and 1; the middle syllable is floating.
    assert_eq!(bracketed_ids.first(), Some(&0));
    assert_eq!(bracketed_ids.last(), Some(&1));
}

#[test]
fn lyrics_with_many_syllables_between_chords_compile_cleanly() {
    let output = render("1 - 4 -", "(a) b c d e f g (h) i j");
    assert_no_diagnostics(&output);
}
