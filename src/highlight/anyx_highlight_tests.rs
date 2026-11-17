use super::{highlight_tokens, section_headers};
use crate::highlight::HighlightKind;
use crate::parser::section::SectionKind;

const DOCUMENT: &str = r#"## The song
metadata { title : "Hello", title style : { bold : yes } },

parts [
  "Piano" [P] : notes { volume : 80% },
  "Echo" [E] : follow [P],
],

preset "Solo" { visible parts : [P] },

score """
  bpm=92 label="Verse"
  [P] 1 2 3 4
  """,
"#;

fn words(kind: HighlightKind) -> Vec<&'static str> {
    highlight_tokens(DOCUMENT)
        .into_iter()
        .filter(|token| token.kind == kind)
        .map(|token| &DOCUMENT[token.span.start..token.span.end])
        .collect()
}

#[test]
fn section_keywords_are_the_headers() {
    let headers = section_headers(DOCUMENT);
    assert_eq!(
        headers.iter().map(|header| header.kind).collect::<Vec<_>>(),
        [
            SectionKind::Metadata,
            SectionKind::Parts,
            SectionKind::Preset,
            SectionKind::Score
        ]
    );
    assert_eq!(
        headers
            .iter()
            .map(|header| &DOCUMENT[header.span.start..header.span.end])
            .collect::<Vec<_>>(),
        ["metadata", "parts", "preset", "score"]
    );
}

#[test]
fn nothing_outside_the_score_string_is_marked() {
    let score_start = DOCUMENT.find(r#"""""#).unwrap();
    assert!(highlight_tokens(DOCUMENT)
        .iter()
        .all(|token| token.span.start > score_start));
}

#[test]
fn directive_keywords_in_the_score_string_are_marked() {
    assert_eq!(words(HighlightKind::DirectiveKey), ["bpm", "label"]);
}

#[test]
fn a_lyric_reading_is_highlighted_from_its_parenthesis() {
    let source = "parts [\"M\" [M] : notes, \"L\" [L] : lyrics [M]],\n\nscore \"\"\"\n  [M] 1 2 3 4\n  [L] 去银行(háng) 我(xyz)\n  \"\"\",\n";
    let readings = highlight_tokens(source)
        .into_iter()
        .filter(|token| token.kind == HighlightKind::LyricReading)
        .map(|token| &source[token.span.start..token.span.end])
        .collect::<Vec<_>>();
    assert_eq!(readings, ["(háng)"]);
}
