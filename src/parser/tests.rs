use super::*;
use crate::ast::parsed::{ParsedTimedTrack, ParsedTrack};
use crate::error::DocumentSection;

fn notes_track(doc: &ParsedDocument) -> &ParsedTimedTrack {
    doc.tracks
        .iter()
        .find_map(|t| match t {
            ParsedTrack::Timed(n) if n.lyrics.is_none() && n.abbreviation != "Chord" => Some(n),
            ParsedTrack::Timed(_) => None,
        })
        .or_else(|| {
            doc.tracks
                .iter()
                .map(|t| match t {
                    ParsedTrack::Timed(n) => n,
                })
                .next()
        })
        .expect("expected a notes track")
}

fn lyrics_track<'a>(doc: &'a ParsedDocument, abbreviation: &str) -> &'a ParsedTimedTrack {
    doc.tracks
        .iter()
        .map(|t| match t {
            ParsedTrack::Timed(n) => n,
        })
        .find(|n| n.abbreviation == abbreviation)
        .expect("expected a lyric track")
}

#[test]
fn parses_full_document() {
    let input = r#"metadata { title "hello world", author "foo" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] 你好wo rld
    """,
"#;
    let doc = parse(input, "test.poentaro", &[]).unwrap();
    assert_eq!(doc.metadata.title, Some("hello world".to_string()));
    assert_eq!(doc.metadata.author, Some("foo".to_string()));
    assert_eq!(doc.declarations.len(), 2);
    assert_eq!(doc.tracks.len(), 2);
    let notes = notes_track(&doc);
    let event_count: usize = notes
        .measure_slots
        .iter()
        .filter_map(|s| match s {
            crate::ast::parsed::ParsedMeasureSlot::Real { events } => Some(events.len()),
            crate::ast::parsed::ParsedMeasureSlot::EmptyNote { .. } => None,
        })
        .sum();
    assert_eq!(event_count, 7);
    let verse = lyrics_track(&doc, "Melodyv1");
    assert_eq!(verse.lyrics.as_ref().unwrap().measure_syllables[0].len(), 4);
}

#[test]
fn an_unknown_top_level_form_is_reported() {
    let input = "unknown { foo : 1 },\n\nscore \"\",\n";
    let doc = parse(input, "test.poentaro", &[]).expect("an unknown form must not abort parsing");
    assert!(doc
        .section_structure_errors
        .iter()
        .any(|e| e.message().contains("found `unknown _` at the top level")));
}

#[test]
fn duplicate_score_section_recoverable() {
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    """,

score """
    [Melody] 5 6 7 1
    """,
"#;
    let doc =
        parse(input, "test.poentaro", &[]).expect("duplicate score section must not abort parsing");
    assert!(doc
        .section_structure_errors
        .iter()
        .any(|e| matches!(&e.kind, crate::error::RecoverableErrorKind::SectionDuplicate { section } if *section == DocumentSection::Score)));
}

#[test]
fn sections_may_appear_in_any_order() {
    let input = r#"score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    """,

metadata { title "hello world" },

parts ["Melody": notes],
"#;
    let doc =
        parse(input, "test.poentaro", &[]).expect("out-of-order sections must not abort parsing");
    assert!(
        !doc.section_structure_errors.iter().any(|e| matches!(
            &e.kind,
            crate::error::RecoverableErrorKind::SectionMissing { .. }
                | crate::error::RecoverableErrorKind::SectionDuplicate { .. }
        )),
        "out-of-order sections must not produce a structure error: {:?}",
        doc.section_structure_errors
    );
    assert_eq!(doc.metadata.title, Some("hello world".to_string()));
    assert_eq!(doc.declarations.len(), 1);
    assert_eq!(doc.tracks.len(), 1);
}

