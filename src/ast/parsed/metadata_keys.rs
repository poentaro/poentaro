//! The `metadata` section's key vocabulary (see `syntax.md`). Each keyword
//! is spelled exactly once, in a `keyword_enum!` table, and everything else
//! (the parser, `source_edit::metadata_edit`'s writer, the wasm boundary)
//! dispatches on these enums.

use super::{FontFamilyChoice, TextStyle};

keyword_enum! {
    /// Every rendered text kind that takes a `<kind> = { ... }` style object.
    /// Declaration order is the canonical order `update_metadata_field`
    /// emits their lines in.
    pub enum TextStyleKind {
        Title => "title",
        Subtitle => "subtitle",
        Author => "author",
        Sequence => "sequence",
        PartLegend => "part_legend",
        MeasureNumber => "measure_number",
        SectionLabel => "section_label",
        PartLabel => "part_label",
        PageNumber => "page_number",
        RunningTitle => "running_title",
        Lyrics => "lyrics",
        Notes => "notes",
        Chords => "chords",
        NoteDash => "note_dash",
    }
}

keyword_enum! {
    /// One component of a `<kind> = { component: value, ... }` style object.
    pub enum TextStyleComponent {
        FontSize => "font_size",
        HorizontalPaddingPt => "horizontal_padding_pt",
        VerticalPaddingPt => "vertical_padding_pt",
        Bold => "bold",
        Italic => "italic",
        Underline => "underline",
        FontFamily => "font_family",
    }
}

keyword_enum! {
    /// Positive-integer `metadata` keys.
    pub enum MetadataNumberKey {
        RowHeight => "row_height",
        MaxMeasuresPerSystem => "max_measures_per_system",
        NoteNumberWidth => "note_number_width",
        PartsListColumns => "parts_list_columns",
        PartLabelWidthPt => "part_label_width_pt",
        PageMarginPt => "page_margin_pt",
    }
}

keyword_enum! {
    /// `yes`/`no` `metadata` keys.
    pub enum MetadataFlagKey {
        MergeDuplicateMeasuresAcrossParts => "merge_duplicate_measures_across_parts",
        HideRestingParts => "hide_resting_parts",
        HideSystemDividers => "hide_system_dividers",
    }
}

keyword_enum! {
    /// The spelling of a `yes`/`no` value.
    pub enum YesNo {
        Yes => "yes",
        No => "no",
    }
}

impl YesNo {
    pub fn from_bool(value: bool) -> Self {
        if value {
            Self::Yes
        } else {
            Self::No
        }
    }

    pub fn to_bool(self) -> bool {
        self == Self::Yes
    }
}

/// The `directive_row_offset = x y` key.
pub const DIRECTIVE_ROW_OFFSET_KEY: &str = "directive_row_offset";

/// The text kinds whose key also takes a plain string (`title = "..."`) for
/// the text itself, besides the `{ ... }` style object every
/// [`TextStyleKind`] takes. Its keyword is its style kind's keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataTextKey {
    Title,
    Subtitle,
    Author,
}

impl MetadataTextKey {
    pub fn style_kind(self) -> TextStyleKind {
        match self {
            Self::Title => TextStyleKind::Title,
            Self::Subtitle => TextStyleKind::Subtitle,
            Self::Author => TextStyleKind::Author,
        }
    }

    pub fn from_style_kind(kind: TextStyleKind) -> Option<Self> {
        match kind {
            TextStyleKind::Title => Some(Self::Title),
            TextStyleKind::Subtitle => Some(Self::Subtitle),
            TextStyleKind::Author => Some(Self::Author),
            TextStyleKind::Sequence
            | TextStyleKind::PartLegend
            | TextStyleKind::MeasureNumber
            | TextStyleKind::SectionLabel
            | TextStyleKind::PartLabel
            | TextStyleKind::PageNumber
            | TextStyleKind::RunningTitle
            | TextStyleKind::Lyrics
            | TextStyleKind::Notes
            | TextStyleKind::Chords
            | TextStyleKind::NoteDash => None,
        }
    }

    pub fn keyword(self) -> &'static str {
        self.style_kind().keyword()
    }
}

/// One style component together with its (possibly unset) value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextStyleComponentValue {
    FontSize(Option<u32>),
    HorizontalPaddingPt(Option<u32>),
    VerticalPaddingPt(Option<u32>),
    Bold(Option<bool>),
    Italic(Option<bool>),
    Underline(Option<bool>),
    FontFamily(Option<FontFamilyChoice>),
}

impl TextStyleComponentValue {
    pub fn component(self) -> TextStyleComponent {
        match self {
            Self::FontSize(_) => TextStyleComponent::FontSize,
            Self::HorizontalPaddingPt(_) => TextStyleComponent::HorizontalPaddingPt,
            Self::VerticalPaddingPt(_) => TextStyleComponent::VerticalPaddingPt,
            Self::Bold(_) => TextStyleComponent::Bold,
            Self::Italic(_) => TextStyleComponent::Italic,
            Self::Underline(_) => TextStyleComponent::Underline,
            Self::FontFamily(_) => TextStyleComponent::FontFamily,
        }
    }

    /// The value as `.poentaro` source spells it, or `None` when unset.
    pub fn source_text(self) -> Option<String> {
        let yes_no = |flag: bool| YesNo::from_bool(flag).keyword().to_owned();
        match self {
            Self::FontSize(value)
            | Self::HorizontalPaddingPt(value)
            | Self::VerticalPaddingPt(value) => value.map(|number| number.to_string()),
            Self::Bold(value) | Self::Italic(value) | Self::Underline(value) => value.map(yes_no),
            Self::FontFamily(value) => value.map(|family| family.keyword().to_owned()),
        }
    }
}

impl TextStyle {
    pub fn component_value(&self, component: TextStyleComponent) -> TextStyleComponentValue {
        match component {
            TextStyleComponent::FontSize => TextStyleComponentValue::FontSize(self.font_size),
            TextStyleComponent::HorizontalPaddingPt => {
                TextStyleComponentValue::HorizontalPaddingPt(self.horizontal_padding_pt)
            }
            TextStyleComponent::VerticalPaddingPt => {
                TextStyleComponentValue::VerticalPaddingPt(self.vertical_padding_pt)
            }
            TextStyleComponent::Bold => TextStyleComponentValue::Bold(self.bold),
            TextStyleComponent::Italic => TextStyleComponentValue::Italic(self.italic),
            TextStyleComponent::Underline => TextStyleComponentValue::Underline(self.underline),
            TextStyleComponent::FontFamily => TextStyleComponentValue::FontFamily(self.font_family),
        }
    }

    pub fn set_component(&mut self, value: TextStyleComponentValue) {
        match value {
            TextStyleComponentValue::FontSize(v) => self.font_size = v,
            TextStyleComponentValue::HorizontalPaddingPt(v) => self.horizontal_padding_pt = v,
            TextStyleComponentValue::VerticalPaddingPt(v) => self.vertical_padding_pt = v,
            TextStyleComponentValue::Bold(v) => self.bold = v,
            TextStyleComponentValue::Italic(v) => self.italic = v,
            TextStyleComponentValue::Underline(v) => self.underline = v,
            TextStyleComponentValue::FontFamily(v) => self.font_family = v,
        }
    }
}
