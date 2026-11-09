use crate::ast::parsed::{
    Accidental, NumberedPitch, ParsedChordNote, PartKind, ScoreEvent, TriadQuality,
};

use super::test_helpers::{
    all_events, chord_track, decl, decls_with_verse, lyric_decl, notes_track, parse,
    parse_recoverable_errors, total_lyrics_syllables,
};

#[test]
fn chord_line_parses_spaced_slur_group() {
    let input = r#"metadata { title "t", author "a" },

parts ["Chord": chords, "Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Chord] (1 - 6m -)
    [Melody] 1 1 5 5
    """,
"#;
    let doc = crate::parser::parse(input, "test.poentaro", &[]).unwrap();
    let chord_events: Vec<_> = all_events(chord_track(&doc.tracks, "Chord"))
        .into_iter()
        .filter(|e| matches!(e.value, ScoreEvent::Chord(_)))
        .collect();
    assert_eq!(chord_events.len(), 2, "expected chord 1 and 6m in group");
}

#[test]
fn chord_column_events_are_parsed() {
    let declarations = vec![decl("C", PartKind::Chords), decl("N", PartKind::Notes)];
    let content = "time=4/4 key=C4 bpm=120\n[C] 1 - - -\n[N] 1---\n";
    let tracks = parse(content, 0, &declarations).unwrap();
    assert_eq!(tracks.len(), 2);
    let chord = chord_track(&tracks, "C");
    let events: Vec<_> = all_events(chord).into_iter().map(|e| &e.value).collect();
    // events[0..3] are directive events (TimeSignatureChange, KeyChange, BpmChange)
    // broadcast to all tracks; the first chord note follows at index 3.
    assert_eq!(
        events[3],
        &ScoreEvent::Chord(ParsedChordNote {
            degree: NumberedPitch::One,
            accidental: Accidental::Natural,
            triad: TriadQuality::Major,
            extension: None,
            bass: None,
            duration: 4,
            slur: false,
            tie_to_next_span: None,
            group_membership: 0,
            group_continuation: 0,
            dotted: false,
            double_dotted: false,
            slur_group_close_at_duration: None,
            tuplet: None,
        })
    );
    assert!(matches!(events[4], ScoreEvent::Extension { .. }));
    assert_eq!(all_events(notes_track(&tracks, "N")).len(), 4);
}

#[test]
fn single_unnamed_part_no_lyrics() {
    let content = "time=4/4 key=C4 bpm=120\n[] 1 2 3 4\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).unwrap();
    assert_eq!(tracks.len(), 1);
    let notes = notes_track(&tracks, "");
    // Only a lyric part carries lyrics.
    assert!(notes.lyrics.is_none());
    assert_eq!(all_events(notes).len(), 7);
}

#[test]
fn lyric_part_line_parses_into_its_own_track() {
    let content = "time=4/4 key=C4 bpm=120\n[] 1 2 3 4\n[v] do re mi fa\n";
    let declarations = decls_with_verse("", "v");
    let tracks = parse(content, 0, &declarations).unwrap();
    assert_eq!(tracks.len(), 2);
    assert!(notes_track(&tracks, "").lyrics.is_none());
    let verse = notes_track(&tracks, "v");
    assert!(verse.measure_slots.is_empty());
    assert_eq!(verse.lyrics.as_ref().unwrap().measure_syllables[0].len(), 4);
    assert_eq!(total_lyrics_syllables(verse), 4);
}

#[test]
fn two_parts_two_bars() {
    let content = concat!(
        "time=4/4 key=C4 bpm=120\n",
        "[Soprano] 1 2 3 4\n",
        "[Alto] 5 6 7 1\n",
        "\n",
        "[Soprano] 1 2 3 4\n",
        "[Alto] 5 6 7 1\n",
    );
    let declarations = vec![
        decl("Soprano", PartKind::Notes),
        decl("Alto", PartKind::Notes),
    ];
    let tracks = parse(content, 0, &declarations).unwrap();
    assert_eq!(tracks.len(), 2);
    assert_eq!(all_events(notes_track(&tracks, "Soprano")).len(), 11);
    // Alto now receives the 3 directive events from bar 1, same as Soprano.
    assert_eq!(all_events(notes_track(&tracks, "Alto")).len(), 11);
}

#[test]
fn too_many_lines_in_group_is_recoverable() {
    // Extra data line beyond what the declared parts expect must not abort parsing.
    let content = "time=4/4 key=C4 bpm=120\n1 2 3 4\na b c d\nextra line\n";
    let declarations = vec![decl("", PartKind::Notes)];
    assert!(
        parse(content, 0, &declarations).is_ok(),
        "extra data lines must not abort parsing"
    );
}

#[test]
fn underscore_on_lyrics_line_means_no_lyrics_for_that_bar() {
    let content = concat!(
        "time=4/4 key=C4 bpm=120\n[] 1 2 3 4\n[v] a b c d\n",
        "\n",
        "[] 5 6 7 1\n",
        "[v] _\n",
    );
    let declarations = decls_with_verse("", "v");
    let tracks = parse(content, 0, &declarations).unwrap();
    let lyrics = notes_track(&tracks, "v").lyrics.as_ref().unwrap();
    assert_eq!(lyrics.measure_syllables.len(), 2);
    assert_eq!(lyrics.measure_syllables[0].len(), 4);
    assert!(lyrics.measure_syllables[1].is_empty());
}

#[test]
fn allows_too_few_lyrics_syllables_for_notes() {
    let content = "time=4/4 key=C4 bpm=120\n[] 1 2 3 4\n[v] a b c\n";
    let declarations = decls_with_verse("", "v");
    let tracks = parse(content, 0, &declarations).unwrap();
    assert_eq!(
        notes_track(&tracks, "v")
            .lyrics
            .as_ref()
            .unwrap()
            .measure_syllables[0]
            .len(),
        3
    );
}

#[test]
fn accepts_too_many_lyrics_syllables_for_notes() {
    // Overflow is recoverable — parsing succeeds and the grouper attaches an error to the measure.
    let content = "time=4/4 key=C4 bpm=120\n1 2 3 4\na b c d e\n";
    let declarations = vec![decl("", PartKind::Notes)];
    assert!(
        parse(content, 0, &declarations).is_ok(),
        "too many syllables must not abort parsing"
    );
}

#[test]
fn cross_measure_paren_group_parses() {
    let content = concat!(
        "time=4/4 key=C4 bpm=120\n",
        "[] 111(1\n",
        "\n",
        "[] 2)345\n",
    );
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).unwrap();
    let notes = notes_track(&tracks, "");
    let note_events: Vec<_> = all_events(notes)
        .into_iter()
        .filter_map(|e| match &e.value {
            ScoreEvent::Note(n) => Some(n),
            _ => None,
        })
        .collect();
    assert_eq!(note_events.len(), 8);
    assert!(note_events[3].slur);
    assert!(!note_events[4].slur);
}

#[test]
fn unclosed_paren_group_at_eof_is_recoverable() {
    let content = "time=4/4 key=C4 bpm=120\n[] 111(1\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).expect("unclosed group must not abort");
    let track = notes_track(&tracks, "");
    assert!(
        track.per_measure_chord_errors[0]
            .iter()
            .any(|diagnostic| diagnostic.message().contains("unclosed '(' group")),
        "expected unclosed group error on measure, got: {:?}",
        track.per_measure_chord_errors
    );
}

#[test]
fn tied_notes_share_one_lyric_slot_in_bar() {
    let content = "time=4/4 key=C4 bpm=120\n[] (33) 1 2\n[v] a b c\n";
    let declarations = decls_with_verse("", "v");
    let tracks = parse(content, 0, &declarations).unwrap();
    assert_eq!(
        notes_track(&tracks, "v")
            .lyrics
            .as_ref()
            .unwrap()
            .measure_syllables[0]
            .len(),
        3
    );
}

#[test]
fn cross_measure_tie_continuation_needs_fewer_lyrics() {
    let content = concat!(
        "time=4/4 key=C4 bpm=120\n[] 0 0 0 (3\n[v] a\n",
        "\n",
        "[] 3) 0 0 0\n",
        "[v] _\n",
    );
    let declarations = decls_with_verse("", "v");
    let tracks = parse(content, 0, &declarations).unwrap();
    let lyrics = notes_track(&tracks, "v").lyrics.as_ref().unwrap();
    assert_eq!(lyrics.measure_syllables.len(), 2);
    assert_eq!(lyrics.measure_syllables[0].len(), 1);
    assert!(lyrics.measure_syllables[1].is_empty());
}

#[test]
fn spaced_open_group_cross_measure_lyrics() {
    let content = concat!(
        "time=4/4 key=C4 bpm=120\n",
        "[main] 1 - 6m -\n",
        "[S1] (6- 7-\n",
        "[v] 慈 -\n",
        "\n",
        "[main] 1 - 6m -\n",
        "[S1] 7) 1 2 3\n",
        "[v] 光 - 光\n",
    );
    let declarations = vec![
        decl("main", PartKind::Chords),
        decl("S1", PartKind::Notes),
        lyric_decl("v", 1),
    ];
    let tracks = parse(content, 0, &declarations).unwrap();
    let verse = notes_track(&tracks, "v");
    assert_eq!(total_lyrics_syllables(verse), 5);
}

#[test]
fn measure_no_data_lines_is_recoverable() {
    // A directive-only group (no note lines) must not abort parsing.
    let content = "time=4/4 key=C4 bpm=120\n\n1 2 3 4\n";
    let declarations = vec![decl("", PartKind::Notes)];
    assert!(
        parse(content, 0, &declarations).is_ok(),
        "measure with no data lines must not abort parsing"
    );
}

#[test]
fn measure_too_many_lines_is_recoverable() {
    // Single-part declaration but two data lines — extra line must be silently ignored.
    let content = "time=4/4 key=C4 bpm=120\n1 2 3 4\n5 6 7 1\n";
    let declarations = vec![decl("", PartKind::Notes)];
    assert!(
        parse(content, 0, &declarations).is_ok(),
        "measure with too many lines must not abort parsing"
    );
}

#[test]
fn measure_missing_chord_line_is_recoverable() {
    // [notes, chord] parts but only the notes line is provided.
    // The chord role cannot be filled implicitly, so this previously errored.
    let content = "time=4/4 key=C4 bpm=120\n1 2 3 4\n";
    let declarations = vec![
        decl("Melody", PartKind::Notes),
        decl("chord", PartKind::Chords),
    ];
    assert!(
        parse(content, 0, &declarations).is_ok(),
        "measure missing a chord role line must not abort parsing"
    );
}

#[test]
fn chord_only_parts_produces_no_errors() {
    // A parts declaration with only a chord track must parse cleanly with no errors.
    let content = "time=4/4 key=C4 bpm=120\n[Chord] 1 2 3 4\n";
    let declarations = vec![decl("Chord", PartKind::Chords)];
    let desugar_errors = parse_recoverable_errors(content, 0, &declarations)
        .expect("chord-only parts must not abort parsing");
    assert!(
        desugar_errors.iter().all(|e| e.is_none()),
        "chord-only parts must produce no errors, got: {desugar_errors:?}"
    );
}

#[test]
fn lex_unexpected_char_in_notes_line_is_recoverable() {
    // `@` at a word boundary in a notes line triggers LexUnexpectedChar — must not abort parsing.
    let content = "time=4/4 key=C4 bpm=120\n[] 1 @ 3 4\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations)
        .expect("LexUnexpectedChar in notes line must not abort parsing");
    let track = notes_track(&tracks, "");
    assert_eq!(track.per_measure_lex_errors.len(), 1);
    assert!(
        track.per_measure_lex_errors[0].is_some(),
        "lex error must be recorded for the failing measure"
    );
}

#[test]
fn directive_error_span_includes_base_offset() {
    // When parse() is called with a non-zero base_offset, the span of a directive
    // parse error must be offset by base_offset, not relative to the start of content.
    let base_offset = 100;
    let content = "(unknown=foo)\n1 2 3 4\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let errors = parse_recoverable_errors(content, base_offset, &declarations)
        .expect("unknown directive must not abort parsing");
    let error = errors
        .into_iter()
        .find_map(|e| e)
        .expect("unknown directive must produce a recoverable error");
    assert!(
        error.span().start >= base_offset,
        "directive error span start ({}) must be >= base_offset ({})",
        error.span().start,
        base_offset
    );
}
