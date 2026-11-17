#![allow(clippy::disallowed_macros)]
use poentaro::error::{Diagnostic, WarningKind};
use poentaro::render_svgs_from_source;

/// A `.poentaro` source whose single measure is wider than the page on its own
/// — `pack_into_systems` breaks a system early before a measure that wouldn't
/// fit, but a lone measure has nowhere to break, so its system's summed column
/// rods still exceed the page's usable music width.
fn overflowing_fixture() -> String {
    let notes = std::iter::repeat_n("1__ 2__ 3__ 4__", 32)
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        r#"metadata {{ title "t", author "a" }},

parts ["Melody": notes],

score """
    time=32/4 key=C4 bpm=120
    [Melody] {notes}
    """,
"#
    )
}

fn has_measure_overflow_warning(output: &poentaro::RenderOutput) -> bool {
    output.diagnostics.iter().any(|d| {
        matches!(
            d,
            Diagnostic::Warning(w) if w.kind == WarningKind::MeasureOverflow
        )
    })
}

#[test]
fn tightly_packed_system_produces_measure_overflow_warning() {
    let source = overflowing_fixture();
    let output = render_svgs_from_source(&source, "test.poentaro", &[])
        .expect("an overflowing system must not abort the render");
    assert!(
        !output.svgs.is_empty(),
        "render should still produce output"
    );
    assert!(
        has_measure_overflow_warning(&output),
        "expected a WarningKind::MeasureOverflow diagnostic, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn ordinary_score_produces_no_measure_overflow_warning() {
    let source = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    """,
"#;
    let output = render_svgs_from_source(source, "test.poentaro", &[])
        .expect("a normal score must render without error");
    assert!(
        !has_measure_overflow_warning(&output),
        "an ordinary, non-overflowing score should not produce a \
         MeasureOverflow warning, got: {:?}",
        output
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}
