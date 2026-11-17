//! Key-addressed access to [`ParsedMetadata`]'s fields, so callers that
//! hold a [`TextStyleKind`]/[`MetadataNumberKey`]/[`MetadataFlagKey`]/
//! [`MetadataTextKey`] reach the matching field through one exhaustive match
//! instead of re-matching keyword strings.

use super::{
    MetadataFlagKey, MetadataNumberKey, MetadataTextKey, ParsedMetadata, TextStyle, TextStyleKind,
};

impl ParsedMetadata {
    pub fn style(&self, kind: TextStyleKind) -> &TextStyle {
        match kind {
            TextStyleKind::Title => &self.title_style,
            TextStyleKind::Subtitle => &self.subtitle_style,
            TextStyleKind::Author => &self.author_style,
            TextStyleKind::Sequence => &self.sequence_style,
            TextStyleKind::PartLegend => &self.part_legend_style,
            TextStyleKind::MeasureNumber => &self.measure_number_style,
            TextStyleKind::SectionLabel => &self.section_label_style,
            TextStyleKind::PartLabel => &self.part_label_style,
            TextStyleKind::PageNumber => &self.page_number_style,
            TextStyleKind::RunningTitle => &self.running_title_style,
            TextStyleKind::Lyrics => &self.lyrics_style,
            TextStyleKind::Notes => &self.notes_style,
            TextStyleKind::Chords => &self.chords_style,
            TextStyleKind::NoteDash => &self.note_dash_style,
        }
    }

    pub fn style_mut(&mut self, kind: TextStyleKind) -> &mut TextStyle {
        match kind {
            TextStyleKind::Title => &mut self.title_style,
            TextStyleKind::Subtitle => &mut self.subtitle_style,
            TextStyleKind::Author => &mut self.author_style,
            TextStyleKind::Sequence => &mut self.sequence_style,
            TextStyleKind::PartLegend => &mut self.part_legend_style,
            TextStyleKind::MeasureNumber => &mut self.measure_number_style,
            TextStyleKind::SectionLabel => &mut self.section_label_style,
            TextStyleKind::PartLabel => &mut self.part_label_style,
            TextStyleKind::PageNumber => &mut self.page_number_style,
            TextStyleKind::RunningTitle => &mut self.running_title_style,
            TextStyleKind::Lyrics => &mut self.lyrics_style,
            TextStyleKind::Notes => &mut self.notes_style,
            TextStyleKind::Chords => &mut self.chords_style,
            TextStyleKind::NoteDash => &mut self.note_dash_style,
        }
    }

    pub fn number_mut(&mut self, key: MetadataNumberKey) -> &mut Option<u32> {
        match key {
            MetadataNumberKey::RowHeight => &mut self.row_height,
            MetadataNumberKey::MaxMeasuresPerSystem => &mut self.max_measures_per_system,
            MetadataNumberKey::NoteNumberWidth => &mut self.note_number_width,
            MetadataNumberKey::PartsListColumns => &mut self.parts_list_columns,
            MetadataNumberKey::PartLabelWidthPt => &mut self.part_label_width_pt,
            MetadataNumberKey::PageMarginPt => &mut self.page_margin_pt,
        }
    }

    pub fn number(&self, key: MetadataNumberKey) -> Option<u32> {
        match key {
            MetadataNumberKey::RowHeight => self.row_height,
            MetadataNumberKey::MaxMeasuresPerSystem => self.max_measures_per_system,
            MetadataNumberKey::NoteNumberWidth => self.note_number_width,
            MetadataNumberKey::PartsListColumns => self.parts_list_columns,
            MetadataNumberKey::PartLabelWidthPt => self.part_label_width_pt,
            MetadataNumberKey::PageMarginPt => self.page_margin_pt,
        }
    }

    pub fn flag_mut(&mut self, key: MetadataFlagKey) -> &mut Option<bool> {
        match key {
            MetadataFlagKey::MergeDuplicateMeasuresAcrossParts => {
                &mut self.merge_duplicate_measures_across_parts
            }
            MetadataFlagKey::HideRestingParts => &mut self.hide_resting_parts,
            MetadataFlagKey::HideSystemDividers => &mut self.hide_system_dividers,
        }
    }

    pub fn flag(&self, key: MetadataFlagKey) -> Option<bool> {
        match key {
            MetadataFlagKey::MergeDuplicateMeasuresAcrossParts => {
                self.merge_duplicate_measures_across_parts
            }
            MetadataFlagKey::HideRestingParts => self.hide_resting_parts,
            MetadataFlagKey::HideSystemDividers => self.hide_system_dividers,
        }
    }

    pub fn text_mut(&mut self, key: MetadataTextKey) -> &mut Option<String> {
        match key {
            MetadataTextKey::Title => &mut self.title,
            MetadataTextKey::Subtitle => &mut self.subtitle,
            MetadataTextKey::Author => &mut self.author,
        }
    }

    pub fn text(&self, key: MetadataTextKey) -> Option<&str> {
        match key {
            MetadataTextKey::Title => self.title.as_deref(),
            MetadataTextKey::Subtitle => self.subtitle.as_deref(),
            MetadataTextKey::Author => self.author.as_deref(),
        }
    }
}
