//! The `score` directive-line vocabulary (see `syntax.md`): `key=value`
//! assignments like `bpm=92`, plus the bare `break`.

keyword_enum! {
    /// The key of a `key=value` directive-line token.
    pub enum DirectiveKey {
        Bpm => "bpm",
        Key => "key",
        Time => "time",
        Label => "label",
        MergeDuplicateMeasuresAcrossParts => "merge_duplicate_measures_across_parts",
        HideRestingParts => "hide_resting_parts",
    }
}

/// The bare directive-line token that forces a system break.
pub const BREAK_KEYWORD: &str = "break";

/// One recognized directive-line token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveToken<'a> {
    Assignment { key: DirectiveKey, value: &'a str },
    Break,
}

impl DirectiveToken<'_> {
    /// Byte length of the token's leading keyword (`bpm` in `bpm=92`).
    pub fn keyword_len(&self) -> usize {
        match self {
            Self::Assignment { key, .. } => key.keyword().len(),
            Self::Break => BREAK_KEYWORD.len(),
        }
    }
}

/// Classifies one whitespace-separated directive-line token, or `None` when
/// it is not a directive.
pub fn classify_directive_token(token: &str) -> Option<DirectiveToken<'_>> {
    if token == BREAK_KEYWORD {
        return Some(DirectiveToken::Break);
    }
    let (key, value) = token.split_once('=')?;
    DirectiveKey::from_keyword(key).map(|key| DirectiveToken::Assignment { key, value })
}

/// Splits a directive line into whitespace-separated tokens, keeping a
/// `"quoted value"` (spaces included) inside its token. Returns
/// `(token_text, byte_offset_within_inner)` pairs.
pub(crate) fn tokenize_directive_tokens(inner: &str) -> Result<Vec<(String, usize)>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut current_start: usize = 0;
    let mut in_quote = false;
    let mut byte_offset: usize = 0;

    for ch in inner.chars() {
        if in_quote {
            current.push(ch);
            if ch == '"' {
                in_quote = false;
            }
            byte_offset += ch.len_utf8();
        } else if ch == '"' {
            if current.is_empty() {
                current_start = byte_offset;
            }
            current.push(ch);
            in_quote = true;
            byte_offset += ch.len_utf8();
        } else if ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push((std::mem::take(&mut current), current_start));
            }
            byte_offset += ch.len_utf8();
        } else {
            if current.is_empty() {
                current_start = byte_offset;
            }
            current.push(ch);
            byte_offset += ch.len_utf8();
        }
    }
    if in_quote {
        return Err("unclosed quote in directive line".to_string());
    }
    if !current.is_empty() {
        tokens.push((current, current_start));
    }
    Ok(tokens)
}
