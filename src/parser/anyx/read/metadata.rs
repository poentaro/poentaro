//! Reads `metadata { … }` and the metadata fields of a preset. The grammar
//! checks each key and the kind of value it takes; this reads the values into
//! [`ParsedMetadata`].

use crate::ast::parsed::{
    FontFamilyChoice, MetadataFlagKey, MetadataNumberKey, MetadataTextKey, Offset, ParsedMetadata,
    TextStyle, TextStyleComponentValue, TextStyleKind,
};
use crate::error::RecoverableError;
use crate::parser::anyx::decoded::{decode_error, invalid, unconverted_error_at};
use poentaro_grammar::{
    DecodeError, FlagField, FontFamily, Keyed, MetadataField, MetadataRecord, NumberField,
    SignedInteger, Spanned, StyleField, StyleRecord, TextField, TextStyle as TextStyleField,
};

pub(super) fn read_metadata(record: &MetadataRecord) -> (ParsedMetadata, Vec<RecoverableError>) {
    let mut metadata = ParsedMetadata::default();
    let mut errors = Vec::new();
    for field in &record.metadata_field {
        read_metadata_field(&mut metadata, field.as_ref(), &mut errors);
    }
    (metadata, errors)
}

/// Reads one field into `metadata`.
pub(crate) fn read_metadata_field(
    metadata: &mut ParsedMetadata,
    field: Result<&MetadataField, &DecodeError>,
    errors: &mut Vec<RecoverableError>,
) {
    let result = match field {
        Err(error) => Err(decode_error(error)),
        Ok(MetadataField::Text(field)) => {
            *metadata.text_mut(text_key(field)) = Some(field.text().value.value.clone());
            Ok(())
        }
        Ok(MetadataField::Style(field)) => {
            read_style(metadata.style_mut(style_kind(field)), field.style(), errors);
            Ok(())
        }
        Ok(MetadataField::Number(field)) => to_u32(field, field.number())
            .map(|number| *metadata.number_mut(number_key(field)) = Some(number)),
        Ok(MetadataField::Flag(field)) => {
            *metadata.flag_mut(flag_key(field)) = Some(field.flag().is_yes());
            Ok(())
        }
        Ok(field @ MetadataField::DirectiveRowOffset { offset, .. }) => to_i32(field, &offset.x)
            .and_then(|x| {
                let y = to_i32(field, &offset.y)?;
                metadata.directive_row_offset = Some(Offset { x, y });
                Ok(())
            }),
        Ok(MetadataField::CouldNotConvert { text, span, .. }) => {
            Err(unconverted_error_at(*span, &text.value.value))
        }
    };
    if let Err(error) = result {
        errors.push(error);
    }
}

fn text_key(field: &TextField) -> MetadataTextKey {
    match field {
        TextField::Title { .. } => MetadataTextKey::Title,
        TextField::Subtitle { .. } => MetadataTextKey::Subtitle,
        TextField::Author { .. } => MetadataTextKey::Author,
    }
}

fn style_kind(field: &TextStyleField) -> TextStyleKind {
    match field {
        TextStyleField::TitleStyle { .. } => TextStyleKind::Title,
        TextStyleField::SubtitleStyle { .. } => TextStyleKind::Subtitle,
        TextStyleField::AuthorStyle { .. } => TextStyleKind::Author,
        TextStyleField::SequenceStyle { .. } => TextStyleKind::Sequence,
        TextStyleField::PartLegendStyle { .. } => TextStyleKind::PartLegend,
        TextStyleField::MeasureNumberStyle { .. } => TextStyleKind::MeasureNumber,
        TextStyleField::SectionLabelStyle { .. } => TextStyleKind::SectionLabel,
        TextStyleField::PartLabelStyle { .. } => TextStyleKind::PartLabel,
        TextStyleField::PageNumberStyle { .. } => TextStyleKind::PageNumber,
        TextStyleField::RunningTitleStyle { .. } => TextStyleKind::RunningTitle,
        TextStyleField::LyricsStyle { .. } => TextStyleKind::Lyrics,
        TextStyleField::NotesStyle { .. } => TextStyleKind::Notes,
        TextStyleField::ChordsStyle { .. } => TextStyleKind::Chords,
        TextStyleField::NoteDashStyle { .. } => TextStyleKind::NoteDash,
    }
}

