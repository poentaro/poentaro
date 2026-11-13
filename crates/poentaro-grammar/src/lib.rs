//! The shapes a `.poentaro` document is made of: `poentaro.anyx` is the
//! grammar, and `generated.rs` is the decoder anyx-generator writes from it.
//! `just generate-parser` writes it again after the grammar changes.
//!
//! What the shapes mean, such as which metadata keys exist, is read in the
//! main crate (`src/parser/anyx/`).

#[rustfmt::skip]
mod generated;

pub use generated::*;

use anyx::Span;

/// A `key value` field, whichever choice decoded it: a key the grammar lists,
/// such as `max measures per system 6`, or the fallback `field` of any other
/// key, which Rust reports.
pub trait Keyed {
    /// The key's words, as written: `max measures per system`.
    fn key_words(&self) -> String;
    fn key_words_span(&self) -> Span;
    fn value(&self) -> &Value;
    /// The whole field.
    fn span(&self) -> Span;
}

impl Keyed for Field {
    fn key_words(&self) -> String {
        self.name.words().collect::<Vec<_>>().join(" ")
    }

    fn key_words_span(&self) -> Span {
        self.name.span
    }

    fn value(&self) -> &Value {
        &self.arguments
    }

    fn span(&self) -> Span {
        self.span
    }
}

/// The words a listed key's choice is named after: `MaxMeasuresPerSystem` is
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

/// [`Keyed`] for a `ONE OF` of listed keys, each `ANYFIX CALL [key: WORDS
/// "…", value]`, and a last choice that is keyed itself. Rust's match is
/// exhaustive, so a key added to the grammar must be added here.
macro_rules! keyed {
    ($type:ident { $($listed:ident),+ $(,)? } else $fallback:ident) => {
        impl Keyed for $type {
            fn key_words(&self) -> String {
                match self {
                    $(Self::$listed { .. } => words_of(stringify!($listed)),)+
                    Self::$fallback(field) => field.key_words(),
                }
            }

            fn key_words_span(&self) -> Span {
                match self {
                    $(Self::$listed { key, .. } => *key,)+
                    Self::$fallback(field) => field.key_words_span(),
                }
            }

            fn value(&self) -> &Value {
                match self {
                    $(Self::$listed { value, .. } => value,)+
                    Self::$fallback(field) => field.value(),
                }
            }

            fn span(&self) -> Span {
                match self {
                    $(Self::$listed { span, .. } => *span,)+
                    Self::$fallback(field) => field.span(),
                }
            }
        }
    };
}

keyed!(MetadataField {
    Title, Subtitle, Author,
    TitleStyle, SubtitleStyle, AuthorStyle, SequenceStyle, PartLegendStyle,
    MeasureNumberStyle, SectionLabelStyle, PartLabelStyle, PageNumberStyle,
    RunningTitleStyle, LyricsStyle, NotesStyle, ChordsStyle, NoteDashStyle,
    RowHeight, MaxMeasuresPerSystem, NoteNumberWidth, PartsListColumns,
    PartLabelWidthPt, PageMarginPt,
    MergeDuplicateMeasuresAcrossParts, HideRestingParts, HideSystemDividers,
    DirectiveRowOffset, CouldNotConvert,
} else Unknown);

keyed!(PresetField { VisibleParts, SoloedParts, Notation } else Metadata);

keyed!(StyleField {
    FontSize, HorizontalPaddingPt, VerticalPaddingPt, Bold, Italic, Underline, FontFamily,
} else Unknown);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_listed_key_is_its_choice_in_words() {
        let source = "metadata { max measures per system 4, colour 3 }";
        let parsed = parse_document(source);
        let document = parsed.document.unwrap();
        let fields = &document.metadata.as_ref().unwrap().fields.metadata_field;
        let keys: Vec<_> = fields
            .iter()
            .map(|field| {
                let field = field.as_ref().unwrap();
                let key = field.key_words_span();
                (field.key_words(), &source[key.start..key.end])
            })
            .collect();
        assert_eq!(
            keys,
            [
                (
                    "max measures per system".to_owned(),
                    "max measures per system"
                ),
                ("colour".to_owned(), "colour"),
            ]
        );
        assert!(matches!(fields[1], Ok(MetadataField::Unknown(_))));
    }
}
