use super::*;
use crate::source_embed::extract_embedded_source;

#[test]
fn round_trip_recovers_original_source() {
    let source = r#"metadata { title "Testing" },

parts ["Melody": notes],

score """
    [Melody] 1 2 3 4
    """,
"#;
    let output = render_svgs_from_source(source, "test.poentaro", &[]).unwrap();
    for svg in &output.svgs {
        assert_eq!(extract_embedded_source(svg).as_deref(), Some(source));
    }
}

#[test]
fn score_only_render_has_no_embedded_source() {
    let source = r#"metadata { title "Testing" },

parts ["Melody": notes],

score """
    [Melody] 1 2 3 4
    """,
"#;
    let score = compile(source, "test.poentaro", &[]).unwrap();
    let svgs = render_svgs(&score).unwrap();
    for svg in &svgs {
        assert!(!svg.contains(r#"<metadata id="poentaro-source">"#));
        assert_eq!(extract_embedded_source(svg), None);
    }
}

#[test]
fn extract_returns_none_when_metadata_missing() {
    assert_eq!(extract_embedded_source("<svg></svg>"), None);
}

#[cfg(feature = "pdf")]
mod pdf_embedding {
    use super::*;
    use crate::source_embed::extract_embedded_source_from_pdf;

    #[test]
    fn round_trip_recovers_original_source_from_pdf() {
        let source = r#"metadata { title "Testing" },

parts ["Melody": notes],

score """
    [Melody] 1 2 3 4
    """,
"#;
        let pdf_bytes = write_pdf_from_source_with_visibility(
            source,
            "test.poentaro",
            &ResolvedPartVisibility::default(),
            &fonts::FontBytesByFamily::embedded(),
            &[],
            NoteLabels::Degrees,
        )
        .unwrap();
        assert_eq!(
            extract_embedded_source_from_pdf(&pdf_bytes).as_deref(),
            Some(source)
        );
    }

    #[test]
    fn score_only_pdf_has_no_embedded_source() {
        let source = r#"metadata { title "Testing" },

parts ["Melody": notes],

score """
    [Melody] 1 2 3 4
    """,
"#;
        let score = compile(source, "test.poentaro", &[]).unwrap();
        let svgs = render_svgs(&score).unwrap();
        let pdf_bytes = pdf::write_pdf(&svgs, &fonts::FontBytesByFamily::embedded(), None).unwrap();
        assert_eq!(extract_embedded_source_from_pdf(&pdf_bytes), None);
    }
}
