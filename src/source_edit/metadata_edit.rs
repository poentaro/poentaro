//! Reading and editing `metadata`, or a preset's overrides of it, as a whole,
//! for the web editor's Edit Metadata form, through [`super::anyfix_edit`].

use std::iter::once;

use itertools::Itertools;

use super::anyfix_edit;
use crate::ast::parsed::{
    MetadataFlagKey, MetadataNumberKey, MetadataTextKey, ParsedMetadata, TextStyle,
    TextStyleComponent, TextStyleComponentValue, TextStyleKind, YesNo, DIRECTIVE_ROW_OFFSET_KEY,
};

/// The edited section's current field values, as the Edit Metadata form
/// shows them.
#[derive(Debug, Default)]
pub struct MetadataFields {
    pub metadata: ParsedMetadata,
    /// `directive_row_offset`'s value exactly as written, even when it
    /// doesn't parse, so a half-typed `0 -` in the form's text input isn't
    /// wiped by the next re-parse. The parser still reports it as an error.
    pub directive_row_offset: Option<String>,
    /// What an unset field falls back to: the song's own `metadata` when
    /// editing a preset, nothing when editing `metadata` itself.
    pub inherited: ParsedMetadata,
}

/// One field edit from the Edit Metadata form. A `None` value removes the
/// field from the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataEdit {
    Text {
        key: MetadataTextKey,
        value: Option<String>,
    },
    Number {
        key: MetadataNumberKey,
        value: Option<u32>,
    },
    Flag {
        key: MetadataFlagKey,
        value: Option<bool>,
    },
    DirectiveRowOffset(Option<String>),
    Style {
        kind: TextStyleKind,
        value: TextStyleComponentValue,
    },
    /// Removes every component of one kind's style.
    ResetStyle(TextStyleKind),
}

fn non_blank(value: Option<String>) -> Option<String> {
    value.filter(|text| !text.trim().is_empty())
}

impl MetadataEdit {
    pub(super) fn apply(self, fields: &mut MetadataFields) {
        match self {
            Self::Text { key, value } => *fields.metadata.text_mut(key) = non_blank(value),
            Self::Number { key, value } => *fields.metadata.number_mut(key) = value,
            Self::Flag { key, value } => *fields.metadata.flag_mut(key) = value,
            Self::DirectiveRowOffset(value) => {
                fields.directive_row_offset = non_blank(value).map(|text| text.trim().to_owned());
            }
            Self::Style { kind, value } => fields.metadata.style_mut(kind).set_component(value),
            Self::ResetStyle(kind) => *fields.metadata.style_mut(kind) = TextStyle::default(),
        }
    }
}

/// The field values of the preset called `preset_name`, or of `metadata`
/// when `None`; all-unset when the source has no such section.
pub fn parse_metadata_fields(source: &str, preset_name: Option<&str>) -> MetadataFields {
    anyfix_edit::parse_metadata_fields(source, preset_name)
}

/// One field in canonical order, with its value in the form
/// [`crate::parser::anyfix::field_text`] takes.
pub(super) struct CanonicalField {
    pub key: String,
    pub value: String,
}

fn field(key: &str, value: String) -> CanonicalField {
    CanonicalField {
        key: key.to_owned(),
        value,
    }
}

fn style_field(metadata: &ParsedMetadata, kind: TextStyleKind) -> Option<CanonicalField> {
    let style = metadata.style(kind);
    let components = TextStyleComponent::ALL
        .iter()
        .filter_map(|&component| {
            style
                .component_value(component)
                .source_text()
                .map(|value| format!("{}: {value}", component.keyword()))
        })
        .collect_vec();
    (!components.is_empty())
        .then(|| field(kind.keyword(), format!("{{ {} }}", components.join(", "))))
}

/// Canonical order: each text kind's content then its style, the numeric keys,
/// the remaining styles, the flags, then `directive_row_offset`.
pub(super) fn canonical_fields(fields: &MetadataFields) -> Vec<CanonicalField> {
    let metadata = &fields.metadata;
    let text_fields = TextStyleKind::ALL
        .iter()
        .filter_map(|&kind| MetadataTextKey::from_style_kind(kind))
        .flat_map(|key| {
            [
                metadata
                    .text(key)
                    .map(|text| field(key.keyword(), format!("\"{text}\""))),
                style_field(metadata, key.style_kind()),
            ]
        });
    let number_fields = MetadataNumberKey::ALL.iter().map(|&key| {
        metadata
            .number(key)
            .map(|number| field(key.keyword(), number.to_string()))
    });
    let other_style_fields = TextStyleKind::ALL
        .iter()
        .filter(|&&kind| MetadataTextKey::from_style_kind(kind).is_none())
        .map(|&kind| style_field(metadata, kind));
    let flag_fields = MetadataFlagKey::ALL.iter().map(|&key| {
        metadata
            .flag(key)
            .map(|flag| field(key.keyword(), YesNo::from_bool(flag).keyword().to_owned()))
    });
    let offset_field = fields
        .directive_row_offset
        .as_ref()
        .map(|offset| field(DIRECTIVE_ROW_OFFSET_KEY, offset.clone()));
    text_fields
        .chain(number_fields)
        .chain(other_style_fields)
        .chain(flag_fields)
        .chain(once(offset_field))
        .flatten()
        .collect()
}

/// Applies `edit` to the preset called `preset_name` (or to `metadata`
/// when `None`) and rewrites that section in canonical order, leaving every
/// other section untouched. Returns `source` unchanged when it has no such
/// section.
pub fn update_metadata_field(
    source: &str,
    preset_name: Option<&str>,
    edit: MetadataEdit,
) -> String {
    anyfix_edit::update_metadata_field(source, preset_name, edit)
}

#[cfg(test)]
#[path = "metadata_edit_tests.rs"]
mod metadata_edit_tests;
