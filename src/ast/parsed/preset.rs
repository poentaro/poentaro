//! A `preset "Name"` section: a named set of visible and soloed parts, a notation,
//! and `metadata` overrides (see `syntax.md`, "Presets").

use super::{AbbreviationReference, ParsedDocument, ParsedMetadata, TextStyle, TextStyleKind};
use crate::error::Span;

/// The preset menu's name for "no preset": the song's own `metadata`, every
/// part, numbered notation. A `preset` can't take this name.
pub const DEFAULT_PRESET_NAME: &str = "Default";

keyword_enum! {
    /// The keys only a `preset` section takes, besides every `metadata` key.
    pub enum PresetKey {
        VisibleParts => "visible_parts",
        SoloedParts => "soloed_parts",
        Notation => "notation",
    }
}

keyword_enum! {
    /// How the preview and exports draw the score.
    pub enum Notation {
        Numbered => "numbered",
        Violin => "violin",
        Cello => "cello",
        Abc => "abc",
        Guitar => "guitar",
        Harmonica => "harmonica",
    }
}

/// What `notation = ...` called [`Notation::Numbered`] before the rename.
/// Still read so older files keep their preset's notation; never written.
const LEGACY_NUMBERED_KEYWORD: &str = "jianpu";

impl Notation {
    /// The notation a `notation = ...` line names: [`Self::from_keyword`],
    /// plus the legacy spelling of [`Self::Numbered`].
    pub fn from_source_value(value: &str) -> Option<Self> {
        Self::from_keyword(value)
            .or_else(|| (value == LEGACY_NUMBERED_KEYWORD).then_some(Self::Numbered))
    }

    /// Whether this notation is drawn as staff notation by Verovio, which
    /// ignores the `metadata` text styles.
    pub fn ignores_text_styles(self) -> bool {
        matches!(self, Self::Violin | Self::Cello)
    }
}

#[derive(Debug, Clone)]
pub struct ParsedPreset {
    pub name: String,
    /// The quoted name in the header, quotes included.
    pub name_span: Span,
    /// The whole `preset "Name"` header line.
    pub header_span: Span,
    /// The `metadata` keys this preset sets; unset fields keep the song's own value.
    pub overrides: ParsedMetadata,
    /// The parts shown and heard, in the order written; `None` shows every
    /// part. Unknown abbreviations are already dropped.
    pub visible_parts: Option<Vec<AbbreviationReference>>,
    /// The parts soloed, in the order written: once any is, only soloed
    /// parts show and sound. Unknown abbreviations are already dropped.
    pub soloed_parts: Vec<AbbreviationReference>,
    pub notation: Option<Notation>,
}

impl TextStyle {
    /// `self` with every component `overrides` sets replaced.
    pub fn overlaid_with(&self, overrides: &TextStyle) -> TextStyle {
        TextStyle {
            font_size: overrides.font_size.or(self.font_size),
            horizontal_padding_pt: overrides
                .horizontal_padding_pt
                .or(self.horizontal_padding_pt),
            vertical_padding_pt: overrides.vertical_padding_pt.or(self.vertical_padding_pt),
            bold: overrides.bold.or(self.bold),
            italic: overrides.italic.or(self.italic),
            underline: overrides.underline.or(self.underline),
            font_family: overrides.font_family.or(self.font_family),
        }
    }
}

impl ParsedMetadata {
    /// `self` with every field `overrides` sets replaced, down to single
    /// text-style components.
    pub fn overlaid_with(&self, overrides: &ParsedMetadata) -> ParsedMetadata {
        let text = |base: &Option<String>, over: &Option<String>| over.clone().or(base.clone());
        let style = |kind: TextStyleKind| self.style(kind).overlaid_with(overrides.style(kind));
        ParsedMetadata {
            title: text(&self.title, &overrides.title),
            subtitle: text(&self.subtitle, &overrides.subtitle),
            author: text(&self.author, &overrides.author),
            row_height: overrides.row_height.or(self.row_height),
            max_measures_per_system: overrides
                .max_measures_per_system
                .or(self.max_measures_per_system),
            note_number_width: overrides.note_number_width.or(self.note_number_width),
            parts_list_columns: overrides.parts_list_columns.or(self.parts_list_columns),
            part_label_width_pt: overrides.part_label_width_pt.or(self.part_label_width_pt),
            page_margin_pt: overrides.page_margin_pt.or(self.page_margin_pt),
            title_style: style(TextStyleKind::Title),
            subtitle_style: style(TextStyleKind::Subtitle),
            author_style: style(TextStyleKind::Author),
            sequence_style: style(TextStyleKind::Sequence),
            part_legend_style: style(TextStyleKind::PartLegend),
            measure_number_style: style(TextStyleKind::MeasureNumber),
            section_label_style: style(TextStyleKind::SectionLabel),
            page_number_style: style(TextStyleKind::PageNumber),
            running_title_style: style(TextStyleKind::RunningTitle),
            part_label_style: style(TextStyleKind::PartLabel),
            lyrics_style: style(TextStyleKind::Lyrics),
            notes_style: style(TextStyleKind::Notes),
            chords_style: style(TextStyleKind::Chords),
            note_dash_style: style(TextStyleKind::NoteDash),
            merge_duplicate_measures_across_parts: overrides
                .merge_duplicate_measures_across_parts
                .or(self.merge_duplicate_measures_across_parts),
            hide_resting_parts: overrides.hide_resting_parts.or(self.hide_resting_parts),
            hide_system_dividers: overrides.hide_system_dividers.or(self.hide_system_dividers),
            directive_row_offset: overrides.directive_row_offset.or(self.directive_row_offset),
        }
    }

    /// Whether any text kind has a style component set.
    pub fn sets_any_text_style(&self) -> bool {
        TextStyleKind::ALL
            .iter()
            .any(|kind| *self.style(*kind) != TextStyle::default())
    }
}

impl ParsedPreset {
    /// The abbreviations of `visible_parts`, in the order written.
    pub fn visible_abbreviations(&self) -> Option<Vec<String>> {
        self.visible_parts.as_ref().map(|references| {
            references
                .iter()
                .map(|reference| reference.abbreviation.clone())
                .collect()
        })
    }

    /// The abbreviations of `soloed_parts`, in the order written.
    pub fn soloed_abbreviations(&self) -> Vec<String> {
        self.soloed_parts
            .iter()
            .map(|reference| reference.abbreviation.clone())
            .collect()
    }
}

impl ParsedDocument {
    /// The `preset` named `name`, if the document has one.
    pub fn preset(&self, name: &str) -> Option<&ParsedPreset> {
        self.presets.iter().find(|preset| preset.name == name)
    }

    /// This document as preset `name` draws it: `metadata` overlaid with the
    /// preset's overrides. An unknown name leaves the document unchanged, so
    /// a renamed or deleted preset falls back to Default.
    pub fn with_preset(self, name: &str) -> ParsedDocument {
        let metadata = match self.preset(name) {
            Some(preset) => self.metadata.overlaid_with(&preset.overrides),
            None => return self,
        };
        ParsedDocument { metadata, ..self }
    }
}

#[cfg(test)]
#[path = "preset_tests.rs"]
mod preset_tests;
