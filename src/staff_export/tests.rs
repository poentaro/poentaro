use super::render_split_meis_from_source;
use crate::mei_export::ClefChoice;
use crate::split_track::SplitSelection;

const SOURCE: &str = r#"parts ["Soprano" [S]: notes, "Alto" [A]: notes, "Words" [w]: lyrics [A]],

score """
    key=C4 time=2/4
    [S] 1 2
    [A] 5 6
    [w] 你好
    """,
"#;

#[test]
fn split_meis_give_each_sounding_part_its_own_score_and_lyrics() {
    let entries = render_split_meis_from_source(
        SOURCE,
        "test.poentaro",
        "song",
        SplitSelection::default(),
        &[],
        ClefChoice::Automatic,
    )
    .unwrap();
    let summary: Vec<(&str, &str)> = entries
        .iter()
        .map(|entry| (entry.track_name.as_str(), entry.filename.as_str()))
        .collect();
    assert_eq!(
        summary,
        [("S", "song - Soprano.pdf"), ("A", "song - Alto.pdf")]
    );
    let soprano = &entries[0].mei;
    let alto = &entries[1].mei;
    assert!(soprano.contains("<label>Soprano</label>"), "{soprano}");
    assert!(!soprano.contains("<label>Alto</label>"), "{soprano}");
    assert!(!soprano.contains("<syl>"), "{soprano}");
    assert!(alto.contains("<label>Alto</label>"), "{alto}");
    assert!(!alto.contains("<label>Soprano</label>"), "{alto}");
    assert!(alto.contains("<syl>"), "{alto}");
}

#[cfg(feature = "pdf")]
mod pdf {
    use super::super::{write_staff_pdf, zip_staff_pdfs, StaffPdfPart};
    use crate::fonts::FontBytesByFamily;

    /// The shape of a Verovio `svgViewBox` page: no width/height, SMuFL
    /// glyphs as `<use>`d `<defs>`, and text asking for `Times, serif`.
    const VEROVIO_PAGE: &str = r##"<svg viewBox="0 0 840 1188" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" overflow="visible">
<defs><g id="E0A4"><path transform="scale(1,-1)" d="M0 0h100v100h-100z"/></g></defs>
<g class="page-margin" transform="translate(50, 50)">
<use xlink:href="#E0A4" transform="translate(100, 100) scale(0.72, 0.72)"/>
<text x="100" y="200" font-family="Times, serif" font-size="20px">你好 world</text>
</g></svg>"##;

    fn media_box(pdf: &[u8]) -> String {
        let text = String::from_utf8_lossy(pdf);
        let start = text.find("/MediaBox").expect("a page");
        text[start..start + text[start..].find(']').unwrap() + 1].to_string()
    }

    #[test]
    fn staff_pages_print_at_the_numbered_page_size_and_keep_the_source() {
        let fonts = FontBytesByFamily::embedded();
        let pdf = write_staff_pdf(&[VEROVIO_PAGE.to_string()], &fonts, Some("[S] 1")).unwrap();
        let numbered = crate::write_pdf_from_source_with_visibility(
            "[S] 1",
            "test.poentaro",
            &crate::ResolvedPartVisibility::default(),
            &fonts,
            &[],
            crate::NoteLabels::Degrees,
        )
        .unwrap();
        assert_eq!(media_box(&pdf), media_box(&numbered));
        assert_eq!(
            crate::source_embed::extract_embedded_source_from_pdf(&pdf).as_deref(),
            Some("[S] 1")
        );
    }

    #[test]
    fn verovio_text_tooltips_are_dropped() {
        let page = r#"<text><tspan x="10" text-anchor="middle"><title class="labelAttr">title</title><tspan>Song</tspan></tspan></text>"#;
        assert_eq!(
            super::super::without_text_tooltips(page),
            r#"<text><tspan x="10" text-anchor="middle"><tspan>Song</tspan></tspan></text>"#
        );
    }

    #[test]
    fn staff_text_is_drawn_with_an_embedded_font() {
        let fonts = FontBytesByFamily::embedded();
        let pdf = write_staff_pdf(&[VEROVIO_PAGE.to_string()], &fonts, None).unwrap();
        assert!(String::from_utf8_lossy(&pdf).contains("/FontFile"));
    }

    #[test]
    fn split_staff_pdfs_are_zipped_under_their_part_file_names() {
        let part = |name: &str| StaffPdfPart {
            track_name: name.to_string(),
            filename: format!("song - {name}.pdf"),
            pages: vec![VEROVIO_PAGE.to_string()],
        };
        let zip = zip_staff_pdfs(&[part("S"), part("A")], &FontBytesByFamily::embedded()).unwrap();
        let archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).unwrap();
        let names: Vec<&str> = archive.file_names().collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"song - S.pdf") && names.contains(&"song - A.pdf"));
    }
}
