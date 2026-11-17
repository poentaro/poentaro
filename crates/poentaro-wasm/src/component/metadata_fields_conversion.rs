//! Conversions between `poentaro::source_edit::metadata_edit`'s
//! types and the WIT `metadata-fields`/`metadata-edit` family. Each enum is
//! converted by an exhaustive match, so a variant added on either side fails
//! to compile here.

use super::*;
use poentaro::ast::{grouped, parsed};
use poentaro::compositor::types::FontFamily;
use poentaro::{grouper, source_edit};

fn font_family_choice_to_wit(family: parsed::FontFamilyChoice) -> FontFamilyChoice {
    match family {
        parsed::FontFamilyChoice::Serif => FontFamilyChoice::Serif,
        parsed::FontFamilyChoice::SansSerif => FontFamilyChoice::SansSerif,
        parsed::FontFamilyChoice::Monospace => FontFamilyChoice::Monospace,
    }
}

fn font_family_choice_from_wit(family: FontFamilyChoice) -> parsed::FontFamilyChoice {
    match family {
        FontFamilyChoice::Serif => parsed::FontFamilyChoice::Serif,
        FontFamilyChoice::SansSerif => parsed::FontFamilyChoice::SansSerif,
        FontFamilyChoice::Monospace => parsed::FontFamilyChoice::Monospace,
    }
}

fn text_style_kind_to_wit(kind: parsed::TextStyleKind) -> TextStyleKind {
    match kind {
        parsed::TextStyleKind::Title => TextStyleKind::Title,
        parsed::TextStyleKind::Subtitle => TextStyleKind::Subtitle,
        parsed::TextStyleKind::Author => TextStyleKind::Author,
        parsed::TextStyleKind::Sequence => TextStyleKind::Sequence,
        parsed::TextStyleKind::PartLegend => TextStyleKind::PartLegend,
        parsed::TextStyleKind::MeasureNumber => TextStyleKind::MeasureNumber,
        parsed::TextStyleKind::SectionLabel => TextStyleKind::SectionLabel,
        parsed::TextStyleKind::PartLabel => TextStyleKind::PartLabel,
        parsed::TextStyleKind::PageNumber => TextStyleKind::PageNumber,
        parsed::TextStyleKind::RunningTitle => TextStyleKind::RunningTitle,
        parsed::TextStyleKind::Lyrics => TextStyleKind::Lyrics,
        parsed::TextStyleKind::Notes => TextStyleKind::Notes,
        parsed::TextStyleKind::Chords => TextStyleKind::Chords,
        parsed::TextStyleKind::NoteDash => TextStyleKind::NoteDash,
    }
}

fn text_style_kind_from_wit(kind: TextStyleKind) -> parsed::TextStyleKind {
    match kind {
        TextStyleKind::Title => parsed::TextStyleKind::Title,
        TextStyleKind::Subtitle => parsed::TextStyleKind::Subtitle,
        TextStyleKind::Author => parsed::TextStyleKind::Author,
        TextStyleKind::Sequence => parsed::TextStyleKind::Sequence,
        TextStyleKind::PartLegend => parsed::TextStyleKind::PartLegend,
        TextStyleKind::MeasureNumber => parsed::TextStyleKind::MeasureNumber,
        TextStyleKind::SectionLabel => parsed::TextStyleKind::SectionLabel,
        TextStyleKind::PartLabel => parsed::TextStyleKind::PartLabel,
        TextStyleKind::PageNumber => parsed::TextStyleKind::PageNumber,
        TextStyleKind::RunningTitle => parsed::TextStyleKind::RunningTitle,
        TextStyleKind::Lyrics => parsed::TextStyleKind::Lyrics,
        TextStyleKind::Notes => parsed::TextStyleKind::Notes,
        TextStyleKind::Chords => parsed::TextStyleKind::Chords,
        TextStyleKind::NoteDash => parsed::TextStyleKind::NoteDash,
    }
}

