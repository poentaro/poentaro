//! Reads `metadata { … }` and the metadata fields of a preset.

use anyfix_expr::{Node, NodeKind};
use itertools::Itertools;

use crate::ast::parsed::{
    FontFamilyChoice, MetadataFlagKey, MetadataNumberKey, MetadataTextKey, Offset, ParsedMetadata,
    PresetKey, TextStyle, TextStyleComponent, TextStyleComponentValue, TextStyleKind, YesNo,
    DIRECTIVE_ROW_OFFSET_KEY,
};
use crate::error::RecoverableError;
use crate::parser::anyfix::tree::{
    as_field, invalid, record_of, signed_number, span_of, unconverted_error, value_text,
    whole_number, Field,
};

/// A recognized `metadata` key, classified by the value it takes.
enum MetadataKey {
    Number(MetadataNumberKey),
    Flag(MetadataFlagKey),
    DirectiveRowOffset,
    /// A text-style kind, which for `title`/`subtitle`/`author` may also take
    /// the text itself.
    Style(TextStyleKind),
}

impl MetadataKey {
    fn from_keyword(key: &str) -> Option<Self> {
        MetadataNumberKey::from_keyword(key)
            .map(Self::Number)
            .or_else(|| MetadataFlagKey::from_keyword(key).map(Self::Flag))
            .or_else(|| (key == DIRECTIVE_ROW_OFFSET_KEY).then_some(Self::DirectiveRowOffset))
            .or_else(|| TextStyleKind::from_keyword(key).map(Self::Style))
    }
}

pub(super) fn read_metadata(body: &Node) -> (ParsedMetadata, Vec<RecoverableError>) {
    let mut metadata = ParsedMetadata::default();
    let mut errors = Vec::new();
    match record_of(body) {
        Ok(fields) => fields
            .iter()
            .for_each(|field| read_metadata_field(&mut metadata, field, &mut errors)),
        Err(error) => errors.push(error),
    }
    (metadata, errors)
}

/// Reads one `name : value` field into `metadata`.
pub(crate) fn read_metadata_field(
    metadata: &mut ParsedMetadata,
    node: &Node,
    errors: &mut Vec<RecoverableError>,
) {
    if let Some(error) = unconverted_error(node) {
        errors.push(error);
        return;
    }
    let Some(field) = as_field(node) else {
        errors.push(invalid(node, "expected a `name : value` field"));
        return;
    };
    if let NodeKind::Braces(style_fields) = &field.value.kind {
        read_style(metadata, &field, style_fields, errors);
    } else if let Err(error) = read_plain_field(metadata, &field) {
        errors.push(error);
    }
}

fn read_plain_field(metadata: &mut ParsedMetadata, field: &Field) -> Result<(), RecoverableError> {
    let name = field.written_key();
    let value = field.value;
    match MetadataKey::from_keyword(&field.key()) {
        Some(MetadataKey::Number(key)) => {
            *metadata.number_mut(key) = Some(read_positive(&name, value)?);
        }
        Some(MetadataKey::Flag(key)) => {
            *metadata.flag_mut(key) = Some(read_flag(&name, value)?);
        }
        Some(MetadataKey::DirectiveRowOffset) => {
            metadata.directive_row_offset = Some(read_offset(&name, value)?);
        }
        Some(MetadataKey::Style(kind)) => match MetadataTextKey::from_style_kind(kind) {
            Some(key) => *metadata.text_mut(key) = Some(written(value)?),
            None => {
                return Err(RecoverableError::metadata_malformed_line(
                    span_of(value),
                    &written(value)?,
                ))
            }
        },
        None if PresetKey::from_keyword(&field.key()).is_some() => {
            return Err(RecoverableError::preset_only_field(field.key_span, &name));
        }
        None => {
            return Err(RecoverableError::metadata_unknown_field(
                field.key_span,
                &name,
            ))
        }
    }
    Ok(())
}

/// `value` as a message quotes it, or why it can't be a value at all.
fn written(value: &Node) -> Result<String, RecoverableError> {
    match &value.kind {
        NodeKind::Parens(pair) => match pair.as_slice() {
            [x, y] => Ok(format!("{} {}", value_text(x)?, value_text(y)?)),
            _ => Err(invalid(value, "expected a pair, like `(0, -4)`")),
        },
        _ => value_text(value),
    }
}

fn read_non_negative(name: &str, value: &Node) -> Result<u32, RecoverableError> {
    whole_number(value).ok_or_else(|| {
        written(value).map_or_else(
            |error| error,
            |text| RecoverableError::metadata_invalid_integer(span_of(value), name, &text),
        )
    })
}

