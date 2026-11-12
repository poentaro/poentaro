//! How anyx splits text into words, as far as the converter and the
//! reader need to know, so a value written one way reads back the same.

/// Whether `text` is one anyx name word: a letter or `_`, then letters,
/// digits or `_`.
pub(super) fn is_name_word(text: &str) -> bool {
    let mut chars = text.chars();
    chars
        .next()
        .is_some_and(|first| first.is_alphabetic() || first == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_')
        && text != "_"
}

/// Whether `text` is an anyx keyword: a name word with an uppercase letter
/// and no lowercase one. Keywords split a run, so a keyword can't stand where
/// a plain word is expected.
pub(super) fn is_keyword(text: &str) -> bool {
    is_name_word(text)
        && text.chars().any(char::is_uppercase)
        && !text.chars().any(char::is_lowercase)
}

/// A snake_case name (`max_measures_per_system`) as an anyx run of words
/// (`max measures per system`), when every part is a non-keyword name word.
pub(super) fn snake_case_as_words(text: &str) -> Option<String> {
    let parts: Vec<&str> = text.split('_').collect();
    parts
        .iter()
        .all(|part| is_name_word(part) && !is_keyword(part))
        .then(|| parts.join(" "))
}

/// `text` as an anyx quoted string.
pub(crate) fn quoted(text: &str) -> String {
    let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// The characters that can't appear in an abbreviation written bare between
/// brackets or commas: anyx's special characters, plus whitespace.
fn breaks_bare_abbreviation(c: char) -> bool {
    c.is_whitespace() || ",:.()[]{}#\"".contains(c)
}

/// An abbreviation as written inside `[ ]` or a list: bare when it reads back
/// the same from the tree (no whitespace, no special
/// character), quoted otherwise.
pub(crate) fn abbreviation(text: &str) -> String {
    // A bare `_` is a placeholder, not a word.
    if text.is_empty() || text == "_" || text.contains(breaks_bare_abbreviation) {
        quoted(text)
    } else {
        text.to_string()
    }
}

/// A section label as a sequence entry: a bare word when it is a plain
/// non-keyword name word, quoted otherwise.
pub(crate) fn label(text: &str) -> String {
    if is_name_word(text) && !is_keyword(text) {
        text.to_string()
    } else {
        quoted(text)
    }
}