#[test]
fn missing_metadata_section_is_allowed() {
    let input = r#"parts ["Melody": notes],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    """,
"#;
    let doc = parse(input, "test.poentaro", &[])
        .expect("missing metadata section must not abort parsing");
    assert!(
        !doc.section_structure_errors.iter().any(|e| matches!(
            &e.kind,
            crate::error::RecoverableErrorKind::SectionMissing { section }
                if *section == DocumentSection::Metadata
        )),
        "missing metadata must not produce an error"
    );
    assert_eq!(doc.metadata.title, None);
}

#[test]
fn parses_two_named_parts() {
    let input = r#"metadata { title "t", author "a" },

parts ["Soprano": notes, "Alto": notes],

score """
    time=4/4 key=C4 bpm=120
    [Soprano] 1 2 3 4
    [Alto] 5 6 7 1
    """,
"#;
    let doc = parse(input, "test.poentaro", &[]).unwrap();
    assert_eq!(doc.tracks.len(), 2);
    let soprano = doc
        .tracks
        .iter()
        .find_map(|t| match t {
            ParsedTrack::Timed(n) if n.abbreviation == "Soprano" => Some(n),
            ParsedTrack::Timed(_) => None,
        })
        .unwrap();
    let alto = doc
        .tracks
        .iter()
        .find_map(|t| match t {
            ParsedTrack::Timed(n) if n.abbreviation == "Alto" => Some(n),
            ParsedTrack::Timed(_) => None,
        })
        .unwrap();
    // Only a lyric part carries lyrics.
    assert!(soprano.lyrics.is_none());
    assert!(alto.lyrics.is_none());
}

#[test]
fn too_many_lines_recoverable_error_span_points_to_absolute_file_position() {
    // One notes part but two data lines in a group → recoverable error.
    // The error span must point to the extra line's position in the *full* input.
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    [Melody] 1 2 3 4
    [Melody] 5 6 7 1
    """,
"#;
    let expected_offset = input.rfind("5 6 7 1").unwrap();
    let doc = parse(input, "test.poentaro", &[]).expect("too-many-lines must not abort parsing");
    let error = doc.per_measure_parse_errors[0]
        .as_ref()
        .expect("recoverable error must be recorded for the measure");
    assert_eq!(
        error.span().start,
        expected_offset,
        "recoverable error span should point to the absolute file position of the extra line"
    );
}

#[test]
fn too_many_lines_recoverable_error_lists_declared_parts() {
    // One notes part but two data lines → recoverable error should name the declared part.
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes],

score """
    [Melody] 1 2 3 4
    [Melody] 5 6 7 1
    """,
"#;
    let doc = parse(input, "test.poentaro", &[]).expect("too-many-lines must not abort parsing");
    let error = doc.per_measure_parse_errors[0]
        .as_ref()
        .expect("recoverable error must be recorded for the measure");
    assert!(
        error.message().contains("Melody"),
        "recoverable error message should list the declared part 'Melody', got: {}",
        error.message()
    );
}

#[test]
fn single_unnamed_part_remains_compatible() {
    let input = r#"metadata { title "t", author "a" },

parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody]],

score """
    time=4/4 key=C4 bpm=120
    [Melody] 1 2 3 4
    [Melodyv1] a b c d
    """,
"#;
    let doc = parse(input, "test.poentaro", &[]).unwrap();
    assert_eq!(doc.tracks.len(), 2);
    let notes = notes_track(&doc);
    assert_eq!(notes.abbreviation, "Melody");
    assert!(notes.lyrics.is_none());
    assert!(lyrics_track(&doc, "Melodyv1").lyrics.is_some());
}

#[test]
fn accepts_zero_vertical_and_horizontal_padding() {
    let source =
        "metadata { title style : { vertical padding pt : 0, horizontal padding pt : 0 } },\n";
    let read = read_document(source, &[]);
    assert!(read.metadata_parse_errors.is_empty());
    assert_eq!(read.metadata.title_style.vertical_padding_pt, Some(0));
    assert_eq!(read.metadata.title_style.horizontal_padding_pt, Some(0));
}
