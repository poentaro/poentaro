use itertools::Itertools;

use crate::ast::parsed::{
    FontFamilyChoice, MetadataFlagKey, MetadataNumberKey, MetadataTextKey, Offset, ParsedMetadata,
    PresetKey, TextStyle, TextStyleComponent, TextStyleKind, YesNo, DIRECTIVE_ROW_OFFSET_KEY,
};
use crate::error::{RecoverableError, Span};

/// One non-blank `key = value` line of a `metadata` section. `value` is
/// trimmed and has its surrounding `"` quotes stripped.
pub struct MetadataLine<'a> {
    pub key: &'a str,
    /// The key as the document spells it, for messages: `notes font size` in
    /// anyfix, the same as `key` in the section syntax.
    pub written_key: &'a str,
    pub value: &'a str,
    pub key_span: Span,
    pub value_span: Span,
}

fn parse_positive_u32(key: &str, value: &str, value_span: Span) -> Result<u32, RecoverableError> {
    let parsed = value
        .parse::<u32>()
        .map_err(|_| RecoverableError::metadata_invalid_integer(value_span, key, value))?;
    if parsed == 0 {
        return Err(RecoverableError::metadata_must_be_positive(value_span, key));
    }
    Ok(parsed)
}

/// Like `parse_positive_u32`, but accepts `0` (e.g. a padding that is truly none).
fn parse_non_negative_u32(
    key: &str,
    value: &str,
    value_span: Span,
) -> Result<u32, RecoverableError> {
    value
        .parse::<u32>()
        .map_err(|_| RecoverableError::metadata_invalid_integer(value_span, key, value))
}

fn parse_offset(key: &str, value: &str, value_span: Span) -> Result<Offset, RecoverableError> {
    let parsed = match value.split_whitespace().collect_vec().as_slice() {
        [x, y] => x.parse::<i32>().ok().zip(y.parse::<i32>().ok()),
        _ => None,
    };
    parsed
        .map(|(x, y)| Offset { x, y })
        .ok_or_else(|| RecoverableError::metadata_invalid_integer_pair(value_span, key, value))
}

/// Formats an offset the way `directive_row_offset = x y` spells it.
pub fn format_offset(offset: Offset) -> String {
    format!("{} {}", offset.x, offset.y)
}

fn parse_bool(key: &str, value: &str, value_span: Span) -> Result<bool, RecoverableError> {
    YesNo::from_keyword(value)
        .map(YesNo::to_bool)
        .ok_or_else(|| RecoverableError::metadata_invalid_boolean(value_span, key, value))
}

fn parse_font_family(
    key: &str,
    value: &str,
    value_span: Span,
) -> Result<FontFamilyChoice, RecoverableError> {
    FontFamilyChoice::from_keyword(value).ok_or_else(|| {
        let allowed = FontFamilyChoice::ALL
            .iter()
            .map(|family| family.keyword())
            .join(", ");
        RecoverableError::metadata_invalid_enum(value_span, key, value, &allowed)
    })
}

/// Keeps `result`'s value, or records its error and yields `None`.
fn recorded<T>(
    result: Result<T, RecoverableError>,
    errors: &mut Vec<RecoverableError>,
) -> Option<T> {
    result.map_err(|error| errors.push(error)).ok()
}

use text_style_parser::{parse_component_value, parse_text_style_object};

#[path = "text_style_parser.rs"]
mod text_style_parser;

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

/// Applies one `metadata` key to `metadata`, recording a bad value or an
/// unknown key in `errors`.
pub(crate) fn apply_field(
    metadata: &mut ParsedMetadata,
    line: &MetadataLine,
    errors: &mut Vec<RecoverableError>,
) {
    let MetadataLine {
        key: keyword,
        written_key: key,
        value,
        key_span,
        value_span,
    } = *line;
    match MetadataKey::from_keyword(keyword) {
        Some(MetadataKey::Number(number_key)) => {
            if let Some(number) = recorded(parse_positive_u32(key, value, value_span), errors) {
                *metadata.number_mut(number_key) = Some(number);
            }
        }
        Some(MetadataKey::Flag(flag_key)) => {
            if let Some(flag) = recorded(parse_bool(key, value, value_span), errors) {
                *metadata.flag_mut(flag_key) = Some(flag);
            }
        }
        Some(MetadataKey::DirectiveRowOffset) => {
            if let Some(offset) = recorded(parse_offset(key, value, value_span), errors) {
                metadata.directive_row_offset = Some(offset);
            }
        }
        // `title`/`subtitle`/`author` take either their text or a style
        // object; every other kind only takes a style object.
        Some(MetadataKey::Style(kind)) => match MetadataTextKey::from_style_kind(kind) {
            Some(text_key) if !value.trim_start().starts_with('{') => {
                *metadata.text_mut(text_key) = Some(value.to_string());
            }
            _ => parse_text_style_object(
                metadata.style_mut(kind),
                key,
                key_span,
                value,
                &value_span,
                errors,
            ),
        },
        None if PresetKey::from_keyword(keyword).is_some() => {
            errors.push(RecoverableError::preset_only_field(key_span, key))
        }
        None => errors.push(RecoverableError::metadata_unknown_field(key_span, key)),
    }
}

/// Applies one field of a `<kind> style { … }` record. `field.written_key`
/// names the style's field in messages.
pub(crate) fn apply_text_style_field(
    style: &mut TextStyle,
    field: &MetadataLine,
    errors: &mut Vec<RecoverableError>,
) {
    let Some(component) = TextStyleComponent::from_keyword(field.key) else {
        errors.push(RecoverableError::metadata_unknown_field(
            field.key_span,
            field.written_key,
        ));
        return;
    };
    if let Some(value) = recorded(
        parse_component_value(component, field.written_key, field.value, field.value_span),
        errors,
    ) {
        style.set_component(value);
    }
}

#[cfg(test)]
#[path = "metadata_parser_layout_tests.rs"]
mod metadata_parser_layout_tests;
#[cfg(test)]
#[path = "metadata_parser_tests.rs"]
mod metadata_parser_tests;