fn read_positive(name: &str, value: &Node) -> Result<u32, RecoverableError> {
    match read_non_negative(name, value)? {
        0 => Err(RecoverableError::metadata_must_be_positive(
            span_of(value),
            name,
        )),
        number => Ok(number),
    }
}

fn read_offset(name: &str, value: &Node) -> Result<Offset, RecoverableError> {
    let offset = match &value.kind {
        NodeKind::Parens(pair) => match pair.as_slice() {
            [x, y] => signed_number(x).zip(signed_number(y)),
            _ => None,
        },
        _ => None,
    };
    let Some((x, y)) = offset else {
        return Err(RecoverableError::metadata_invalid_integer_pair(
            span_of(value),
            name,
            &written(value)?,
        ));
    };
    Ok(Offset { x, y })
}

fn read_flag(name: &str, value: &Node) -> Result<bool, RecoverableError> {
    let text = written(value)?;
    YesNo::from_keyword(&text)
        .map(YesNo::to_bool)
        .ok_or_else(|| RecoverableError::metadata_invalid_boolean(span_of(value), name, &text))
}

fn read_font_family(name: &str, value: &Node) -> Result<FontFamilyChoice, RecoverableError> {
    let text = written(value)?;
    FontFamilyChoice::from_keyword(&text).ok_or_else(|| {
        let allowed = FontFamilyChoice::ALL
            .iter()
            .map(|family| family.keyword())
            .join(", ");
        RecoverableError::metadata_invalid_enum(span_of(value), name, &text, &allowed)
    })
}

fn read_style_component(
    component: TextStyleComponent,
    name: &str,
    value: &Node,
) -> Result<TextStyleComponentValue, RecoverableError> {
    Ok(match component {
        TextStyleComponent::FontSize => {
            TextStyleComponentValue::FontSize(Some(read_positive(name, value)?))
        }
        TextStyleComponent::HorizontalPaddingPt => {
            TextStyleComponentValue::HorizontalPaddingPt(Some(read_non_negative(name, value)?))
        }
        TextStyleComponent::VerticalPaddingPt => {
            TextStyleComponentValue::VerticalPaddingPt(Some(read_non_negative(name, value)?))
        }
        TextStyleComponent::Bold => TextStyleComponentValue::Bold(Some(read_flag(name, value)?)),
        TextStyleComponent::Italic => {
            TextStyleComponentValue::Italic(Some(read_flag(name, value)?))
        }
        TextStyleComponent::Underline => {
            TextStyleComponentValue::Underline(Some(read_flag(name, value)?))
        }
        TextStyleComponent::FontFamily => {
            TextStyleComponentValue::FontFamily(Some(read_font_family(name, value)?))
        }
    })
}

/// `lyrics style : { font size : 14, … }`.
fn read_style(
    metadata: &mut ParsedMetadata,
    field: &Field,
    style_fields: &[Node],
    errors: &mut Vec<RecoverableError>,
) {
    let key = field.key();
    let Some(kind_keyword) = key.strip_suffix("_style") else {
        errors.push(invalid(
            field.value,
            format!(
                "only a text style takes `{{ … }}`; did you mean `{} style`?",
                field.written_key()
            ),
        ));
        return;
    };
    let Some(kind) = TextStyleKind::from_keyword(kind_keyword) else {
        errors.push(RecoverableError::metadata_unknown_field(
            field.key_span,
            &field.written_key(),
        ));
        return;
    };
    let style = metadata.style_mut(kind);
    for node in style_fields {
        let Some(style_field) = as_field(node) else {
            errors.push(invalid(node, "expected a `name : value` field"));
            continue;
        };
        if let Err(error) = read_style_field(style, field, &style_field) {
            errors.push(error);
        }
    }
}

fn read_style_field(
    style: &mut TextStyle,
    field: &Field,
    style_field: &Field,
) -> Result<(), RecoverableError> {
    let name = format!("{} {}", field.written_key(), style_field.written_key());
    let component = TextStyleComponent::from_keyword(&style_field.key())
        .ok_or_else(|| RecoverableError::metadata_unknown_field(style_field.key_span, &name))?;
    style.set_component(read_style_component(component, &name, style_field.value)?);
    Ok(())
}

#[cfg(test)]
#[path = "metadata_layout_tests.rs"]
mod metadata_layout_tests;
#[cfg(test)]
#[path = "metadata_tests.rs"]
mod metadata_tests;
