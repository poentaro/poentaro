use crate::grouper::tests::parse_and_group;

fn source_with(body: &str, sequence: &str) -> String {
    format!(
        r#"metadata {{ title "t", author "a" }},

parts ["Melody": notes],

sequence [{sequence}],

score """
    {body}
    """,
"#
    )
}

fn source_with_voices(body: &str, sequence: &str) -> String {
    format!(
        r#"metadata {{ title "t", author "a" }},

parts ["Soprano" [S]: notes, "Alto 2" [A2]: notes, "Tenor" [T]: notes],

sequence [{sequence}],

score """
    {body}
    """,
"#
    )
}

fn voices_body() -> &'static str {
    concat!(
        "time=4/4 key=C4 bpm=120 label=\"Verse\"\n[S] 1 2 3 4\n[A2] 1 2 3 4\n[T] 1 2 3 4\n\n",
        "label=\"Chorus\"\n[S] 5 6 7 1\n[A2] 5 6 7 1\n[T] 5 6 7 1\n",
    )
}

fn labeled_body() -> &'static str {
    concat!(
        "time=4/4 key=C4 bpm=120 label=\"A\"\n[Melody] 1 2 3 4\n\n",
        "label=\"B\"\n[Melody] 5 6 7 1\n\n",
        "[Melody] 2 3 4 5\n\n", // still part of B's span: no label on this measure
        "label=\"C\"\n[Melody] 1 1 1 1\n",
    )
}

fn all_error_messages(score: &crate::ast::grouped::Score) -> Vec<String> {
    score
        .document_diagnostics
        .iter()
        .map(|d| d.message())
        .chain(
            score
                .measures
                .iter()
                .flat_map(|m| m.diagnostics.iter().map(|d| d.message())),
        )
        .collect()
}

#[test]
fn resolves_spans_from_labels_including_eof_terminated_last_span() {
    let score = parse_and_group(&source_with(labeled_body(), "A, B, C"));
    let sequence = score.sequence.expect("expected a resolved sequence");
    assert_eq!(sequence.len(), 3);
    assert_eq!(sequence[0].label, "A");
    assert_eq!(sequence[0].start, 0);
    assert_eq!(sequence[0].end, 0);
    assert_eq!(sequence[1].label, "B");
    assert_eq!(sequence[1].start, 1);
    assert_eq!(sequence[1].end, 2); // spans the unlabeled measure too
    assert_eq!(sequence[2].label, "C");
    assert_eq!(sequence[2].start, 3);
    assert_eq!(sequence[2].end, 3); // last span runs to EOF
}

#[test]
fn sequence_can_repeat_a_label() {
    let score = parse_and_group(&source_with(labeled_body(), "A, B, A, C"));
    let sequence = score.sequence.expect("expected a resolved sequence");
    let labels: Vec<&str> = sequence.iter().map(|s| s.label.as_str()).collect();
    assert_eq!(labels, vec!["A", "B", "A", "C"]);
}

#[test]
fn no_sequence_section_leaves_score_sequence_none() {
    let score = parse_and_group(
        r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120 label="A"
    [Melody] 1 2 3 4
    """,
"#,
    );
    assert!(score.sequence.is_none());
}

#[test]
fn duplicate_label_definition_is_a_recoverable_error() {
    let body = concat!(
        "time=4/4 key=C4 bpm=120 label=\"A\"\n[Melody] 1 2 3 4\n\n",
        "label=\"A\"\n[Melody] 5 6 7 1\n",
    );
    let score = parse_and_group(&source_with(body, "A"));
    assert!(score.sequence.is_none());
    let messages = all_error_messages(&score);
    assert!(
        messages
            .iter()
            .any(|m| m.contains("defined more than once")),
        "expected a duplicate-label error, got: {messages:?}"
    );
}

#[test]
fn undefined_label_reference_is_a_recoverable_error() {
    let score = parse_and_group(&source_with(labeled_body(), "A, Z"));
    let messages = all_error_messages(&score);
    let sequence = score.sequence.expect("expected a resolved sequence");
    // The bad entry is skipped, the rest of the sequence still resolves.
    assert_eq!(sequence.len(), 1);
    assert_eq!(sequence[0].label, "A");
    assert!(
        messages.iter().any(|m| m.contains("undefined label \"Z\"")),
        "expected an undefined-label error, got: {messages:?}"
    );
}

#[test]
fn part_omission_suffix_resolves_to_omit_parts_on_the_span() {
    let score = parse_and_group(&source_with_voices(
        voices_body(),
        "Verse, Chorus WITHOUT [S, A2], Verse, Chorus WITHOUT [A2]",
    ));
    let sequence = score.sequence.expect("expected a resolved sequence");
    assert_eq!(sequence.len(), 4);
    assert!(sequence[0].omit_parts.is_empty()); // Verse
    assert_eq!(sequence[1].omit_parts, vec!["S", "A2"]); // Chorus WITHOUT [S, A2]
    assert!(sequence[2].omit_parts.is_empty()); // Verse
    assert_eq!(sequence[3].omit_parts, vec!["A2"]); // Chorus WITHOUT [A2]
}

#[test]
fn part_omission_suffix_with_unknown_abbreviation_is_a_recoverable_error() {
    let score = parse_and_group(&source_with_voices(voices_body(), "Chorus WITHOUT [Bogus]"));
    let messages = all_error_messages(&score);
    let sequence = score.sequence.expect("expected a resolved sequence");
    assert!(sequence[0].omit_parts.is_empty());
    assert!(
        messages
            .iter()
            .any(|m| m.contains("omits unknown part \"Bogus\"")),
        "expected an unknown-omission error, got: {messages:?}"
    );
}

#[test]
fn part_only_suffix_resolves_to_the_complement_omit_parts_on_the_span() {
    let score = parse_and_group(&source_with_voices(voices_body(), "Verse, Chorus ONLY [S]"));
    let sequence = score.sequence.expect("expected a resolved sequence");
    assert_eq!(sequence.len(), 2);
    assert!(sequence[0].omit_parts.is_empty()); // Verse
    assert_eq!(sequence[1].omit_parts, vec!["A2", "T"]); // Chorus ONLY [S] keeps only S
}

#[test]
fn part_only_suffix_naming_every_declared_part_omits_nothing() {
    let score = parse_and_group(&source_with_voices(voices_body(), "Chorus ONLY [S, A2, T]"));
    let sequence = score.sequence.expect("expected a resolved sequence");
    assert!(sequence[0].omit_parts.is_empty());
}

#[test]
fn part_only_suffix_with_unknown_abbreviation_is_a_recoverable_error() {
    let score = parse_and_group(&source_with_voices(voices_body(), "Chorus ONLY [Bogus]"));
    let messages = all_error_messages(&score);
    assert!(
        messages
            .iter()
            .any(|m| m.contains("keeps unknown part \"Bogus\"")),
        "expected an unknown-only error, got: {messages:?}"
    );
}
