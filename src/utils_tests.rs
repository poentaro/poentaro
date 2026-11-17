use super::*;

#[test]
fn tokenises_cjk_without_spaces() {
    let syllables = tokenize_lyrics("你好世界", 0);
    assert_eq!(syllables.len(), 4);
    assert_eq!(syllables[0].text, "你");
    assert_eq!(syllables[1].text, "好");
    assert_eq!(syllables[2].text, "世");
    assert_eq!(syllables[3].text, "界");
}

#[test]
fn tokenises_non_cjk_by_space() {
    let syllables = tokenize_lyrics("he llo world", 0);
    assert_eq!(syllables.len(), 3);
    assert_eq!(syllables[0].text, "he");
    assert_eq!(syllables[1].text, "llo");
    assert_eq!(syllables[2].text, "world");
}

#[test]
fn mixed_cjk_and_latin() {
    let syllables = tokenize_lyrics("你好world", 0);
    assert_eq!(syllables.len(), 3);
    assert_eq!(syllables[0].text, "你");
    assert_eq!(syllables[1].text, "好");
    assert_eq!(syllables[2].text, "world");
}

#[test]
fn spaces_around_cjk_are_ignored() {
    let syllables = tokenize_lyrics("你好 world", 0);
    assert_eq!(syllables.len(), 3);
    assert_eq!(syllables[2].text, "world");
}

#[test]
fn dash_marks_held_syllable() {
    // `he llo - world` → 4 syllables: he, llo (held=true), - (placeholder), world
    let syllables = tokenize_lyrics("he llo - world", 0);
    assert_eq!(syllables.len(), 4);
    assert!(!syllables[0].held);
    assert!(syllables[1].held);
    assert_eq!(syllables[2].text, "-");
    assert!(!syllables[3].held);
}

#[test]
fn held_is_false_by_default() {
    let syllables = tokenize_lyrics("你好", 0);
    assert!(!syllables[0].held);
    assert!(!syllables[1].held);
}

#[test]
fn ignores_leading_trailing_whitespace() {
    let syllables = tokenize_lyrics("  hello  ", 0);
    assert_eq!(syllables.len(), 1);
    assert_eq!(syllables[0].text, "hello");
}

#[test]
fn span_covers_each_syllables_own_token_offset_by_base() {
    // "he llo world" with base_offset 100: "he" at 0..2, "llo" at 3..6,
    // "world" at 7..12 — each shifted by the base offset.
    let syllables = tokenize_lyrics("he llo world", 100);
    assert_eq!(syllables[0].span, Span::new(100, 102));
    assert_eq!(syllables[1].span, Span::new(103, 106));
    assert_eq!(syllables[2].span, Span::new(107, 112));
}

#[test]
fn span_of_trailing_dash_syllable_includes_the_dash() {
    // "twin-" spans the whole 5-byte token, dash included.
    let syllables = tokenize_lyrics("twin- kle", 0);
    assert_eq!(syllables[0].text, "twin-");
    assert_eq!(syllables[0].span, Span::new(0, 5));
}

#[test]
fn span_of_cjk_syllable_covers_its_utf8_byte_width() {
    // Each CJK character in "你好" is 3 UTF-8 bytes.
    let syllables = tokenize_lyrics("你好", 0);
    assert_eq!(syllables[0].span, Span::new(0, 3));
    assert_eq!(syllables[1].span, Span::new(3, 6));
}

// --- new tests ---

#[test]
fn empty_string_returns_empty() {
    assert_eq!(tokenize_lyrics("", 0), Vec::<Syllable>::new());
}

#[test]
fn dash_at_start_no_panic() {
    let syllables = tokenize_lyrics("- hello", 0);
    // first token is "-" (no previous syllable to mark held), second is "hello"
    assert_eq!(syllables.len(), 2);
    assert_eq!(syllables[0].text, "-");
    assert!(!syllables[0].held);
    assert_eq!(syllables[1].text, "hello");
    assert!(!syllables[1].held);
}

#[test]
fn dash_at_end() {
    let syllables = tokenize_lyrics("hello -", 0);
    assert_eq!(syllables.len(), 2);
    assert_eq!(syllables[0].text, "hello");
    assert!(syllables[0].held);
    assert_eq!(syllables[1].text, "-");
    assert!(!syllables[1].held);
}

