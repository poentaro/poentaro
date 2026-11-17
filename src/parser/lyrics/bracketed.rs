//! Tokenising a lyric line under a chord part, where `( … )` / `（ … ）`
//! mark the bracketed items paired with chord attacks. See "Lyrics under a
//! chord part" in `syntax.md`.
use crate::ast::parsed::Syllable;
use crate::error::{RecoverableError, Span};
use crate::utils::{is_han_char, pinyin_override_at, tokenize_lyrics, tokenize_lyrics_with_errors};

/// The syllables of one chord-lyric line plus any bracket mistakes found.
/// Each bracket yields one syllable with `bracketed: true` (when valid).
#[derive(Debug, Clone)]
pub struct BracketedLyrics {
    pub syllables: Vec<Syllable>,
    pub errors: Vec<RecoverableError>,
}

fn is_open(c: char) -> bool {
    c == '(' || c == '（'
}

fn is_close(c: char) -> bool {
    c == ')' || c == '）'
}

/// Like [`tokenize_lyrics`], but additionally recognises bracketed items.
/// Text outside brackets is tokenised exactly as in an ordinary lyric line.
/// `base_offset` is the absolute offset of `content` in the source.
pub fn tokenize_bracketed_lyrics(content: &str, base_offset: usize) -> BracketedLyrics {
    let mut result = BracketedLyrics {
        syllables: Vec::new(),
        errors: Vec::new(),
    };
    let mut plain_start = 0;
    let mut chars = content.char_indices().peekable();
    while let Some((idx, c)) = chars.next() {
        if !is_open(c) && !is_close(c) {
            continue;
        }
        // `行(háng)` is the character's reading, not a bracketed item: the
        // plain text around it is tokenised with it.
        if let Some(end) = attached_pinyin_end(content, idx, base_offset) {
            while chars.peek().is_some_and(|(i, _)| *i < end) {
                chars.next();
            }
            continue;
        }
        push_plain(
            &mut result,
            &content[plain_start..idx],
            base_offset + plain_start,
        );
        let after = idx + c.len_utf8();
        plain_start = after;
        if is_close(c) {
            result
                .errors
                .push(RecoverableError::lyrics_bracket_unmatched(Span::new(
                    base_offset + idx,
                    base_offset + after,
                )));
            continue;
        }
        match content[after..].find(is_close) {
            None => result
                .errors
                .push(RecoverableError::lyrics_bracket_unclosed(Span::new(
                    base_offset + idx,
                    base_offset + after,
                ))),
            Some(rel) => {
                let close = after + rel;
                let inner = tokenize_lyrics(&content[after..close], base_offset + after);
                let close_end = close + content[close..].chars().next().map_or(1, char::len_utf8);
                let whole = Span::new(base_offset + idx, base_offset + close_end);
                match <[Syllable; 1]>::try_from(inner) {
                    Ok([only]) if only.text != "-" => result.syllables.push(Syllable {
                        bracketed: true,
                        held: false,
                        ..only
                    }),
                    Ok(_) => result
                        .errors
                        .push(RecoverableError::lyrics_bracket_not_one_syllable(whole, 0)),
                    Err(many) => {
                        result
                            .errors
                            .push(RecoverableError::lyrics_bracket_not_one_syllable(
                                whole,
                                many.len(),
                            ))
                    }
                }
                plain_start = close_end;
                while chars.peek().is_some_and(|(i, _)| *i < close_end) {
                    chars.next();
                }
            }
        }
    }
    push_plain(
        &mut result,
        &content[plain_start..],
        base_offset + plain_start,
    );
    result
}

fn push_plain(result: &mut BracketedLyrics, text: &str, base_offset: usize) {
    let plain = tokenize_lyrics_with_errors(text, base_offset);
    result.syllables.extend(plain.syllables);
    result.errors.extend(plain.errors);
}

/// Where a valid `(pinyin)` attached to the Chinese character before `open`
/// ends; a bracket holding anything else stays a bracketed item (`我(陪)`).
fn attached_pinyin_end(content: &str, open: usize, base_offset: usize) -> Option<usize> {
    content
        .get(..open)?
        .chars()
        .next_back()
        .filter(|c| is_han_char(*c))?;
    pinyin_override_at(content, open, base_offset)
        .filter(|attached| attached.pinyin.is_ok())
        .map(|attached| attached.end)
}

#[cfg(test)]
#[path = "bracketed_tests.rs"]
mod tests;
