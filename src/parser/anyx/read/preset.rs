//! Reads `preset "Name" { … }`.

use itertools::Itertools;

use super::read_metadata_field;
use crate::ast::parsed::{AbbreviationReference, Notation, PartDecl};
use crate::error::{RecoverableError, Span, Spanned};
use crate::parser::anyx::decoded::{decode_error, span, AbbreviationText};
use crate::parser::preset_parser::{
    collect_presets, finish_preset, known_parts, ParsedPresetSection, ParsedPresets, PresetFields,
};
use crate::parser::section::PresetName;
use poentaro_grammar::{DocumentPreset, Notation as NotationChoice, PartReferences, PresetField};

pub(super) fn read_presets(presets: &[DocumentPreset], declarations: &[PartDecl]) -> ParsedPresets {
    let named = presets
        .iter()
        .map(|preset| NamedPreset {
            name: PresetName {
                text: preset.name.value.value.clone(),
                span: span(preset.name.span),
            },
            preset,
        })
        .collect_vec();
    collect_presets(
        named,
        |preset| &preset.name,
        |preset| read_preset(preset, declarations),
    )
}

struct NamedPreset<'p> {
    name: PresetName,
    preset: &'p DocumentPreset,
}

fn read_preset(named: NamedPreset, declarations: &[PartDecl]) -> ParsedPresetSection {
    let NamedPreset { name, preset } = named;
    let mut fields = PresetFields::default();
    let mut errors = Vec::new();
    for field in &preset.fields.preset_field {
        match field {
            Err(error) => errors.push(decode_error(error)),
            Ok(PresetField::Metadata(field)) => {
                read_metadata_field(&mut fields.overrides, Ok(field), &mut errors);
            }
            Ok(PresetField::VisibleParts { parts, .. }) => {
                fields.visible_parts = Some(part_list(parts, declarations, &mut errors));
            }
            Ok(PresetField::SoloedParts { parts, .. }) => {
                fields.soloed_parts = part_list(parts, declarations, &mut errors);
            }
            Ok(PresetField::Notation { notation, .. }) => {
                fields.notation = Some(notation_of(notation));
            }
        }
    }
    let header_span = Span::new(preset.span.start, preset.name.span.end);
    finish_preset(name, header_span, fields, errors)
}

/// The notation a `notation` field names, `jianpu` being `numbered`'s old name.
pub(crate) fn notation_of(notation: &NotationChoice) -> Spanned<Notation> {
    let (value, at) = match notation {
        NotationChoice::Numbered(at) | NotationChoice::Jianpu(at) => (Notation::Numbered, at),
        NotationChoice::Violin(at) => (Notation::Violin, at),
        NotationChoice::Cello(at) => (Notation::Cello, at),
        NotationChoice::Abc(at) => (Notation::Abc, at),
        NotationChoice::Guitar(at) => (Notation::Guitar, at),
        NotationChoice::Harmonica(at) => (Notation::Harmonica, at),
    };
    Spanned::new(value, span(*at))
}

/// `[S, A1]`, as the declared parts it names.
fn part_list(
    parts: &PartReferences,
    declarations: &[PartDecl],
    errors: &mut Vec<RecoverableError>,
) -> Vec<AbbreviationReference> {
    let references = parts
        .abbreviation
        .iter()
        .filter_map(|element| {
            let reference = element.as_ref().map_err(decode_error).and_then(|element| {
                Ok(AbbreviationReference {
                    abbreviation: element.text()?,
                    span: element.text_span(),
                })
            });
            reference.map_err(|error| errors.push(error)).ok()
        })
        .collect();
    known_parts(references, declarations, errors)
}
