use crate::ast::parsed::{ParsedDocument, ParsedMetadata, PartDecl};
use crate::error::{IrrecoverableError, RecoverableError};

pub mod anyfix;
pub mod lyrics;
pub mod metadata_parser;
pub mod parts_parser;
pub mod preset_parser;
pub mod score;
pub mod section;
pub mod sequence_parser;

/// A document's score, as the text the score parser reads, and where that text
/// starts in the document. Spans in it are spans in the document.
pub(crate) struct ScoreText {
    pub text: String,
    pub offset: usize,
}

/// Everything in a document but the score's content.
pub(crate) struct ReadDocument {
    pub metadata: ParsedMetadata,
    pub metadata_parse_errors: Vec<RecoverableError>,
    pub declarations: Vec<PartDecl>,
    pub parts_parse_errors: Vec<RecoverableError>,
    pub section_structure_errors: Vec<RecoverableError>,
    pub score: ScoreText,
    pub presets: preset_parser::ParsedPresets,
    pub sequence: Option<sequence_parser::SequenceSection>,
    pub sequence_parse_errors: Vec<RecoverableError>,
}

pub(crate) use anyfix::read as read_document;

pub fn parse(
    input: &str,
    filename: &str,
    instruments: &[parts_parser::InstrumentInfo],
) -> Result<ParsedDocument, IrrecoverableError> {
    let path = std::path::Path::new(filename);
    let read = read_document(input, instruments);

    let (tracks, directive_events_per_measure, per_measure_parse_errors, abbreviation_references) =
        if read.declarations.is_empty() {
            (Vec::new(), Vec::new(), Vec::new(), Vec::new())
        } else {
            score::interleaved_parser::parse(
                &read.score.text,
                read.score.offset,
                &read.declarations,
            )
            .map_err(|error| error.with_path(path))?
        };

    Ok(ParsedDocument {
        metadata: read.metadata,
        declarations: read.declarations,
        tracks,
        directive_events_per_measure,
        abbreviation_references,
        per_measure_parse_errors,
        metadata_parse_errors: read.metadata_parse_errors,
        parts_parse_errors: read.parts_parse_errors,
        section_structure_errors: read.section_structure_errors,
        sequence: read.sequence,
        sequence_parse_errors: read.sequence_parse_errors,
        presets: read.presets.presets,
        preset_diagnostics: read.presets.diagnostics,
    })
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod relative_key_tests;
