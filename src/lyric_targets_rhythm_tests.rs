use crate::error::Diagnostic;

fn errors_of(soprano: &str, alto: &str) -> Vec<String> {
    errors_with_words(soprano, alto, "我 们")
}

fn errors_with_words(soprano: &str, alto: &str, words: &str) -> Vec<String> {
    let source = format!(
        "parts [\"S\" [S]: notes, \"A\" [A]: notes, \"w\" [w]: lyrics [S, A]],\nscore \"\"\"\ntime=2/4 key=C4 bpm=60\n[S] {soprano}\n[A] {alto}\n[w] {words}\n\"\"\",\n"
    );
    let score = crate::compile(&source, "test.poentaro", &[]).expect("compiles");
    score
        .measures
        .iter()
        .flat_map(|measure| measure.diagnostics.iter())
        .filter_map(|diagnostic| match diagnostic {
            Diagnostic::Error(error) => Some(error.message()),
            Diagnostic::Warning(_) => None,
        })
        .collect()
}

#[test]
fn targets_with_the_same_rhythm_are_fine() {
    assert!(errors_of("1 2", "5, 7,").is_empty());
}

#[test]
fn targets_taking_different_numbers_of_syllables_are_an_error() {
    let errors = errors_of("1 2", "5, -");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("[S] doesn't take its syllables at the same times as [A]"),
        "{}",
        errors[0]
    );
}

#[test]
fn a_slur_carries_one_syllable_over_several_notes() {
    assert!(errors_of("(1_2_) 3", "5, 7,").is_empty());
}

#[test]
fn a_tie_onto_the_same_pitch_takes_no_syllable_of_its_own() {
    assert!(errors_of("1_~1_ 2", "5, 7,").is_empty());
}

#[test]
fn targets_taking_as_many_syllables_at_different_times_are_an_error() {
    let errors = errors_of("1. 2_", "5, 7,");
    assert_eq!(errors.len(), 1, "{errors:?}");
}

#[test]
fn a_target_resting_all_measure_is_not_checked() {
    assert!(errors_of("0 0", "5, 7,").is_empty());
}

#[test]
fn a_target_may_hold_its_note_through_a_held_syllable() {
    assert!(errors_with_words("1 -", "5, 7,", "我 -").is_empty());
}

#[test]
fn a_target_resting_when_a_syllable_is_sung_is_an_error() {
    let errors = errors_of("1 0", "5, 7,");
    assert_eq!(errors.len(), 1, "{errors:?}");
}

#[test]
fn a_choir_with_one_part_slurring_two_notes_on_a_syllable_is_fine() {
    let source = "parts [\"S\" [S]: notes, \"T\" [T]: notes, \"A\" [A]: notes, \"B\" [B]: notes, \"Words\" [i]: lyrics [S, T, A, B]],\nscore \"\"\"\ntime=3/4 key=C4 bpm=60\n[S] 3 3 (2_1_)\n[T] 3 3 (2_1_)\n[A] 6,6,5,\n[B] 6,3,5,\n[i] 我們道\n[i] 我們心\n\"\"\",\n";
    let score = crate::compile(source, "test.poentaro", &[]).expect("compiles");
    let errors = score
        .measures
        .iter()
        .flat_map(|measure| measure.diagnostics.iter())
        .filter(|diagnostic| matches!(diagnostic, Diagnostic::Error(_)))
        .count();
    assert_eq!(errors, 0);
}

#[test]
fn the_words_pair_with_the_part_singing_while_the_bottommost_rests() {
    let source = "parts [\"S\" [S]: notes, \"A\" [A]: notes, \"w\" [w]: lyrics [S, A]],\nscore \"\"\"\ntime=2/4 key=C4 bpm=60\n[S] 1_2_ 3\n[A] 0 0\n[w] 我 们 有\n\"\"\",\n";
    let score = crate::compile(source, "test.poentaro", &[]).expect("compiles");
    let diagnostics = score
        .measures
        .iter()
        .flat_map(|measure| measure.diagnostics.iter())
        .collect::<Vec<_>>();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}
