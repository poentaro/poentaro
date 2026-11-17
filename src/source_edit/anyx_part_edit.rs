//! The part edit of a document written in anyx expressions: rewrites the kind
//! of one part in the tree and formats only the `parts` section.

use crate::ast::parsed::{PitchRange, RangeInstrument};
use crate::error::Span;
use crate::parser::anyx::decoded::{decode, span, AbbreviationText};
use crate::parser::anyx::{abbreviation, part_kind_text, PartKindText};
use crate::parser::parts_parser::{
    DEFAULT_PART_OCTAVE_OFFSET, DEFAULT_PART_VOLUME, PART_SETTING_LIMITS,
};
use poentaro_grammar::Part;

use super::anyx_edit::{formatted_section, replace_span, text_of};
use super::{PartMode, PartSettings};

pub(super) fn update_part_declaration(
    source: &str,
    abbreviation: &str,
    settings: &PartSettings,
) -> Option<String> {
    let parts = decode(source).document.parts?;
    let kind = parts
        .parts
        .part
        .iter()
        .find_map(|part| match part.as_ref().ok()? {
            Part::Abbreviated {
                abbreviation: written,
                kind,
                ..
            } => (written.text().ok()? == abbreviation).then_some(kind),
            Part::Named { display, kind, .. } => {
                (display.value.value == abbreviation).then_some(kind)
            }
            Part::Unconverted(_) => None,
        })?;
    let whole = span(parts.span);
    let kind = span(kind.span());
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
