use crate::error::Diagnostic;

fn errors_of(soprano: &str, alto: &str) -> Vec<String> {
    let source = format!(
        "parts [\"S\" [S]: notes, \"A\" [A]: notes, \"w\" [w]: lyrics [S, A]],\nscore \"\"\"\ntime=2/4 key=C4 bpm=60\n[S] {soprano}\n[A] {alto}\n[w] 我 们\n\"\"\",\n"
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
        errors[0].contains("[S] takes 2 syllable(s)") && errors[0].contains("[A] takes 1"),
        "{}",
        errors[0]
    );
}
