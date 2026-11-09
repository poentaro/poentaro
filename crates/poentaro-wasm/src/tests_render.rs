use super::*;
use crate::responses::render_response;
use crate::types::RenderResponse;
use poentaro::{NoteLabels, ResolvedPartVisibility};
use types::DiagnosticSeverity;

#[test]
fn ok_response_has_svgs() {
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#;
    let resp = render_response(
        input,
        &ResolvedPartVisibility::default(),
        &[],
        NoteLabels::Degrees,
    );
    match resp {
        RenderResponse::Ok { documents, .. } => {
            assert_eq!(documents.len(), 1);
            assert!(!documents[0].elements.is_empty());
        }
        RenderResponse::Err { .. } => panic!("expected ok"),
    }
}

#[test]
fn render_without_a_lyric_part_hides_its_lyrics() {
    let input = r#"metadata { title "t", author "a" },

parts [
    "Soprano": notes,
    "Soprano lyrics" [Sopranov1]: lyrics [Soprano],
    "Alto": notes,
    "Alto lyrics" [Altov1]: lyrics [Alto],
],

score """
    time=4/4 key=C4 bpm=120
    [Soprano] 1 2 3 4
    [Sopranov1] sop sop sop sop
    [Alto] 5 6 7 1
    [Altov1] alt alt alt alt
    """,
"#;
    let all = match render_response(
        input,
        &ResolvedPartVisibility::default(),
        &[],
        NoteLabels::Degrees,
    ) {
        RenderResponse::Ok { documents, .. } => documents,
        RenderResponse::Err { .. } => panic!("expected ok"),
    };
    let alto_lyrics_hidden = match render_response(
        input,
        &ResolvedPartVisibility {
            rendered_tracks: Some(vec!["Soprano".into(), "Sopranov1".into(), "Alto".into()]),
            ..Default::default()
        },
        &[],
        NoteLabels::Degrees,
    ) {
        RenderResponse::Ok { documents, .. } => documents,
        RenderResponse::Err { .. } => panic!("expected ok"),
    };
    // With lyrics, both parts render more elements than without
    assert!(all[0].elements.len() > alto_lyrics_hidden[0].elements.len());
}

#[test]
fn render_with_enabled_tracks_filters_parts() {
    let input = r#"metadata { title "t", author "a" },

parts ["Soprano": notes, "Alto": notes],

score """
    time=4/4 key=C4 bpm=120
    [Soprano] 1 2 3 4
    [Alto] 5 6 7 1
    """,
"#;
    let all = match render_response(
        input,
        &ResolvedPartVisibility::default(),
        &[],
        NoteLabels::Degrees,
    ) {
        RenderResponse::Ok { documents, .. } => documents,
        RenderResponse::Err { .. } => panic!("expected ok"),
    };
    let soprano_only = match render_response(
        input,
        &ResolvedPartVisibility {
            rendered_tracks: Some(vec!["Soprano".into()]),
            ..Default::default()
        },
        &[],
        NoteLabels::Degrees,
    ) {
        RenderResponse::Ok { documents, .. } => documents,
        RenderResponse::Err { .. } => panic!("expected ok"),
    };
    // Rendering both parts produces more elements than rendering one
    assert_ne!(all[0].elements.len(), soprano_only[0].elements.len());
}

#[test]
fn err_response_has_structured_diagnostic() {
    // Missing sections are now recoverable; render returns Ok with error diagnostics.
    let resp = render_response(
        "not valid poentaro",
        &ResolvedPartVisibility::default(),
        &[],
        NoteLabels::Degrees,
    );
    let diagnostics = match resp {
        RenderResponse::Err { diagnostics, .. } | RenderResponse::Ok { diagnostics, .. } => {
            diagnostics
        }
    };
    assert!(!diagnostics.is_empty());
    let d = &diagnostics[0];
    assert!(!d.message.is_empty());
}

