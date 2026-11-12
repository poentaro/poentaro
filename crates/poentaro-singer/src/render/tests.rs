use super::lyrics::phrase_text;
use super::{mix, phrases, piece_count, segments, sing, sing_piece};
use crate::{SingingPlan, SungNote, SungPart, SungSyllable, Voice};

fn note(start_seconds: f64, end_seconds: f64, syllable: SungSyllable) -> SungNote {
    SungNote {
        start_seconds,
        end_seconds,
        midi: 60,
        syllable,
    }
}

fn word(text: &str) -> SungSyllable {
    SungSyllable::Word(text.to_string())
}

fn phrase_starts(notes: &[SungNote]) -> Vec<Vec<f64>> {
    phrases(notes)
        .iter()
        .map(|phrase| phrase.iter().map(|note| note.start_seconds).collect())
        .collect()
}

#[test]
fn a_rest_or_a_silent_note_ends_a_phrase_and_a_leading_hold_is_dropped() {
    let notes = [
        note(0.0, 1.0, word("银")),
        note(1.0, 2.0, word("行")),
        note(3.0, 4.0, SungSyllable::Held),
        note(4.0, 5.0, word("我")),
        note(5.0, 6.0, SungSyllable::Silent),
        note(6.0, 7.0, word("们")),
    ];
    assert_eq!(
        phrase_starts(&notes),
        vec![vec![0.0, 1.0], vec![4.0], vec![6.0]]
    );
}

#[test]
fn chinese_characters_join_without_spaces_and_holds_stand_alone() {
    let notes = [
        note(0.0, 1.0, word("银")),
        note(1.0, 2.0, word("行")),
        note(2.0, 3.0, SungSyllable::Held),
        note(3.0, 4.0, word("hello")),
    ];
    let phrase = notes.iter().collect::<Vec<_>>();
    assert_eq!(phrase_text(&phrase), "银行 - hello");
}

#[test]
fn a_long_rest_starts_a_new_voice_segment() {
    let notes = vec![
        note(0.0, 1.0, word("一")),
        note(1.5, 2.0, word("二")),
        note(10.0, 11.0, word("三")),
    ];
    let segment_sizes = segments(phrases(&notes))
        .iter()
        .map(Vec::len)
        .collect::<Vec<_>>();
    assert_eq!(segment_sizes, vec![2, 1]);
}

#[test]
fn a_sung_part_peaks_at_its_volume_and_starts_where_its_notes_do() {
    let sample_rate = 8000;
    let samples = sing(&SingingPlan {
        sample_rate,
        parts: vec![SungPart {
            voice: Voice::Soprano,
            volume_percent: 50,
            notes: vec![note(2.0, 2.5, word("la"))],
        }],
    });
    let peak = samples
        .iter()
        .fold(0.0f32, |max, sample| max.max(sample.abs()));
    assert!((peak - 0.5).abs() < 1e-4, "peak {peak}");
    let first_sound = samples.iter().position(|sample| sample.abs() > 1e-3);
    let first_sound_seconds = first_sound.map(|index| index as f64 / f64::from(sample_rate));
    assert!(
        first_sound_seconds.is_some_and(|seconds| (1.5..2.5).contains(&seconds)),
        "first sound at {first_sound_seconds:?}"
    );
}

#[test]
fn pieces_sung_separately_and_mixed_in_any_order_make_the_same_song() {
    let plan = SingingPlan {
        sample_rate: 44100,
        parts: vec![
            SungPart {
                voice: Voice::Soprano,
                volume_percent: 80,
                notes: vec![note(0.0, 0.5, word("la")), note(4.0, 4.5, word("li"))],
            },
            SungPart {
                voice: Voice::Bass,
                volume_percent: 60,
                notes: vec![note(1.0, 1.5, word("lo"))],
            },
        ],
    };
    assert_eq!(piece_count(&plan), 3);
    let reversed = (0..piece_count(&plan))
        .rev()
        .filter_map(|index| sing_piece(&plan, index))
        .collect::<Vec<_>>();
    assert_eq!(mix(&plan, &reversed), sing(&plan));
    assert_eq!(sing_piece(&plan, 3), None);
}
