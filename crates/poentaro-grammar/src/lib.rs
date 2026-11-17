//! The shapes a `.poentaro` document is made of: `poentaro.anyx` is the
//! grammar, and `generated.rs` is the decoder anyx-generator writes from it.
//! `just generate-parser` writes it again after the grammar changes.
//!
//! What the shapes mean, such as which metadata keys exist, is read in the
//! main crate (`src/parser/anyx/`).

#[rustfmt::skip]
mod generated;

pub use generated::*;

use anyx::{Span, StringLiteral};

/// A `key value` field of a record: metadata, preset or text style. Its value
/// is read from its choice, whose type the grammar gives.
pub trait Keyed {
    /// The key's words, as written: `max measures per system`.
    fn key_words(&self) -> String;
    fn key_words_span(&self) -> Span;
    /// The whole field.
    fn span(&self) -> Span;
}

/// The words a key's choice is named after: `MaxMeasuresPerSystem` is
/// `max measures per system`.
fn words_of(choice: &str) -> String {
    let mut words = String::new();
    for character in choice.chars() {
        if character.is_uppercase() && !words.is_empty() {
            words.push(' ');
        }
        words.extend(character.to_lowercase());
    }
    words
}

/// [`Keyed`] for a `ONE OF` whose `listed` choices are each
/// `ANYFIX CALL [key: WORDS "…", …]`, named after their words, and whose
/// `nested` choices are themselves keyed. Rust's match is exhaustive, so a
/// choice added to the grammar must be added here.
macro_rules! keyed {
    ($type:ident { $($listed:ident),* $(,)? } $(nested { $($nested:ident),+ $(,)? })?) => {
        impl Keyed for $type {
            fn key_words(&self) -> String {
                match self {
                    $(Self::$listed { .. } => words_of(stringify!($listed)),)*
                    $($(Self::$nested(field) => field.key_words(),)+)?
                }
            }

            fn key_words_span(&self) -> Span {
                match self {
                    $(Self::$listed { key, .. } => *key,)*
                    $($(Self::$nested(field) => field.key_words_span(),)+)?
                }
            }

            fn span(&self) -> Span {
                match self {
                    $(Self::$listed { span, .. } => *span,)*
                    $($(Self::$nested(field) => field.span(),)+)?
                }
            }
        }
    };
}

keyed!(MetadataField { DirectiveRowOffset, CouldNotConvert } nested { Text, Style, Number, Flag });
keyed!(TextField {
    Title,
    Subtitle,
    Author
});
keyed!(TextStyle {
    TitleStyle,
    SubtitleStyle,
    AuthorStyle,
    SequenceStyle,
    PartLegendStyle,
    MeasureNumberStyle,
    SectionLabelStyle,
    PartLabelStyle,
    PageNumberStyle,
    RunningTitleStyle,
    LyricsStyle,
    NotesStyle,
    ChordsStyle,
    NoteDashStyle,
});
keyed!(NumberField {
    RowHeight,
    MaxMeasuresPerSystem,
    NoteNumberWidth,
    PartsListColumns,
    PartLabelWidthPt,
    PageMarginPt,
});
keyed!(FlagField {
    MergeDuplicateMeasuresAcrossParts,
    HideRestingParts,
    HideSystemDividers
});
keyed!(PresetField { VisibleParts, SoloedParts, Notation } nested { Metadata });
keyed!(StyleField {
    FontSize,
    HorizontalPaddingPt,
    VerticalPaddingPt,
    Bold,
    Italic,
    Underline,
    FontFamily,
});

/// The value every choice of a `ONE OF` keeps under the same name.
macro_rules! value {
    ($type:ident { $($choice:ident),+ $(,)? } . $field:ident: $value:ty) => {
        impl $type {
            pub fn $field(&self) -> &$value {
                match self {
                    $(Self::$choice { $field, .. } => $field,)+
                }
            }
        }
    };
}

value!(TextField { Title, Subtitle, Author } . text: Spanned<StringLiteral>);
value!(TextStyle {
    TitleStyle, SubtitleStyle, AuthorStyle, SequenceStyle, PartLegendStyle,
    MeasureNumberStyle, SectionLabelStyle, PartLabelStyle, PageNumberStyle,
    RunningTitleStyle, LyricsStyle, NotesStyle, ChordsStyle, NoteDashStyle,
} . style: StyleRecord);
value!(NumberField {
    RowHeight, MaxMeasuresPerSystem, NoteNumberWidth, PartsListColumns,
    PartLabelWidthPt, PageMarginPt,
} . number: Positive);
value!(FlagField { MergeDuplicateMeasuresAcrossParts, HideRestingParts, HideSystemDividers } . flag: Flag);

impl Flag {
    pub fn is_yes(&self) -> bool {
        matches!(self, Flag::Yes(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(source: &str) -> Vec<(String, String)> {
        let document = parse_document(source).document.unwrap();
        let fields = &document.metadata.as_ref().unwrap().fields.metadata_field;
        fields
            .iter()
            .map(|field| match field {
                Ok(field) => {
                    let key = field.key_words_span();
                    (field.key_words(), source[key.start..key.end].to_owned())
                }
                Err(error) => ("error".to_owned(), error.message()),
            })
            .collect()
    }

    #[test]
    fn a_key_is_its_choice_in_words() {
        assert_eq!(
            keys(
                "metadata { max measures per system 4, lyrics style {}, hide resting parts : yes }"
            ),
            [
                (
                    "max measures per system".to_owned(),
                    "max measures per system".to_owned()
                ),
                ("lyrics style".to_owned(), "lyrics style".to_owned()),
                (
                    "hide resting parts".to_owned(),
                    "hide resting parts".to_owned()
                ),
            ]
        );
    }

    #[test]
    fn a_wrong_value_is_the_grammars_error() {
        let [(kind, message)] = &keys("metadata { max measures per system 0 }")[..] else {
            panic!()
        };
        assert_eq!(kind, "error");
        assert!(message.contains("0"), "{message}");
    }
}
