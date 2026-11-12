//! Reads `metadata { … }` and the metadata fields of a preset.

use itertools::Itertools;

use crate::ast::parsed::{
    FontFamilyChoice, MetadataFlagKey, MetadataNumberKey, MetadataTextKey, Offset, ParsedMetadata,
    PresetKey, TextStyle, TextStyleComponent, TextStyleComponentValue, TextStyleKind, YesNo,
    DIRECTIVE_ROW_OFFSET_KEY,
};
use crate::error::RecoverableError;
use crate::parser::anyfix::decoded::{decode_error, invalid, span, FieldName, Signed, ValueText};
use poentaro_grammar::{DecodeError, Field, Record, Value};

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

pub(super) fn read_metadata(record: &Record) -> (ParsedMetadata, Vec<RecoverableError>) {
    let mut metadata = ParsedMetadata::default();
    let mut errors = Vec::new();
    record
        .field
        .iter()
        .for_each(|field| read_metadata_field(&mut metadata, field, &mut errors));
    (metadata, errors)
}

/// Reads one `name : value` field into `metadata`.
pub(crate) fn read_metadata_field(
    metadata: &mut ParsedMetadata,
    field: &Result<Field, DecodeError>,
    errors: &mut Vec<RecoverableError>,
) {
    let field = match field {
        Ok(field) => field,
        Err(error) => return errors.push(decode_error(error)),
    };
    if let Some(error) = field.unconverted_error() {
        errors.push(error);
    } else if let Value::Record(style_fields) = &field.arguments {
        read_style(metadata, field, style_fields, errors);
    } else if let Err(error) = read_plain_field(metadata, field) {
        errors.push(error);
    }
}

fn read_plain_field(metadata: &mut ParsedMetadata, field: &Field) -> Result<(), RecoverableError> {
    let name = field.written_key();
    let value = &field.arguments;
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
            Some(key) => *metadata.text_mut(key) = Some(value.text()?),
            None => {
                return Err(RecoverableError::metadata_malformed_line(
                    span(value.span()),
                    &value.text()?,
                ))
            }
        },
        None if PresetKey::from_keyword(&field.key()).is_some() => {
            return Err(RecoverableError::preset_only_field(field.key_span(), &name));
        }
        None => {
            return Err(RecoverableError::metadata_unknown_field(
                field.key_span(),
                &name,
            ))
        }
    }
    Ok(())
}

fn read_non_negative(name: &str, value: &Value) -> Result<u32, RecoverableError> {
    let number = match value {
        Value::Integer(integer) => u32::try_from(integer.value()).ok(),
        _ => None,
    };
    number.ok_or_else(|| {
        value.text().map_or_else(
            |error| error,
            |text| RecoverableError::metadata_invalid_integer(span(value.span()), name, &text),
        )
    })
}

fn read_positive(name: &str, value: &Value) -> Result<u32, RecoverableError> {
    match read_non_negative(name, value)? {
        0 => Err(RecoverableError::metadata_must_be_positive(
            span(value.span()),
            name,
        )),
        number => Ok(number),
    }
}

fn read_offset(name: &str, value: &Value) -> Result<Offset, RecoverableError> {
    let offset = match value {
        Value::Pair { x, y, .. } => offset_component(x).zip(offset_component(y)),
        _ => None,
    };
    let Some((x, y)) = offset else {
        return Err(RecoverableError::metadata_invalid_integer_pair(
            span(value.span()),
            name,
            &value.text()?,
        ));
    };
    Ok(Offset { x, y })
}

fn offset_component(value: &Value) -> Option<i32> {
    match value {
        Value::Integer(integer) => i32::try_from(integer.value()).ok(),
        _ => None,
    }
}

fn read_flag(name: &str, value: &Value) -> Result<bool, RecoverableError> {
    let text = value.text()?;
    YesNo::from_keyword(&text)
        .map(YesNo::to_bool)
        .ok_or_else(|| RecoverableError::metadata_invalid_boolean(span(value.span()), name, &text))
}

fn read_font_family(name: &str, value: &Value) -> Result<FontFamilyChoice, RecoverableError> {
    let text = value.text()?;
    FontFamilyChoice::from_keyword(&text).ok_or_else(|| {
        let allowed = FontFamilyChoice::ALL
            .iter()
            .map(|family| family.keyword())
            .join(", ");
        RecoverableError::metadata_invalid_enum(span(value.span()), name, &text, &allowed)
    })
}

fn read_style_component(
    component: TextStyleComponent,
    name: &str,
    value: &Value,
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
    style_fields: &Record,
    errors: &mut Vec<RecoverableError>,
) {
    let key = field.key();
    let Some(kind_keyword) = key.strip_suffix("_style") else {
        errors.push(invalid(
            style_fields.span,
            format!(
                "only a text style takes `{{ … }}`; did you mean `{} style`?",
                field.written_key()
            ),
        ));
        return;
    };
    let Some(kind) = TextStyleKind::from_keyword(kind_keyword) else {
        errors.push(RecoverableError::metadata_unknown_field(
            field.key_span(),
            &field.written_key(),
        ));
        return;
    };
    let style = metadata.style_mut(kind);
    for style_field in &style_fields.field {
        let result = match style_field {
            Ok(style_field) => read_style_field(style, field, style_field),
            Err(error) => Err(decode_error(error)),
        };
        if let Err(error) = result {
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
        .ok_or_else(|| RecoverableError::metadata_unknown_field(style_field.key_span(), &name))?;
    style.set_component(read_style_component(
        component,
        &name,
        &style_field.arguments,
    )?);
    Ok(())
}

#[cfg(test)]
#[path = "metadata_layout_tests.rs"]
mod metadata_layout_tests;
#[cfg(test)]
#[path = "metadata_tests.rs"]
mod metadata_tests;
