//! Reads `preset "Name" { … }`.

use itertools::Itertools;

use super::read_metadata_field;
use crate::ast::parsed::{AbbreviationReference, PartDecl, PresetKey};
use crate::error::{RecoverableError, Span};
use crate::parser::anyfix::decoded::{
    decode_error, invalid, span, AbbreviationText, FieldName, PresetSection,
};
use crate::parser::preset_parser::{
    collect_presets, finish_preset, known_parts, parse_notation, ParsedPresetSection,
    ParsedPresets, PresetFields,
};
use crate::parser::section::PresetName;
use poentaro_grammar::{Field, Value};

pub(super) fn read_presets(presets: &[PresetSection], declarations: &[PartDecl]) -> ParsedPresets {
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

struct NamedPreset<'p, 'd> {
    name: PresetName,
    preset: &'p PresetSection<'d>,
}

fn read_preset(named: NamedPreset, declarations: &[PartDecl]) -> ParsedPresetSection {
    let NamedPreset { name, preset } = named;
    let mut fields = PresetFields::default();
    let mut errors = Vec::new();
    for field in &preset.fields.field {
        match field {
            Ok(field) if PresetKey::from_keyword(&field.key()).is_some() => {
                read_preset_field(&mut fields, field, declarations, &mut errors);
            }
            _ => read_metadata_field(&mut fields.overrides, field, &mut errors),
        }
    }
    let header_span = Span::new(preset.whole.start, preset.name.span.end);
    finish_preset(name, header_span, fields, errors)
}

fn read_preset_field(
    fields: &mut PresetFields,
    field: &Field,
    declarations: &[PartDecl],
    errors: &mut Vec<RecoverableError>,
) {
    match PresetKey::from_keyword(&field.key()) {
        Some(PresetKey::VisibleParts) => {
            fields.visible_parts = Some(part_list(field, declarations, errors));
        }
        Some(PresetKey::SoloedParts) => {
            fields.soloed_parts = part_list(field, declarations, errors);
        }
        Some(PresetKey::Notation) => {
            fields.notation = parse_notation(field, errors).or(fields.notation.take());
        }
        None => {}
    }
}

/// `[S, A1]`, as the declared parts it names.
fn part_list(
    field: &Field,
    declarations: &[PartDecl],
    errors: &mut Vec<RecoverableError>,
) -> Vec<AbbreviationReference> {
    let Value::List(elements) = &field.arguments else {
        errors.push(invalid(field.arguments.span(), "expected `[ … ]` here"));
        return Vec::new();
    };
    let references = elements
        .value
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
