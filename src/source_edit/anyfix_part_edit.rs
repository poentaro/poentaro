//! The part edit of a document written in anyfix expressions: rewrites the kind
//! of one part in the tree and formats only the `parts` section.

use crate::ast::parsed::{PitchRange, RangeInstrument};
use crate::error::Span;
use crate::parser::anyfix::tree::{as_part, list_of, span_of};
use crate::parser::anyfix::{abbreviation, part_kind_text, PartKindText};
use crate::parser::parts_parser::{
    DEFAULT_PART_OCTAVE_OFFSET, DEFAULT_PART_VOLUME, PART_SETTING_LIMITS,
};

use super::anyfix_edit::{formatted_section, replace_span, sections_of, text_of, top_level_nodes};
use super::{PartMode, PartSettings};

pub(super) fn update_part_declaration(
    source: &str,
    abbreviation: &str,
    settings: &PartSettings,
) -> Option<String> {
    let nodes = top_level_nodes(source);
    let sections = sections_of(source, &nodes);
    let parts = sections.parts?;
    let part = list_of(parts.body)
        .ok()?
        .iter()
        .filter_map(as_part)
        .find(|part| {
            part.abbreviation_text()
                .is_some_and(|text| text == abbreviation)
        })?;
    let whole = span_of(parts.whole);
    let kind = span_of(part.kind);
    let edited = replace_span(
        text_of(source, whole),
        Span::new(kind.start - whole.start, kind.end - whole.start),
        &kind_text(settings),
    );
    Some(replace_span(source, whole, &formatted_section(&edited)))
}

fn kind_text(settings: &PartSettings) -> String {
    let volume = PART_SETTING_LIMITS
        .volume
        .clamp(i32::from(settings.volume))
        .value;
    let octave_offset = PART_SETTING_LIMITS
        .octave_offset
        .clamp(i32::from(settings.octave_offset))
        .value;
    part_kind_text(&PartKindText {
        head: match &settings.mode {
            PartMode::Chords => "chords".to_owned(),
            PartMode::Notes => "notes".to_owned(),
            PartMode::Percussion => "percussion".to_owned(),
            PartMode::Follow { target } => {
                format!("follow [{}]", abbreviation(target))
            }
            PartMode::Lyrics { target } => {
                return format!("lyrics [{}]", abbreviation(target));
            }
        },
        range_instrument: settings.range_instrument,
        written_range: matches!(settings.mode, PartMode::Notes | PartMode::Follow { .. })
            .then(|| {
                PitchRange::from_bounds(
                    settings.range_lowest,
                    settings.range_highest,
                    settings.range_instrument.map(RangeInstrument::range),
                )
            })
            .flatten(),
        sound: settings.program,
        volume: (volume != DEFAULT_PART_VOLUME).then_some(volume),
        octave_offset: (octave_offset != DEFAULT_PART_OCTAVE_OFFSET).then_some(octave_offset),
    })
}
