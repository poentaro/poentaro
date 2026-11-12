use crate::document::anyx_errors;
use crate::parser::read_document;
use crate::parser::sequence_parser::PartFilterKind;
use crate::part_info::SourcePartDeclaration;
use crate::source_edit::{parse_metadata_fields, MetadataFields};

fn error_messages(source: &str) -> Vec<String> {
    anyx_errors(source, &[])
        .iter()
        .map(crate::error::RecoverableError::message)
        .collect()
}

fn declarations(source: &str) -> Vec<SourcePartDeclaration> {
    crate::list_part_declarations_from_source(source, "test.poentaro", &[])
        .unwrap()
        .into_iter()
        .map(|declaration| SourcePartDeclaration {
            line_number: 0,
            ..declaration
        })
        .collect()
}

fn metadata_debug(fields: &MetadataFields) -> String {
    format!("{:?} {:?}", fields.metadata, fields.directive_row_offset)
}

#[test]
fn metadata_reads_like_the_section_syntax() {
    let anyx = r#"metadata {
        title : "Song",
        max measures per system : 4,
        hide resting parts : no,
        title style : { font size : 32, bold : yes, font family : sans serif },
        directive row offset : (0, -4),
    }, parts [], score """#;
    let section_syntax = r#"metadata {
    title "Song",
    max measures per system 4,
    hide resting parts: no,
    title style { font size 32, bold: yes, font family: sans serif },
    directive row offset (0, - 4),
},

parts [],

score """
    """,
"#;
    assert_eq!(error_messages(anyx), [] as [String; 0]);
    assert_eq!(
        metadata_debug(&parse_metadata_fields(anyx, None)),
        metadata_debug(&parse_metadata_fields(section_syntax, None))
    );
}

#[test]
fn parts_read_like_the_section_syntax() {
    let anyx = r#"parts [
        "Violin" [V] : notes { octave : -1, volume : 80%, instrument : violin },
        "Echo" [E] : follow [V] { volume : 50% },
        "Words" [Vv1] : lyrics [V],
        "Snare" : percussion { sound : 38 },
        "Alto 1 & Tenor" [A1&T] : notes,
    ], score """#;
    let section_syntax = r#"parts [
    "Violin" [V]: notes { instrument: violin, volume: 80 %, octave: - 1 },
    "Echo" [E]: follow [V] { volume: 50 % },
    "Words" [Vv1]: lyrics [V],
    "Snare": percussion { sound 38 },
    "Alto 1 & Tenor" [A1 & T]: notes,
],

score """
    """,
"#;
    assert_eq!(error_messages(anyx), [] as [String; 0]);
    assert_eq!(declarations(anyx), declarations(section_syntax));
}

#[test]
fn a_sequence_entry_reads_its_filter() {
    let read = read_document(
        r#"sequence [ Verse, Chorus WITHOUT [S, A2], "B" ONLY [S], "Last verse" ]"#,
        &[],
    );
    let entries = read
        .sequence
        .unwrap()
        .entries
        .into_iter()
        .map(|entry| {
            let filter = entry.part_filter.map(|filter| (filter.kind, filter.parts));
            (entry.label, filter)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        entries,
        [
            ("Verse".to_owned(), None),
            (
                "Chorus".to_owned(),
                Some((PartFilterKind::Omit, vec!["S".to_owned(), "A2".to_owned()]))
            ),
            (
                "B".to_owned(),
                Some((PartFilterKind::Only, vec!["S".to_owned()]))
            ),
            ("Last verse".to_owned(), None),
        ]
    );
}

#[test]
fn a_preset_reads_its_part_lists_and_notation() {
    let read = read_document(
        r#"parts [ "M" [M] : notes, "B" [B] : notes ],
        preset "Solo" { visible parts : [M, B], soloed parts : [], notation : cello },
        score """#,
        &[],
    );
    let preset = read.presets.presets.first().unwrap();
    assert_eq!(preset.name, "Solo");
    let abbreviations = preset
        .visible_parts
        .iter()
        .flatten()
        .map(|part| part.abbreviation.as_str())
        .collect::<Vec<_>>();
    assert_eq!(abbreviations, ["M", "B"]);
    assert_eq!(preset.notation, Some(crate::ast::parsed::Notation::Cello));
}

#[test]
fn the_score_is_the_strings_own_text_at_its_own_offset() {
    let source = "parts [],\nscore \"\"\"\n    [M] 1 2 3 // first\n    \"\"\",\n";
    let score = read_document(source, &[]).score;
    let in_score = score.text.find("[M]").unwrap();
    assert_eq!(score.offset + in_score, source.find("[M]").unwrap());
}

#[test]
fn an_empty_score_string_is_an_empty_score() {
    let read = read_document(r#"parts [], score """#, &[]);
    assert!(read.score.text.trim().is_empty());
    assert_eq!(error_messages(r#"parts [], score """#), [] as [String; 0]);
}

#[test]
fn a_score_that_is_not_a_string_is_reported() {
    assert_eq!(
        error_messages("parts [], score [ 1 ]"),
        ["found `[...]`; expected a string"]
    );
}

#[test]
fn an_unknown_section_is_reported() {
    assert_eq!(
        error_messages("lyrics []"),
        ["found the word `lyrics`; expected the word `could`, `metadata`, `parts`, `sequence`, `preset` or `score`"]
    );
}

#[test]
fn an_unknown_part_option_is_reported() {
    assert_eq!(
        error_messages(r#"parts [ "Piano" [P] : notes { loudness : 3 } ], score """#),
        ["found the word `loudness`; expected the word `instrument`, `range`, `sound`, `volume` or `octave`"]
    );
}

#[test]
fn an_unknown_metadata_field_is_reported_as_written() {
    let messages = read_document(
        r#"metadata { notes font size : 14 }, parts [], score """#,
        &[],
    )
    .metadata_parse_errors
    .iter()
    .map(crate::error::RecoverableError::message)
    .collect::<Vec<_>>();
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("notes font size"), "{messages:?}");
}

#[test]
fn anyx_syntax_errors_are_reported() {
    assert_eq!(error_messages("parts [").len(), 1);
}

#[test]
fn what_could_not_be_converted_is_reported() {
    let messages = error_messages(
        r#"could not convert "S = notes+lyrics", parts [ could not convert "T = notes+lyrics" ], score """#,
    );
    assert_eq!(messages.len(), 2, "{messages:?}");
}

#[test]
fn a_comment_with_one_hash_is_reported_and_the_rest_still_reads() {
    let source = "# My song\nparts [ \"S\" [S] : notes ],  # trailing\nscore \"\"\"\n  [S] 1 2 3 4 # 5\n  \"\"\",\n";
    assert_eq!(
        error_messages(source),
        [
            "found a `#` line; a comment starts with `##`",
            "found a `#` line; a comment starts with `##`"
        ]
    );
    let errors = anyx_errors(source, &[]);
    assert_eq!(
        errors
            .iter()
            .map(|error| &source[error.span.start..error.span.end])
            .collect::<Vec<_>>(),
        ["# My song", "# trailing"]
    );
    assert_eq!(declarations(source).len(), 1);
}

#[test]
fn a_comment_with_two_hashes_reads() {
    let source = "## My song\nparts [ \"S\" [S] : notes ],  ## trailing\nscore \"\"\"\n  [S] 1 2 3 4\n  \"\"\",\n";
    assert_eq!(error_messages(source), Vec::<String>::new());
}
