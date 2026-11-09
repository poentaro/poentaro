use super::*;
use std::io::Read;
use zip::ZipArchive;

fn multi_track_input() -> &'static str {
    r#"metadata { title "test score", author "tester" },

parts [
    "Soprano 1" [S1]: notes,
    "Soprano 1 lyrics" [S1v1]: lyrics [S1],
    "Soprano 2" [S2]: notes,
    "Soprano 2 lyrics" [S2v1]: lyrics [S2],
],

score """
    time=4/4 key=C4 bpm=120
    [S1] 1 2 3 4
    [S1v1] do re mi fa
    [S2] 5 6 7 1
    [S2v1] sol la ti do
    """,
"#
}

#[test]
fn write_split_pdfs_from_source_produces_one_pdf_per_track() {
    let entries = write_split_pdfs_from_source(
        multi_track_input(),
        "test.poentaro",
        "test_split",
        SplitSelection::default(),
        &fonts::FontBytesByFamily::embedded(),
        NoteLabels::Degrees,
    )
    .unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].track_name, "S1");
    assert_eq!(entries[0].filename, "test_split - Soprano 1.pdf");
    assert_eq!(entries[1].track_name, "S2");
    assert_eq!(entries[1].filename, "test_split - Soprano 2.pdf");
    assert_eq!(&entries[0].pdf[0..4], b"%PDF");
    assert_eq!(&entries[1].pdf[0..4], b"%PDF");
}

#[test]
fn write_split_pdfs_from_source_keeps_the_presets_parts_and_names_files_after_it() {
    let input = format!(
        r#"{}
preset "Soprano 2" {{ visible parts [S2, S2v1] }},
"#,
        multi_track_input()
    );
    let entries = write_split_pdfs_from_source(
        &input,
        "test.poentaro",
        "test_split",
        SplitSelection {
            tracks: &[],
            preset_name: Some("Soprano 2"),
        },
        &fonts::FontBytesByFamily::embedded(),
        NoteLabels::Degrees,
    )
    .unwrap();
    let filenames: Vec<&str> = entries
        .iter()
        .map(|entry| entry.filename.as_str())
        .collect();
    assert_eq!(filenames, ["test_split (Soprano 2) - Soprano 2.pdf"]);
}

#[test]
fn write_split_pdfs_from_source_single_part_uses_split_naming() {
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#;
    let entries = write_split_pdfs_from_source(
        input,
        "test.poentaro",
        "song",
        SplitSelection::default(),
        &fonts::FontBytesByFamily::embedded(),
        NoteLabels::Degrees,
    )
    .unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].filename, "song - Melody.pdf");
    assert_eq!(&entries[0].pdf[0..4], b"%PDF");
}

#[test]
fn write_split_pdfs_from_source_no_sections_returns_empty() {
    // Source with no section headers: section-structure errors are recoverable,
    // so no Err is returned; the missing parts section means no tracks exist.
    let entries = write_split_pdfs_from_source(
        "not valid",
        "test.poentaro",
        "song",
        SplitSelection::default(),
        &fonts::FontBytesByFamily::embedded(),
        NoteLabels::Degrees,
    )
    .unwrap();
    assert!(entries.is_empty());
}

// `svg2pdf` embeds font subsets using a randomized-hasher `HashSet`/`HashMap`
// internally, so byte-for-byte PDF output is not deterministic across calls
// even for identical input SVGs — only content-driven properties like output
// length are stable. These tests compare a part declared with a display name
// distinct from its abbreviation (which adds a legend row, e.g. "S — Soprano")
// against an otherwise-identical score whose part has no distinct display
// name (abbreviation == display_name, so no legend row is emitted, per
// `build_header` in lib.rs), and assert the legend-bearing PDF is larger.

fn with_legend_input() -> &'static str {
    r#"metadata { title "t", author "a" },

parts ["Soprano" [S]: notes],

score """
    time=4/4 key=C4 bpm=120
    [S] 1 2 3 4
    """,
"#
}

fn without_legend_input() -> &'static str {
    r#"metadata { title "t", author "a" },

parts ["S": notes],

score """
    time=4/4 key=C4 bpm=120
    [S] 1 2 3 4
    """,
"#
}

#[test]
fn write_pdf_from_source_with_visibility_includes_part_legend() {
    let with_legend_pdf = write_pdf_from_source_with_visibility(
        with_legend_input(),
        "test.poentaro",
        &ResolvedPartVisibility::default(),
        &fonts::FontBytesByFamily::embedded(),
        &[],
        NoteLabels::Degrees,
    )
    .unwrap();
    let without_legend_pdf = write_pdf_from_source_with_visibility(
        without_legend_input(),
        "test.poentaro",
        &ResolvedPartVisibility::default(),
        &fonts::FontBytesByFamily::embedded(),
        &[],
        NoteLabels::Degrees,
    )
    .unwrap();
    assert!(
        with_legend_pdf.len() > without_legend_pdf.len(),
        "PDF for a part with a distinct display name ({} bytes) should be larger \
         than one without a legend row ({} bytes)",
        with_legend_pdf.len(),
        without_legend_pdf.len()
    );
}

#[test]
fn write_split_pdfs_from_source_includes_part_legend() {
    let with_legend_entries = write_split_pdfs_from_source(
        with_legend_input(),
        "test.poentaro",
        "song",
        SplitSelection::default(),
        &fonts::FontBytesByFamily::embedded(),
        NoteLabels::Degrees,
    )
    .unwrap();
    let without_legend_entries = write_split_pdfs_from_source(
        without_legend_input(),
        "test.poentaro",
        "song",
        SplitSelection::default(),
        &fonts::FontBytesByFamily::embedded(),
        NoteLabels::Degrees,
    )
    .unwrap();
    assert_eq!(with_legend_entries.len(), 1);
    assert_eq!(without_legend_entries.len(), 1);
    assert!(
        with_legend_entries[0].pdf.len() > without_legend_entries[0].pdf.len(),
        "split PDF for a part with a distinct display name ({} bytes) should be larger \
         than one without a legend row ({} bytes)",
        with_legend_entries[0].pdf.len(),
        without_legend_entries[0].pdf.len()
    );
}

#[test]
fn zip_split_pdfs_contains_named_entries() {
    let entries = write_split_pdfs_from_source(
        multi_track_input(),
        "test.poentaro",
        "test_split",
        SplitSelection::default(),
        &fonts::FontBytesByFamily::embedded(),
        NoteLabels::Degrees,
    )
    .unwrap();
    let zip_bytes = zip_split_pdfs(&entries).unwrap();
    assert_eq!(&zip_bytes[0..2], b"PK");

    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).unwrap();
    assert_eq!(archive.len(), 2);
    let mut names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "test_split - Soprano 1.pdf".to_string(),
            "test_split - Soprano 2.pdf".to_string()
        ]
    );

    let mut first = archive.by_name("test_split - Soprano 1.pdf").unwrap();
    let mut buf = Vec::new();
    first.read_to_end(&mut buf).unwrap();
    assert_eq!(&buf[0..4], b"%PDF");
}
