//! Reads `preset "Name" { … }`.

use anyfix_expr::{Node, NodeKind};
use itertools::Itertools;

use super::read_metadata_field;
use crate::ast::parsed::{AbbreviationReference, PartDecl, PresetKey};
use crate::error::{RecoverableError, Span};
use crate::parser::anyfix::tree::{
    abbreviation_text, as_field, invalid, list_of, name_span, record_of, span_of, value_text,
    Field, PresetNode,
};
use crate::parser::metadata_parser::MetadataLine;
use crate::parser::preset_parser::{
    collect_presets, finish_preset, known_parts, parse_notation, ParsedPresetSection,
    ParsedPresets, PresetFields,
};
use crate::parser::section::PresetName;

pub(super) fn read_presets(presets: &[PresetNode], declarations: &[PartDecl]) -> ParsedPresets {
    let named = presets
        .iter()
        .map(|preset| NamedPreset {
            name: preset_name(preset.name),
            preset,
        })
        .collect_vec();
    collect_presets(
        named,
        |preset| &preset.name,
        |preset| read_preset(preset, declarations),
    )
}

struct NamedPreset<'p, 'n> {
    name: PresetName,
    preset: &'p PresetNode<'n>,
}

fn preset_name(node: &Node) -> PresetName {
    let text = match &node.kind {
        NodeKind::String(literal) => literal.value.clone(),
        _ => String::new(),
    };
    PresetName {
        text,
        span: span_of(node),
    }
}

fn read_preset(named: NamedPreset, declarations: &[PartDecl]) -> ParsedPresetSection {
    let NamedPreset { name, preset } = named;
    let mut fields = PresetFields::default();
    let mut errors = Vec::new();
    if !matches!(preset.name.kind, NodeKind::String(_)) {
        errors.push(invalid(preset.name, "a preset name must be a string"));
    }
    match record_of(preset.fields) {
        Ok(elements) => elements
            .iter()
            .for_each(|element| read_field(&mut fields, element, declarations, &mut errors)),
        Err(error) => errors.push(error),
    }
    let header_span = Span::new(preset.whole.span.start, preset.name.span.end);
    finish_preset(name, header_span, fields, errors)
}

fn read_field(
    fields: &mut PresetFields,
    node: &Node,
    declarations: &[PartDecl],
    errors: &mut Vec<RecoverableError>,
) {
    let Some(field) =
        as_field(node).filter(|field| PresetKey::from_keyword(&field.key()).is_some())
    else {
        read_metadata_field(&mut fields.overrides, node, errors);
        return;
    };
    match PresetKey::from_keyword(&field.key()) {
        Some(PresetKey::VisibleParts) => {
            fields.visible_parts = Some(part_list(&field, declarations, errors));
        }
        Some(PresetKey::SoloedParts) => {
            fields.soloed_parts = part_list(&field, declarations, errors);
        }
        Some(PresetKey::Notation) => match value_text(field.value) {
            Ok(value) => {
                let line = MetadataLine {
                    key: &field.key(),
                    written_key: &field.written_key(),
                    value: &value,
                    key_span: field.key_span,
                    value_span: span_of(field.value),
                };
                fields.notation = parse_notation(&line, errors).or(fields.notation.take());
            }
            Err(error) => errors.push(error),
        },
        None => {}
    }
}

/// `[S, A1]`, as the declared parts it names.
fn part_list(
    field: &Field,
    declarations: &[PartDecl],
    errors: &mut Vec<RecoverableError>,
) -> Vec<AbbreviationReference> {
    let elements = match list_of(field.value) {
        Ok(elements) => elements,
        Err(error) => {
            errors.push(error);
            return Vec::new();
        }
    };
    let references = elements
        .iter()
        .filter_map(|element| match abbreviation_text(element) {
            Ok(abbreviation) => Some(AbbreviationReference {
                abbreviation,
                span: name_span(element),
            }),
            Err(error) => {
                errors.push(error);
                None
            }
        })
        .collect();
    known_parts(references, declarations, errors)
}