#[test]
fn trailing_dash_on_word_is_syllable_break() {
    let syllables = tokenize_lyrics("twin- kle twin- kle", 0);
    assert_eq!(syllables.len(), 4);
    assert_eq!(syllables[0].text, "twin-");
    assert!(!syllables[0].held);
    assert_eq!(syllables[1].text, "kle");
    assert_eq!(syllables[2].text, "twin-");
    assert_eq!(syllables[3].text, "kle");
}

#[test]
fn count_lyric_slots_skips_tie_continuation() {
    use crate::ast::parsed::{NumberedPitch, ParsedNote, ScoreEvent};
    use crate::error::{Span, Spanned};

    let events = vec![
        Spanned::new(
            ScoreEvent::Note(ParsedNote {
                pitch: NumberedPitch::Three,
                accidental: crate::ast::parsed::Accidental::Natural,
                octave: 0,
                duration: 4,
                slur: false,
                group_membership: 0,
                group_continuation: 0,
                dotted: false,
                double_dotted: false,
                tie_to_next_span: Some(Span::new(0, 1)),
                slur_group_close_at_duration: None,
                tuplet: None,
            }),
            Span::new(0, 1),
        ),
        Spanned::new(
            ScoreEvent::Note(ParsedNote {
                pitch: NumberedPitch::Three,
                accidental: crate::ast::parsed::Accidental::Natural,
                octave: 0,
                duration: 4,
                slur: false,
                group_membership: 0,
                group_continuation: 0,
                dotted: false,
                double_dotted: false,
                tie_to_next_span: None,
                slur_group_close_at_duration: None,
                tuplet: None,
            }),
            Span::new(1, 2),
        ),
        Spanned::new(
            ScoreEvent::Note(ParsedNote {
                pitch: NumberedPitch::One,
                accidental: crate::ast::parsed::Accidental::Natural,
                octave: 0,
                duration: 4,
                slur: false,
                group_membership: 0,
                group_continuation: 0,
                dotted: false,
                double_dotted: false,
                tie_to_next_span: None,
                slur_group_close_at_duration: None,
                tuplet: None,
            }),
            Span::new(2, 3),
        ),
    ];
    let mut state = LyricTieState::default();
    assert_eq!(count_lyric_slots_in_events(&events, &mut state), 2);
    assert!(!state.prev_tie_to_next);
}

#[test]
fn count_lyric_slots_carries_tie_across_bars() {
    use crate::ast::parsed::{NumberedPitch, ParsedNote, ScoreEvent};
    use crate::error::{Span, Spanned};

    let bar1 = vec![Spanned::new(
        ScoreEvent::Note(ParsedNote {
            pitch: NumberedPitch::Three,
            accidental: crate::ast::parsed::Accidental::Natural,
            octave: 0,
            duration: 4,
            slur: false,
            tie_to_next_span: Some(Span::new(0, 1)),
            group_membership: 0,
            group_continuation: 0,
            dotted: false,
            double_dotted: false,
            slur_group_close_at_duration: None,
            tuplet: None,
        }),
        Span::new(0, 1),
    )];
    let bar2 = vec![Spanned::new(
        ScoreEvent::Note(ParsedNote {
            pitch: NumberedPitch::Three,
            accidental: crate::ast::parsed::Accidental::Natural,
            octave: 0,
            duration: 4,
            slur: false,
            tie_to_next_span: None,
            group_membership: 0,
            group_continuation: 0,
            dotted: false,
            double_dotted: false,
            slur_group_close_at_duration: None,
            tuplet: None,
        }),
        Span::new(0, 1),
    )];
    let mut state = LyricTieState::default();
    assert_eq!(count_lyric_slots_in_events(&bar1, &mut state), 1);
    assert_eq!(count_lyric_slots_in_events(&bar2, &mut state), 0);
}

#[test]
fn consecutive_dashes() {
    // "你 - - 好" → 4 syllables: "你" held=true, "-" held=true, "-", "好"
    let syllables = tokenize_lyrics("你 - - 好", 0);
    assert_eq!(syllables.len(), 4);
    assert_eq!(syllables[0].text, "你");
    assert!(syllables[0].held);
    assert_eq!(syllables[1].text, "-");
    assert!(syllables[1].held);
    assert_eq!(syllables[2].text, "-");
    assert!(!syllables[2].held);
    assert_eq!(syllables[3].text, "好");
    assert!(!syllables[3].held);
}
