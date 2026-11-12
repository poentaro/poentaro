use super::*;
use crate::ast::parsed::{PartKind, Soundfont};

pub(super) fn decl(name: &str, kind: PartKind) -> PartDecl {
    PartDecl {
        abbreviation: name.to_string(),
        abbreviation_span: Span::new(0, 0),
        display_name: name.to_string(),
        kind,
        follow_target: None,
        range_instrument: None,
        written_range: None,
        singing_voice: None,
        soundfont: Soundfont::CHOIR_AAHS,
        volume: 100,
        octave_offset: 0,
    }
}

/// A `notes` part named `name` followed by one lyric part per entry of `verses`.
fn with_verses(name: &str, verses: &[&str]) -> Vec<PartDecl> {
    std::iter::once(decl(name, PartKind::Notes))
        .chain(verses.iter().map(|abbreviation| {
            decl(
                abbreviation,
                PartKind::Lyrics {
                    target_part_index: 0,
                },
            )
        }))
        .collect()
}

fn decl_follow(name: &str, kind: PartKind, target: &str) -> PartDecl {
    PartDecl {
        abbreviation: name.to_string(),
        abbreviation_span: Span::new(0, 0),
        display_name: name.to_string(),
        kind,
        follow_target: Some(target.to_string()),
        range_instrument: None,
        written_range: None,
        singing_voice: None,
        soundfont: Soundfont::CHOIR_AAHS,
        volume: 100,
        octave_offset: 0,
    }
}

pub(super) fn group(lines: &[&str]) -> Vec<(String, usize)> {
    lines
        .iter()
        .enumerate()
        .map(|(i, l)| (l.to_string(), i * 10))
        .collect()
}