fn metadata_text_key_from_wit(key: MetadataTextKey) -> parsed::MetadataTextKey {
    match key {
        MetadataTextKey::Title => parsed::MetadataTextKey::Title,
        MetadataTextKey::Subtitle => parsed::MetadataTextKey::Subtitle,
        MetadataTextKey::Author => parsed::MetadataTextKey::Author,
    }
}

fn metadata_number_key_from_wit(key: MetadataNumberKey) -> parsed::MetadataNumberKey {
    match key {
        MetadataNumberKey::RowHeight => parsed::MetadataNumberKey::RowHeight,
        MetadataNumberKey::MaxMeasuresPerSystem => parsed::MetadataNumberKey::MaxMeasuresPerSystem,
        MetadataNumberKey::NoteNumberWidth => parsed::MetadataNumberKey::NoteNumberWidth,
        MetadataNumberKey::PartsListColumns => parsed::MetadataNumberKey::PartsListColumns,
        MetadataNumberKey::PartLabelWidthPt => parsed::MetadataNumberKey::PartLabelWidthPt,
        MetadataNumberKey::PageMarginPt => parsed::MetadataNumberKey::PageMarginPt,
    }
}

fn metadata_flag_key_from_wit(key: MetadataFlagKey) -> parsed::MetadataFlagKey {
    match key {
        MetadataFlagKey::MergeDuplicateMeasuresAcrossParts => {
            parsed::MetadataFlagKey::MergeDuplicateMeasuresAcrossParts
        }
        MetadataFlagKey::HideRestingParts => parsed::MetadataFlagKey::HideRestingParts,
        MetadataFlagKey::HideSystemDividers => parsed::MetadataFlagKey::HideSystemDividers,
    }
}

fn text_style_component_value_from_wit(
    value: TextStyleComponentValue,
) -> parsed::TextStyleComponentValue {
    match value {
        TextStyleComponentValue::FontSize(v) => parsed::TextStyleComponentValue::FontSize(v),
        TextStyleComponentValue::HorizontalPaddingPt(v) => {
            parsed::TextStyleComponentValue::HorizontalPaddingPt(v)
        }
        TextStyleComponentValue::VerticalPaddingPt(v) => {
            parsed::TextStyleComponentValue::VerticalPaddingPt(v)
        }
        TextStyleComponentValue::Bold(v) => parsed::TextStyleComponentValue::Bold(v),
        TextStyleComponentValue::Italic(v) => parsed::TextStyleComponentValue::Italic(v),
        TextStyleComponentValue::Underline(v) => parsed::TextStyleComponentValue::Underline(v),
        TextStyleComponentValue::FontFamily(v) => {
            parsed::TextStyleComponentValue::FontFamily(v.map(font_family_choice_from_wit))
        }
    }
}

fn text_style_fields_to_wit(style: &parsed::TextStyle) -> TextStyleFields {
    TextStyleFields {
        font_size: style.font_size,
        horizontal_padding_pt: style.horizontal_padding_pt,
        vertical_padding_pt: style.vertical_padding_pt,
        bold: style.bold,
        italic: style.italic,
        underline: style.underline,
        font_family: style.font_family.map(font_family_choice_to_wit),
    }
}

fn font_family_to_wit(family: FontFamily) -> FontFamilyChoice {
    match family {
        FontFamily::Serif => FontFamilyChoice::Serif,
        FontFamily::SansSerif => FontFamilyChoice::SansSerif,
        FontFamily::Monospace => FontFamilyChoice::Monospace,
    }
}

fn text_style_defaults_to_wit(style: &grouped::TextStyle) -> TextStyleDefaults {
    TextStyleDefaults {
        font_size: style.font_size,
        horizontal_padding_pt: style.horizontal_padding_pt,
        vertical_padding_pt: style.vertical_padding_pt,
        bold: style.bold,
        italic: style.italic,
        underline: style.underline,
        font_family: font_family_to_wit(style.font_family),
    }
}

