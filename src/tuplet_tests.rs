use crate::ast::parsed::{
    Accidental, NumberedPitch, ParsedNote, ParsedRest, ScoreEvent, TupletInfo,
};
use crate::error::{Span, Spanned};
use crate::tuplet::{apply_resolution_multiplier, resolution_multiplier_of};

fn rescale_tuplets(events: Vec<Spanned<ScoreEvent>>) -> (Vec<Spanned<ScoreEvent>>, u32) {
    let resolution_multiplier = resolution_multiplier_of(&events);
    let events = apply_resolution_multiplier(events, resolution_multiplier);
    (events, resolution_multiplier)
}

fn note(duration: u32, tuplet: Option<TupletInfo>) -> Spanned<ScoreEvent> {
    Spanned::new(
        ScoreEvent::Note(ParsedNote {
            pitch: NumberedPitch::One,
            accidental: Accidental::Natural,
            octave: 0,
            duration,
            slur: false,
            tie_to_next_span: None,
            group_membership: 0,
            group_continuation: 0,
            dotted: false,
            double_dotted: false,
            slur_group_close_at_duration: None,
            tuplet,
        }),
        Span::new(0, 0),
    )
}

fn rest(duration: u32, tuplet: Option<TupletInfo>) -> Spanned<ScoreEvent> {
    Spanned::new(
        ScoreEvent::Rest(ParsedRest {
            duration,
            dotted: false,
            double_dotted: false,
            group_membership: 0,
            group_continuation: 0,
            tuplet,
            implicit_fill: false,
        }),
        Span::new(0, 0),
    )
}

fn duration_of(spanned: &Spanned<ScoreEvent>) -> u32 {
    match &spanned.value {
        ScoreEvent::Note(n) => n.duration,
        ScoreEvent::Rest(r) => r.duration,
        _ => panic!("expected Note or Rest, got {:?}", spanned.value),
    }
}

#[test]
fn no_tuplets_is_a_no_op() {
    let events = vec![note(4, None), note(4, None), rest(4, None), note(4, None)];
    let (events, resolution_multiplier) = rescale_tuplets(events);
    assert_eq!(resolution_multiplier, 1);
    assert_eq!(
        events.iter().map(duration_of).collect::<Vec<_>>(),
        vec![4, 4, 4, 4]
    );
}

#[test]
fn eighth_note_triplet_fills_exactly_one_beat() {
    // `3:{1_1_1_}` — three eighth notes (duration 2 each) compressed 3-in-2.
    let tuplet = Some(TupletInfo {
        num: 3,
        den: 2,
        id: 0,
    });
    let events = vec![note(2, tuplet), note(2, tuplet), note(2, tuplet)];
    let (events, resolution_multiplier) = rescale_tuplets(events);
    assert_eq!(resolution_multiplier, 3);
    let durations: Vec<u32> = events.iter().map(duration_of).collect();
    assert_eq!(durations, vec![4, 4, 4]);
    // One beat in the rescaled grid is `4 * resolution_multiplier`; the triplet's three
    // notes together must sum to exactly that, i.e. the tuplet fills exactly one beat.
    let beat_in_rescaled_units = 4 * resolution_multiplier;
    assert_eq!(durations.iter().sum::<u32>(), beat_in_rescaled_units);
}

#[test]
fn plain_notes_in_the_same_measure_are_scaled_by_the_same_multiplier() {
    // A triplet filling beat 1, followed by three plain quarter notes filling beats 2-4.
    let tuplet = Some(TupletInfo {
        num: 3,
        den: 2,
        id: 0,
    });
    let events = vec![
        note(2, tuplet),
        note(2, tuplet),
        note(2, tuplet),
        note(4, None),
        note(4, None),
        note(4, None),
    ];
    let (events, resolution_multiplier) = rescale_tuplets(events);
    assert_eq!(resolution_multiplier, 3);
    let durations: Vec<u32> = events.iter().map(duration_of).collect();
    // Triplet notes: 2 * 3 * 2 / 3 = 4 each. Plain notes: 4 * 3 = 12 each.
    assert_eq!(durations, vec![4, 4, 4, 12, 12, 12]);
    // The whole measure (16 quarter-beats in 4/4) must rescale to `16 * multiplier`.
    assert_eq!(durations.iter().sum::<u32>(), 16 * resolution_multiplier);
}

#[test]
fn multiplier_is_lcm_of_every_tuplet_num_present() {
    // A triplet (num=3) and a quintuplet (num=5) in the same measure: multiplier = 15.
    let triplet = Some(TupletInfo {
        num: 3,
        den: 2,
        id: 0,
    });
    let quintuplet = Some(TupletInfo {
        num: 5,
        den: 4,
        id: 0,
    });
    let events = vec![note(1, triplet), note(1, quintuplet)];
    let (_, resolution_multiplier) = rescale_tuplets(events);
    assert_eq!(resolution_multiplier, 15);
}

#[test]
fn rests_inside_a_tuplet_are_rescaled_like_notes() {
    let tuplet = Some(TupletInfo {
        num: 3,
        den: 2,
        id: 0,
    });
    let events = vec![note(2, tuplet), rest(2, tuplet), note(2, tuplet)];
    let (events, _) = rescale_tuplets(events);
    let durations: Vec<u32> = events.iter().map(duration_of).collect();
    assert_eq!(durations, vec![4, 4, 4]);
}
