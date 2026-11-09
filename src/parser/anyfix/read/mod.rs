//! Reads an anyfix document's tree into the document it describes.

mod metadata;
mod parts;
mod preset;
mod sequence;

use anyfix_expr::{Node, NodeKind};

use super::tree::{invalid, Sections};
use crate::ast::parsed::ParsedMetadata;
use crate::error::{DocumentSection, RecoverableError, Span};
use crate::parser::parts_parser::{InstrumentInfo, SourceRawPartDecl};
use crate::parser::{ReadDocument, ScoreText};

pub(crate) use metadata::read_metadata_field;

/// Reads `source`'s tree. `instruments` names the sound a `sound : N` option picks.
pub(crate) fn read(source: &str, instruments: &[InstrumentInfo]) -> ReadDocument {
    let parsed = anyfix_expr::parse(source);
    let nodes = parsed.syntax.nodes();
    let doc_span = Span::new(0, source.len());
    let sections = Sections::of(&nodes, doc_span);
    let syntax_errors = parsed.errors.iter().map(|error| {
        RecoverableError::anyfix_invalid(
            Span::new(error.span.start, error.span.end),
            error.message.clone(),
        )
    });
    let mut section_structure_errors = syntax_errors.collect::<Vec<_>>();
    section_structure_errors.extend(sections.errors.iter().cloned());

    let (metadata, metadata_parse_errors) = match sections.metadata {
        Some(section) => metadata::read_metadata(section.body),
        None => (ParsedMetadata::default(), Vec::new()),
    };
    if sections.parts.is_none() {
        section_structure_errors.push(RecoverableError::section_missing(
            doc_span,
            DocumentSection::Parts,
        ));
    }
    let (declarations, parts_parse_errors) = parts::read_parts(
        sections.parts.map(|section| section.body),
        doc_span,
        instruments,
    );
    if sections.score.is_none() {
        section_structure_errors.push(RecoverableError::section_missing(
            doc_span,
            DocumentSection::Score,
        ));
    }
    let score = sections
        .score
        .map_or(Ok(empty_score()), |section| {
            score_text(section.body, source)
        })
        .unwrap_or_else(|error| {
            section_structure_errors.push(error);
            empty_score()
        });
    let presets = preset::read_presets(&sections.presets, &declarations);
    let (sequence, sequence_parse_errors) = match sections.sequence {
        Some(section) => sequence::read_sequence(section.body),
        None => (None, Vec::new()),
    };
    ReadDocument {
        metadata,
        metadata_parse_errors,
        declarations,
        parts_parse_errors,
        section_structure_errors,
        score,
        presets,
        sequence,
        sequence_parse_errors,
    }
}

/// `source`'s part declarations before follow inheritance.
pub(crate) fn source_part_declarations(
    source: &str,
    errors: &mut Vec<RecoverableError>,
    instruments: &[InstrumentInfo],
) -> Vec<SourceRawPartDecl> {
    let nodes = anyfix_expr::parse(source).syntax.nodes();
    let sections = Sections::of(&nodes, Span::new(0, source.len()));
    let raw = sections
        .parts
        .map(|section| parts::raw_parts(section.body, instruments, errors))
        .unwrap_or_default();
    crate::parser::parts_parser::source_declarations(raw, source)
}

fn empty_score() -> ScoreText {
    ScoreText {
        text: String::new(),
        offset: 0,
    }
}

/// The score string's source text, which the score parser reads as it is. Its
/// spans are spans in the document. A comment in it is a `//` line, as in any
/// score, so it is blanked here, keeping offsets.
fn score_text(body: &Node, source: &str) -> Result<ScoreText, RecoverableError> {
    let NodeKind::String(literal) = &body.kind else {
        return Err(invalid(
            body,
            "the score is one string of measures, like `score \"\"\" … \"\"\"`",
        ));
    };
    if literal.pieces.iter().any(|piece| !piece.verbatim) {
        return Err(invalid(
            body,
            "write the score as a raw string, without escapes, like `score \"\"\" … \"\"\"`",
        ));
    }
    Ok(ScoreText {
        text: strip_comments(
            source
                .get(literal.content.start..literal.content.end)
                .unwrap_or_default(),
        ),
        offset: literal.content.start,
    })
}

/// Blanks out `//` line comments with spaces, preserving byte length and line
/// structure so downstream byte offsets (used in error spans) stay valid.
/// A `//` inside a double-quoted string (e.g. `title = "http://example.com"`)
/// does not start a comment.
fn strip_comments(input: &str) -> String {
    let mut out = Vec::with_capacity(input.len());
    let mut in_string = false;
    let mut in_comment = false;
    let mut bytes = input.as_bytes().iter().copied().peekable();
    while let Some(byte) = bytes.next() {
        match byte {
            b'\n' => {
                in_string = false;
                in_comment = false;
                out.push(byte);
            }
            b'"' if !in_comment => {
                in_string = !in_string;
                out.push(byte);
            }
            b'/' if !in_string && !in_comment && bytes.peek() == Some(&b'/') => {
                in_comment = true;
                out.push(b' ');
            }
            _ => out.push(if in_comment { b' ' } else { byte }),
        }
    }
    // Every replaced byte is either an ASCII space or an unmodified original
    // byte, so `out` stays valid UTF-8 even where a multi-byte character was
    // blanked out.
    String::from_utf8(out).unwrap_or_default()
}