fn font_size_default_to_wit(rule: grouped::FontSizeDefault) -> FontSizeDefault {
    match rule {
        grouped::FontSizeDefault::Fixed { points } => FontSizeDefault::Fixed(points),
        grouped::FontSizeDefault::RowHeightPercent { percent } => {
            FontSizeDefault::RowHeightPercent(percent)
        }
        grouped::FontSizeDefault::SameAs { kind } => {
            FontSizeDefault::SameAs(text_style_kind_to_wit(kind))
        }
    }
}

fn metadata_defaults_to_wit(defaults: &grouped::Metadata) -> MetadataDefaults {
    MetadataDefaults {
        title: defaults.title.clone(),
        subtitle: defaults.subtitle.clone(),
        author: defaults.author.clone(),
        row_height: defaults.row_height,
        max_measures_per_system: defaults.max_measures_per_system,
        note_number_width: defaults.note_number_width,
        parts_list_columns: defaults.parts_list_columns,
        part_label_width_pt: defaults.part_label_width_pt,
        page_margin_pt: defaults.page_margin_pt,
        merge_duplicate_measures_across_parts: defaults.merge_duplicate_measures_across_parts,
        hide_resting_parts: defaults.hide_resting_parts,
        hide_system_dividers: defaults.hide_system_dividers,
        directive_row_offset: defaults.directive_row_offset.to_string(),
    }
}

pub(super) fn metadata_fields_to_wit(fields: source_edit::MetadataFields) -> MetadataFields {
    let source_edit::MetadataFields {
        metadata,
        directive_row_offset,
        inherited,
    } = fields;
    let defaults = grouper::metadata_defaults(&inherited, &metadata);
    MetadataFields {
        styles: parsed::TextStyleKind::ALL
            .iter()
            .map(|&kind| TextStyleEntry {
                kind: text_style_kind_to_wit(kind),
                fields: text_style_fields_to_wit(metadata.style(kind)),
                defaults: text_style_defaults_to_wit(defaults.style(kind)),
                font_size_default: font_size_default_to_wit(kind.font_size_default()),
            })
            .collect(),
        defaults: metadata_defaults_to_wit(&defaults),
        title: metadata.title,
        subtitle: metadata.subtitle,
        author: metadata.author,
        row_height: metadata.row_height,
        max_measures_per_system: metadata.max_measures_per_system,
        note_number_width: metadata.note_number_width,
        parts_list_columns: metadata.parts_list_columns,
        part_label_width_pt: metadata.part_label_width_pt,
        page_margin_pt: metadata.page_margin_pt,
        merge_duplicate_measures_across_parts: metadata.merge_duplicate_measures_across_parts,
        hide_resting_parts: metadata.hide_resting_parts,
        hide_system_dividers: metadata.hide_system_dividers,
        directive_row_offset,
    }
}

pub(super) fn metadata_edit_from_wit(edit: MetadataEdit) -> source_edit::MetadataEdit {
    match edit {
        MetadataEdit::Text(edit) => source_edit::MetadataEdit::Text {
            key: metadata_text_key_from_wit(edit.key),
            value: edit.value,
        },
        MetadataEdit::Number(edit) => source_edit::MetadataEdit::Number {
            key: metadata_number_key_from_wit(edit.key),
            value: edit.value,
        },
        MetadataEdit::Flag(edit) => source_edit::MetadataEdit::Flag {
            key: metadata_flag_key_from_wit(edit.key),
            value: edit.value,
        },
        MetadataEdit::DirectiveRowOffset(value) => {
            source_edit::MetadataEdit::DirectiveRowOffset(value)
        }
        MetadataEdit::Style(edit) => source_edit::MetadataEdit::Style {
            kind: text_style_kind_from_wit(edit.kind),
            value: text_style_component_value_from_wit(edit.value),
        },
        MetadataEdit::ResetStyle(kind) => {
            source_edit::MetadataEdit::ResetStyle(text_style_kind_from_wit(kind))
        }
    }
}
