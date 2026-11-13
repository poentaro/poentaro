use super::*;
use crate::error::Span;

fn syllable(text: &str, bracketed: bool) -> Syllable {
    Syllable {
        text: text.to_string(),
        held: false,
        bracketed,
        pinyin: None,
        span: Span::new(0, 0),
    }
}

#[test]
fn multiplier_is_one_without_unbracketed_syllables() {
    assert_eq!(
        unbracketed_resolution_multiplier(&[syllable("a", true), syllable("b", true)]),
        1
    );
}

#[test]
fn multiplier_leaves_room_for_every_unbracketed_syllable() {
    let line = [
        syllable("~", true),
        syllable("a", false),
        syllable("b", false),
        syllable("c", false),
    ];
    assert_eq!(unbracketed_resolution_multiplier(&line), 4);
}

#[test]
fn spread_puts_columns_strictly_between_the_ends() {
    assert_eq!(spread(0, 8, 3, false), vec![2, 4, 6]);
}

#[test]
fn spread_lead_in_starts_at_the_bar_start() {
    assert_eq!(spread(0, 8, 2, true), vec![0, 4]);
}

#[test]
fn spread_of_nothing_is_empty() {
    assert_eq!(spread(0, 8, 0, false), Vec::<u32>::new());
}

#[test]
fn an_unbracketed_first_syllable_is_chord_bound_when_the_first_attack_is_on_beat_one() {
    let line = [
        syllable("a", false),
        syllable("b", false),
        syllable("c", true),
    ];
    assert_eq!(chord_bound_indices(&line, true), vec![0, 2]);
}

#[test]
fn an_unbracketed_first_syllable_is_a_pickup_when_the_first_attack_is_past_beat_one() {
    let line = [
        syllable("a", false),
        syllable("b", false),
        syllable("c", true),
    ];
    assert_eq!(chord_bound_indices(&line, false), vec![2]);
}

#[test]
fn a_bracketed_first_syllable_is_chord_bound_only_once() {
    let line = [syllable("a", true), syllable("b", false)];
    assert_eq!(chord_bound_indices(&line, true), vec![0]);
}

#[test]
fn the_tally_counts_an_unbracketed_first_syllable_on_beat_one() {
    let line = [syllable("a", false), syllable("b", true)];
    assert!(chord_lyric_tally_error(&line, 2, true, Span::new(0, 0)).is_none());
    assert_eq!(
        chord_lyric_tally_error(&line, 1, true, Span::new(0, 0)).map(|error| error.message()),
        Some(
            "lyrics under a chord part need at most one bracketed item per chord attack, \
             found 1 bracketed item for 1 chord attack; \
             the unbracketed first syllable already takes the beat-1 chord"
                .to_string()
        )
    );
}

#[test]
fn the_tally_passes_a_line_without_syllables() {
    assert!(chord_lyric_tally_error(&[], 0, true, Span::new(0, 0)).is_none());
}
