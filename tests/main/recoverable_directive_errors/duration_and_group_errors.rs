use super::*;

// Group — Dotted eighth without sixteenth tail

#[test]
fn dotted_eighth_note_without_sixteenth_tail_is_recoverable() {
    // `1_.` is a dotted eighth note; it must be followed by a sixteenth.
    // When no sixteenth tail follows, render must continue and surface a diagnostic.
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\n[Melody] 1_. 2_ 3_ 4_ 5_ 6_ 7_ 0=\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("dotted eighth without sixteenth tail must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "dotted eighth"),
        "expected error about dotted eighth, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn dotted_eighth_rest_without_sixteenth_tail_is_recoverable() {
    // `0_.` is a dotted eighth rest; same rule applies.
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\n[Melody] 0_. 1_ 2_ 3_ 4_ 5_ 6_ 0=\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("dotted eighth rest without sixteenth tail must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "dotted eighth"),
        "expected error about dotted eighth, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// NoteExpectedPitchDigit

#[test]
fn note_invalid_pitch_char_is_recoverable() {
    // 'y' is not a valid pitch digit (0-7) and not a duplicate atom (`x`/`_`/`=`); the lexer
    // rejects it as LexUnexpectedChar, which is recoverable — the measure is skipped and the
    // render continues.
    let source = minimal_fixture("[Melody] 1 y 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("invalid pitch char must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "unexpected character"),
        "expected error about unexpected character, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn dotted_eighth_with_sixteenth_tail_is_valid() {
    // `1_.` followed by `2=` (sixteenth) is a valid pattern — no error expected.
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\n[Melody] 1_. 2= 3_ 4_ 5_ 6_ 7_ 1_\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("dotted eighth with sixteenth tail must not abort");
    assert!(!output.svgs.is_empty());
    assert!(
        !has_error_containing(&output, "dotted eighth"),
        "expected no dotted-eighth error for valid pattern, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// DurationCannotDotQuarterBeat

#[test]
fn dotted_quarter_beat_is_recoverable() {
    // `1=.` applies a dot to a quarter-beat note, which is invalid.
    // The render must continue; the dot is ignored and duration stays at 1 beat.
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\n[Melody] 1=. 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("dotted quarter-beat must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "cannot dot a quarter-beat"),
        "expected error about dotted quarter-beat, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// DurationCannotDoubleDot

#[test]
fn double_dotted_eighth_falls_back_to_single_dot_and_is_recoverable() {
    // `1_..` applies a second dot to an eighth note (duration 2, before dotting).
    // 2 % 4 != 0, so the second dot is invalid; it's dropped and the first dot survives,
    // producing the same duration as a plain dotted eighth (`1_.`, duration 3). The render
    // must continue and surface a diagnostic about the failed second dot.
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\n[Melody] 1_.. 2= 3_ 4_ 5_ 6_ 7_ 1_\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("double-dotted eighth must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "cannot apply a second dot"),
        "expected error about the second dot, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// GroupUnexpectedCloseParen

#[test]
fn group_unexpected_close_paren_is_recoverable() {
    // `1 2) 3 4` has a stray `)` with no matching `(`.
    // The render must continue; the `)` is ignored and an error is reported on the measure.
    let source = minimal_fixture("[Melody] 1 2) 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("stray ) must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "unexpected"),
        "expected error about unexpected ), got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// UnclosedGroupAtEnd

#[test]
fn unclosed_paren_group_at_eof_is_recoverable() {
    // `(1 2 3 4` opens a group but never closes it before EOF.
    // The render must continue; the group is treated as open and an error is reported.
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\n[Melody] (1 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("unclosed group at EOF must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "unclosed '(' group"),
        "expected error about unclosed group, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

// DurationMixedOctaveMarkers

#[test]
fn mixed_octave_markers_are_recoverable() {
    // `1',` has both ' (octave-up) and , (octave-down) — mixed octave markers.
    // The render must continue; the note is emitted with octave shift zeroed out.
    let source = minimal_fixture("time=4/4 key=C4 bpm=120\n[Melody] 1', 2 3 4\n");
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("mixed octave markers must not abort the render");
    assert!(!output.svgs.is_empty());
    assert!(
        has_error_containing(&output, "mixed octave"),
        "expected error about mixed octave markers, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}