fn number_key(field: &NumberField) -> MetadataNumberKey {
    match field {
        NumberField::RowHeight { .. } => MetadataNumberKey::RowHeight,
        NumberField::MaxMeasuresPerSystem { .. } => MetadataNumberKey::MaxMeasuresPerSystem,
        NumberField::NoteNumberWidth { .. } => MetadataNumberKey::NoteNumberWidth,
        NumberField::PartsListColumns { .. } => MetadataNumberKey::PartsListColumns,
        NumberField::PartLabelWidthPt { .. } => MetadataNumberKey::PartLabelWidthPt,
        NumberField::PageMarginPt { .. } => MetadataNumberKey::PageMarginPt,
    }
}

fn flag_key(field: &FlagField) -> MetadataFlagKey {
    match field {
        FlagField::MergeDuplicateMeasuresAcrossParts { .. } => {
            MetadataFlagKey::MergeDuplicateMeasuresAcrossParts
        }
        FlagField::HideRestingParts { .. } => MetadataFlagKey::HideRestingParts,
        FlagField::HideSystemDividers { .. } => MetadataFlagKey::HideSystemDividers,
    }
}

/// A number the grammar allows that a `u32` can't hold.
fn to_u32<N: Copy + TryInto<u32> + std::fmt::Display>(
    field: &dyn Keyed,
    number: &Spanned<N>,
) -> Result<u32, RecoverableError> {
    number
        .value
        .try_into()
        .map_err(|_| too_large(field, number.span, number.value))
}

fn to_i32(field: &dyn Keyed, number: &SignedInteger) -> Result<i32, RecoverableError> {
    i32::try_from(number.value).map_err(|_| too_large(field, number.span, number.value))
}

fn too_large(
    field: &dyn Keyed,
    at: anyx::Span,
    number: impl std::fmt::Display,
) -> RecoverableError {
    invalid(
        at,
        format!(
            "found {number} for `{}`, which is too large",
            field.key_words()
        ),
    )
}

/// `lyrics style { font size 14, … }`.
fn read_style(style: &mut TextStyle, record: &StyleRecord, errors: &mut Vec<RecoverableError>) {
    for field in &record.style_field {
        let component = field
            .as_ref()
            .map_err(decode_error)
            .and_then(style_component);
        match component {
            Ok(component) => style.set_component(component),
            Err(error) => errors.push(error),
        }
    }
}

fn style_component(field: &StyleField) -> Result<TextStyleComponentValue, RecoverableError> {
    Ok(match field {
        StyleField::FontSize { size, .. } => {
            TextStyleComponentValue::FontSize(Some(to_u32(field, size)?))
        }
        StyleField::HorizontalPaddingPt { points, .. } => {
            TextStyleComponentValue::HorizontalPaddingPt(Some(to_u32(field, points)?))
        }
        StyleField::VerticalPaddingPt { points, .. } => {
            TextStyleComponentValue::VerticalPaddingPt(Some(to_u32(field, points)?))
        }
        StyleField::Bold { flag, .. } => TextStyleComponentValue::Bold(Some(flag.is_yes())),
        StyleField::Italic { flag, .. } => TextStyleComponentValue::Italic(Some(flag.is_yes())),
        StyleField::Underline { flag, .. } => {
            TextStyleComponentValue::Underline(Some(flag.is_yes()))
        }
        StyleField::FontFamily { family, .. } => {
            TextStyleComponentValue::FontFamily(Some(match family {
                FontFamily::Serif(_) => FontFamilyChoice::Serif,
                FontFamily::SansSerif { .. } => FontFamilyChoice::SansSerif,
                FontFamily::Monospace(_) => FontFamilyChoice::Monospace,
            }))
        }
    })
}

#[cfg(test)]
#[path = "metadata_layout_tests.rs"]
mod metadata_layout_tests;
#[cfg(test)]
#[path = "metadata_tests.rs"]
mod metadata_tests;