#[test]
fn recoverable_error_produces_warning_severity_view_zone() {
    // lyrics underflow is a recoverable error
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b
    """,
"#;
    let resp = render_response(
        input,
        &ResolvedPartVisibility::default(),
        &[],
        NoteLabels::Degrees,
    );
    match resp {
        RenderResponse::Ok {
            diagnostics,
            diagnostic_view_zones,
            ..
        } => {
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
            assert_eq!(diagnostic_view_zones.len(), 1);
            assert_eq!(
                diagnostic_view_zones[0].severity,
                DiagnosticSeverity::Warning
            );
            assert_eq!(diagnostic_view_zones[0].messages.len(), 1);
        }
        RenderResponse::Err { .. } => panic!("expected ok"),
    }
}

#[test]
fn reference_poentaro_renders() {
    for path in demo_file_paths() {
        let source = read_demo_file(&path);
        let resp = render_response(
            &source,
            &ResolvedPartVisibility::default(),
            &[],
            NoteLabels::Degrees,
        );
        match resp {
            RenderResponse::Ok {
                documents,
                diagnostics,
                ..
            } => {
                assert!(
                    !documents.is_empty(),
                    "{path:?} should render in the wasm path used by the web editor"
                );
                assert!(
                    !documents[0].elements.is_empty(),
                    "{path:?} first page should have elements"
                );
                assert!(
                    diagnostics.is_empty(),
                    "{path:?} should have no errors or warnings, got: {:?}",
                    diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
                );
            }
            RenderResponse::Err { diagnostics, .. } => {
                panic!(
                    "{path:?} failed in wasm render path: {}",
                    diagnostics[0].message
                );
            }
        }
    }
}

#[test]
fn diagnostic_span_is_utf8_byte_offset() {
    // 'z' in a notes line is a recoverable error (LexUnexpectedChar),
    // so render returns Ok with a warning diagnostic.
    let source = r#"metadata { title "你好", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 z 4
    [Melodyv1] a b c d
    """,
"#;
    let token_byte_start = source.find('z').expect("error token in source");
    let resp = render_response(
        source,
        &ResolvedPartVisibility::default(),
        &[],
        NoteLabels::Degrees,
    );
    let diagnostics = match resp {
        RenderResponse::Ok { diagnostics, .. } => diagnostics,
        RenderResponse::Err { diagnostics, .. } => diagnostics,
    };
    assert!(!diagnostics.is_empty());
    // The diagnostic span must overlap with the 'x' token — verify the span
    // is anchored absolutely in the source (not line-locally, which would be < 4).
    let d = &diagnostics[0];
    assert!(
        d.span.start >= token_byte_start || d.span.end > token_byte_start,
        "span ({}, {}) does not cover token at {token_byte_start}",
        d.span.start,
        d.span.end
    );
    assert!(
        d.span.start > 4,
        "span.start {} should be absolute in source, not line-local",
        d.span.start
    );
}

#[test]
fn follow_part_with_explicit_override_in_some_sections_renders_ok() {
    // p2 follows p1 but has an explicit override ([p2]12) in the first section.
    // s2 follows s1 but has an explicit override ([s2]67) in the first section.
    // In the second section (time=4/4), only [s1] and [a] appear; the follow
    // parts (p2, s2) and their leaders (p1) are absent and should be filled
    // implicitly / via follow resolution.
    let input = r#"metadata { title "", author "" },

parts [
    "Pluck" [p1]: notes,
    "Pluck 2" [p2]: follow [p1],
    "String" [s1]: notes,
    "String 2" [s2]: follow [s1],
    "Accompaniment" [a]: chords,
],

score """
    time=2/4
    [p1]12
    [p2]12
    [s1]1'2'
    [s2]67
    [a]4 5

    time=4/4
    [s1] 5---
    [a] 1---
    """,
"#;
    let resp = render_response(
        input,
        &ResolvedPartVisibility::default(),
        &[],
        NoteLabels::Degrees,
    );
    match resp {
        RenderResponse::Ok {
            documents,
            diagnostics,
            ..
        } => {
            assert!(!documents.is_empty(), "expected at least one page");
            assert!(
                diagnostics.is_empty(),
                "expected no diagnostics, got: {}",
                diagnostics
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
        RenderResponse::Err { diagnostics, .. } => {
            panic!(
                "expected ok but got error: {}",
                diagnostics
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
    }
}

#[test]
fn two_parts_with_non_four_four_first_section_has_no_diagnostics() {
    // Minimal reproducer: two parts, time=2/4 first section, both parts
    // present in the second section.  The renderer must not emit any
    // diagnostics — the bug caused spurious "measure count mismatch" and
    // "beat overflow" errors whenever the first section was not 4/4.
    let input = r#"metadata { title "", author "" },

parts ["A" [a]: notes, "B" [b]: notes],

score """
    time=2/4
    [a]12
    [b]12

    time=4/4
    [a] 5
    [b] 1
    """,
"#;
    let resp = render_response(
        input,
        &ResolvedPartVisibility::default(),
        &[],
        NoteLabels::Degrees,
    );
    match resp {
        RenderResponse::Ok {
            documents,
            diagnostics,
            ..
        } => {
            assert!(!documents.is_empty(), "expected at least one page");
            assert!(
                diagnostics.is_empty(),
                "expected no diagnostics, got: {}",
                diagnostics
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
        RenderResponse::Err { diagnostics, .. } => {
            panic!(
                "expected ok but got error: {}",
                diagnostics
                    .iter()
                    .map(|d| d.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
    }
}
