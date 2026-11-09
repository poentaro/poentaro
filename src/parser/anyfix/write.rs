//! Writes fields and part kinds in anyfix, for the editor's forms.

use itertools::Itertools;

use super::words::{abbreviation, is_name_word, quoted, snake_case_as_words};
use crate::ast::parsed::{RangeInstrument, DIRECTIVE_ROW_OFFSET_KEY};

/// The preset fields that hold a list of part abbreviations.
const PART_LIST_KEYS: [&str; 2] = ["visible_parts", "soloed_parts"];

/// A snake_case key as words, or quoted when it can't be (reading it then
/// reports it).
fn key_words(key: &str) -> String {
    snake_case_as_words(key).unwrap_or_else(|| quoted(key))
}

/// A field, from its snake_case key and its value: `a, b` for a part list,
/// `{ font_size: 12 }` for a text style, quoted text, or a bare scalar.
pub(crate) fn key_value_field(key: &str, value: &str) -> String {
    if PART_LIST_KEYS.contains(&key) {
        let parts = value
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(abbreviation)
            .join(", ");
        return format!("{} : [{parts}]", key_words(key));
    }
    if let Some(inner) = value
        .strip_prefix('{')
        .and_then(|rest| rest.strip_suffix('}'))
    {
        let fields = inner
            .split(',')
            .filter(|field| !field.trim().is_empty())
            .map(|field| match field.split_once(':') {
                Some((name, value)) => {
                    format!("{} : {}", key_words(name.trim()), scalar(value.trim()))
                }
                None => quoted(field.trim()),
            })
            .join(", ");
        return format!("{} style : {{ {fields} }}", key_words(key));
    }
    if key == DIRECTIVE_ROW_OFFSET_KEY {
        if let Some((x, y)) = value.trim_matches('"').split_whitespace().collect_tuple() {
            return format!("{} : ({x}, {y})", key_words(key));
        }
    }
    format!("{} : {}", key_words(key), scalar(value))
}

/// A value: quoted text stays quoted, numbers and signed numbers stay as
/// written, snake_case words become runs (`sans_serif` → `sans serif`).
fn scalar(value: &str) -> String {
    if let Some(text) = value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        return quoted(text);
    }
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    let is_number = unsigned.starts_with(|c: char| c.is_ascii_digit())
        && unsigned
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.');
    if is_number {
        return value.to_string();
    }
    match snake_case_as_words(value) {
        Some(words) if is_name_word(value) => words,
        _ => quoted(value),
    }
}

/// A part's kind and options, as they are written.
pub(crate) struct PartKindText {
    pub head: String,
    pub range_instrument: Option<RangeInstrument>,
    pub sound: Option<u8>,
    pub volume: Option<u8>,
    pub octave_offset: Option<i8>,
}

pub(crate) fn part_kind_text(kind: &PartKindText) -> String {
    let options = [
        kind.range_instrument
            .map(|instrument| format!("instrument : {}", instrument.keyword())),
        kind.sound.map(|sound| format!("sound : {sound}")),
        kind.volume.map(|volume| format!("volume : {volume}%")),
        kind.octave_offset
            .map(|octave| format!("octave : {octave:+}")),
    ]
    .into_iter()
    .flatten()
    .join(", ");
    if options.is_empty() {
        kind.head.clone()
    } else {
        format!("{} {{ {options} }}", kind.head)
    }
}