#[test]
fn abbreviation_reference_span_covers_only_trimmed_key_text() {
    let groups = vec![group(&["[ A ] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::Notes)];
    let (_result, _slots, _errors, refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].abbreviation, "A");
    // "[ A ] 1 2 3 4" -> `A` starts at byte 2, right after `[ `.
    assert_eq!(refs[0].span.start, 2);
    assert_eq!(refs[0].span.end, 3);
}

#[test]
fn score_lines_are_passed_through_unchanged() {
    let groups = vec![group(&["[A] 1 2 3 4", "[v1] hello"])];
    let declarations = with_verses("A", &["v1"]);
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4");
    assert_eq!(result[0][1].content, "hello");
}

#[test]
fn omitted_trailing_notes_without_precedent_fills_with_rest_silently() {
    let groups = vec![group(&["[A] 1 - - -"])];
    let declarations = vec![decl("A", PartKind::Chords), decl("B", PartKind::Notes)];
    let (result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(
        result[0][1].content, "0 0 0 0",
        "should fill in quarter-rest placeholder for all 4 beats"
    );
    assert!(
        errors[0].is_none(),
        "omitted notes with no precedent should not produce an error"
    );
}

#[test]
fn omitted_trailing_chord_without_precedent_fills_with_rest_silently() {
    let groups = vec![group(&["[A] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::Notes), decl("B", PartKind::Chords)];
    let (result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(
        result[0][1].content, "0 0 0 0",
        "should fill in chord-rest placeholder for all 4 beats"
    );
    assert!(
        errors[0].is_none(),
        "omitted chord with no precedent should not produce an error"
    );
}

// --- [Key] prefix tests ---

#[test]
fn key_prefix_only_c_plays_others_fill_implicitly() {
    let groups = vec![group(&["[C] 5 6 7 0"])];
    let declarations = vec![
        decl("A", PartKind::Notes),
        decl("B", PartKind::Notes),
        decl("C", PartKind::Notes),
    ];
    let (result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].content, "0 0 0 0", "A: no precedent → rest");
    assert_eq!(result[0][1].content, "0 0 0 0", "B: no precedent → rest");
    assert_eq!(result[0][2].content, "5 6 7 0", "C: explicit content");
    assert!(errors[0].is_none());
}

#[test]
fn key_prefix_unknown_abbreviation_is_recoverable_error() {
    let groups = vec![group(&["[Z] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::Notes)];
    let (result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].content, "0 0 0 0");
    let err = errors[0]
        .as_ref()
        .expect("should produce a recoverable error");
    assert!(err.message().contains("[Z]"), "got: {}", err.message());
    assert!(
        err.message().contains("abbreviation"),
        "got: {}",
        err.message()
    );
    assert_eq!(err.span.start, 0, "span must start at `[`");
    assert_eq!(err.span.end, "[Z]".len(), "span must cover `[Z]`");
}

// --- follow[X] tests ---

#[test]
fn follow_with_no_key_override_copies_target_content() {
    let groups = vec![group(&["[A] 1 2 3 4"])];
    let declarations = vec![
        decl("A", PartKind::Notes),
        decl_follow("B", PartKind::Notes, "A"),
    ];
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4", "A: explicit content");
    assert_eq!(
        result[0][1].content, "1 2 3 4",
        "B: copied from A via follow"
    );
}

#[test]
fn follow_with_key_override_uses_key_content() {
    let groups = vec![group(&["[A] 1 2 3 4", "[B] 5 6 7 0"])];
    let declarations = vec![
        decl("A", PartKind::Notes),
        decl_follow("B", PartKind::Notes, "A"),
    ];
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4", "A: key-prefixed");
    assert_eq!(
        result[0][1].content, "5 6 7 0",
        "B: key override takes precedence over follow"
    );
}

#[test]
fn follow_chain_resolves_correctly() {
    // C follows B, B follows A.
    let groups = vec![group(&["[A] 1 2 3 4"])];
    let declarations = vec![
        decl("A", PartKind::Notes),
        decl_follow("B", PartKind::Notes, "A"),
        decl_follow("C", PartKind::Notes, "B"),
    ];
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4", "A: explicit");
    assert_eq!(result[0][1].content, "1 2 3 4", "B: copied from A");
    assert_eq!(
        result[0][2].content, "1 2 3 4",
        "C: copied from B (which has A content)"
    );
}

#[test]
fn non_follow_non_first_part_not_mentioned_fills_with_rest() {
    let groups = vec![group(&["[A] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::Notes), decl("B", PartKind::Notes)];
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4", "A: explicit");
    assert_eq!(
        result[0][1].content, "0 0 0 0",
        "B: no follow target, not mentioned → rest for all 4 beats"
    );
}

#[test]
fn non_follow_part_with_key_line_uses_key_content() {
    let groups = vec![group(&["[A] 1 2 3 4", "[B] 5 6 7 0"])];
    let declarations = vec![decl("A", PartKind::Notes), decl("B", PartKind::Notes)];
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4", "A: key-prefixed");
    assert_eq!(result[0][1].content, "5 6 7 0", "B: key-based explicit");
}

// --- Lyric part (`lyrics[X]`) tests ---

#[test]
fn lyric_part_line_is_routed_to_its_own_slot() {
    let groups = vec![group(&["[A] 1 2 3 4", "[v1] la la la la"])];
    let declarations = with_verses("A", &["v1"]);
    let (result, slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(slots[0].len(), 2);
    assert_eq!(slots[0][1].role, ScoreLineRole::Lyrics);
    assert_eq!(slots[0][1].track_index, 1);
    assert_eq!(result[0][0].content, "1 2 3 4");
    assert_eq!(result[0][1].content, "la la la la");
    assert!(errors[0].is_none());
}

#[test]
fn verse_lines_follow_declaration_order_not_score_order() {
    let groups = vec![group(&[
        "[A] 1 2 3 4",
        "[v2] two two two two",
        "[v1] one one one one",
    ])];
    let declarations = with_verses("A", &["v1", "v2"]);
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][1].content, "one one one one", "verse 1");
    assert_eq!(result[0][2].content, "two two two two", "verse 2");
}

#[test]
fn skipped_verse_is_filled_with_no_lyrics() {
    let groups = vec![group(&["[A] 1 2 3 4", "[v2] two two two two"])];
    let declarations = with_verses("A", &["v1", "v2"]);
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][1].content, "_", "verse 1 skipped");
    assert_eq!(result[0][2].content, "two two two two");
}

#[test]
fn lyric_part_with_no_line_is_filled_with_no_lyrics() {
    let groups = vec![group(&["[A] 1 2 3 4"])];
    let declarations = with_verses("A", &["v1"]);
    let (result, slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0].len(), 2);
    assert_eq!(slots[0].len(), 2);
    assert_eq!(result[0][1].content, "_");
    assert!(result[0][1].is_implicit_fill);
}

#[test]
fn verse_line_without_target_notes_line_fills_target_with_rests() {
    let groups = vec![group(&["[v1] la la la la"])];
    let declarations = with_verses("A", &["v1"]);
    let (result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].content, "0 0 0 0");
    assert_eq!(result[0][1].content, "la la la la");
    assert!(errors[0].is_none());
}

#[test]
fn follow_part_does_not_copy_its_targets_verses() {
    let groups = vec![group(&["[A] 1 2 3 4", "[v1] la la la la"])];
    let declarations: Vec<PartDecl> = with_verses("A", &["v1"])
        .into_iter()
        .chain([decl_follow("B", PartKind::Notes, "A")])
        .collect();
    let (result, slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(slots[0].len(), 3);
    assert_eq!(result[0][2].content, "1 2 3 4", "B copies notes only");
}

#[test]
fn bare_line_is_a_missing_key_prefix_error() {
    let groups = vec![group(&["[A] 1 2 3 4", "la la la la"])];
    let declarations = vec![decl("A", PartKind::Notes)];
    let (_result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    let err = errors[0].as_ref().expect("bare line should error");
    assert_eq!(
        err.kind,
        crate::error::RecoverableErrorKind::ScoreLineMissingKeyPrefix
    );
}

#[test]
fn verse_key_is_an_abbreviation_reference() {
    let groups = vec![group(&["[A] 1 2 3 4", "[v1] la la la la"])];
    let declarations = with_verses("A", &["v1"]);
    let (_result, _slots, _errors, refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[1].abbreviation, "v1");
}

#[test]
fn duplicated_verse_line_errors() {
    let groups = vec![group(&["[A] 1 2 3 4", "[v1] a b c d", "[v1] e f g h"])];
    let declarations = with_verses("A", &["v1"]);
    let (_result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert!(errors[0].is_some());
}

#[test]
fn genuinely_duplicated_key_prefix_on_plain_notes_part_still_errors() {
    // Two real `[A]`-prefixed lines under a plain `notes` part (1 slot) is
    // still a capacity error, unlike a real line plus a positionally-attached
    // bare line (which is unlimited).
    let groups = vec![group(&["[A] 1 2 3 4", "[A] 5 6 7 0"])];
    let declarations = vec![decl("A", PartKind::Notes)];
    let (_result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert!(
        errors[0].is_some(),
        "duplicate [A]-prefixed lines should still trip the fixed-schema capacity check"
    );
}

// Group-broadcast desugaring tests (slot filling, member overrides, and
// `group` provenance tagging) live in `tests_groups.rs`.
