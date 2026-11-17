use super::*;
use crate::parser::parse;

fn span_text(source: &str, span: Span) -> &str {
    &source[span.start..span.end]
}

#[test]
fn part_abbreviation_declaration_and_score_reference_are_collected() {
    let source = r#"metadata { title "t" },

parts ["Soprano" [S]: notes],

score """
    time=4/4 key=C4 bpm=120
    [S] 1 2 3 4
    """,
"#;
    let doc = parse(source, "test.poentaro", &[]).unwrap();
    let symbols = collect_symbols(&doc);
    let symbol = symbols
        .iter()
        .find(|s| s.kind == SymbolKind::Abbreviation && s.name == "S")
        .expect("expected symbol S");

    assert_eq!(symbol.occurrences.len(), 2);
    assert!(symbol
        .occurrences
        .iter()
        .any(|o| o.role == OccurrenceRole::Declaration && span_text(source, o.span) == "S"));
    assert!(symbol
        .occurrences
        .iter()
        .any(|o| o.role == OccurrenceRole::Reference && span_text(source, o.span) == "S"));
}

#[test]
fn section_label_declaration_and_sequence_reference_are_collected() {
    let source = r#"metadata { title "t" },

parts ["Soprano" [S]: notes],

sequence [Verse, Verse],

score """
    time=4/4 key=C4 bpm=120 label="Verse"
    1 2 3 4
    """,
"#;
    let doc = parse(source, "test.poentaro", &[]).unwrap();
    let symbols = collect_symbols(&doc);
    let label = symbols
        .iter()
        .find(|s| s.kind == SymbolKind::SectionLabel && s.name == "Verse")
        .expect("expected symbol Verse");

    assert_eq!(
        label.occurrences.len(),
        3,
        "1 declaration + 2 sequence refs"
    );
    let declaration = label
        .occurrences
        .iter()
        .find(|o| o.role == OccurrenceRole::Declaration)
        .expect("expected a declaration occurrence");
    assert_eq!(span_text(source, declaration.span), "Verse");
    assert_eq!(
        span_text(source, declaration.hit_span),
        r#"label="Verse""#,
        "hit_span should cover the whole label=\"...\" token, not just the quoted text"
    );
}

#[test]
fn sequence_omit_parts_reference_is_collected() {
    let source = r#"metadata { title "t" },

parts ["Soprano" [S]: notes],

sequence [Verse WITHOUT [S]],

score """
    time=4/4 key=C4 bpm=120 label="Verse"
    1 2 3 4
    """,
"#;
    let doc = parse(source, "test.poentaro", &[]).unwrap();
    let symbols = collect_symbols(&doc);
    let soprano = symbols
        .iter()
        .find(|s| s.kind == SymbolKind::Abbreviation && s.name == "S")
        .expect("expected symbol S");

    assert!(soprano
        .occurrences
        .iter()
        .any(|o| o.role == OccurrenceRole::Reference && span_text(source, o.span) == "S"));
}

#[test]
fn sequence_only_parts_reference_is_collected() {
    let source = r#"metadata { title "t" },

parts ["Soprano" [S]: notes],

sequence [Verse ONLY [S]],

score """
    time=4/4 key=C4 bpm=120 label="Verse"
    1 2 3 4
    """,
"#;
    let doc = parse(source, "test.poentaro", &[]).unwrap();
    let symbols = collect_symbols(&doc);
    let soprano = symbols
        .iter()
        .find(|s| s.kind == SymbolKind::Abbreviation && s.name == "S")
        .expect("expected symbol S");

    assert!(soprano
        .occurrences
        .iter()
        .any(|o| o.role == OccurrenceRole::Reference && span_text(source, o.span) == "S"));
}

#[test]
fn rename_edits_produce_replacement_for_every_occurrence() {
    let source = r#"metadata { title "t" },

parts ["Soprano" [S]: notes],

score """
    time=4/4 key=C4 bpm=120
    [S] 1 2 3 4
    """,
"#;
    let doc = parse(source, "test.poentaro", &[]).unwrap();
    let edits = rename_edits(&doc, SymbolKind::Abbreviation, "S", "Sop");

    assert_eq!(edits.len(), 2);
    for edit in &edits {
        assert_eq!(edit.replacement, "Sop");
        assert_eq!(span_text(source, edit.span), "S");
    }
}

#[test]
fn rename_edits_returns_empty_for_unknown_symbol() {
    let source = r#"metadata { title "t" },

parts ["Soprano" [S]: notes],

score """
    time=4/4 key=C4 bpm=120
    [S] 1 2 3 4
    """,
"#;
    let doc = parse(source, "test.poentaro", &[]).unwrap();
    let edits = rename_edits(&doc, SymbolKind::Abbreviation, "Nope", "X");
    assert!(edits.is_empty());
}
