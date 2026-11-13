use crate::ast::parsed::{ScoreEvent, Syllable};
use crate::error::{RecoverableError, Span, Spanned};
use crate::pinyin::PinyinSyllable;

/// Tracks tie state across measure boundaries for lyric-slot counting.
#[derive(Debug, Clone, Default)]
pub struct LyricTieState {
    pub prev_tie_to_next: bool,
}

/// Count note heads in `events` that consume a lyric syllable (non-tie-continuation notes).
/// Updates `state` so the next measure can continue cross-bar ties.
pub fn count_lyric_slots_in_events(
    events: &[Spanned<ScoreEvent>],
    state: &mut LyricTieState,
) -> u32 {
    let mut count = 0u32;
    for spanned in events {
        match &spanned.value {
            ScoreEvent::Note(note) => {
                if !state.prev_tie_to_next {
                    count += 1;
                }
                state.prev_tie_to_next = note.tie_to_next();
            }
            ScoreEvent::Rest(_) => {
                state.prev_tie_to_next = false;
            }
            _ => {}
        }
    }
    count
}

/// Returns true if `c` is a CJK or Japanese/Korean character.
/// Covers Hiragana, Katakana, CJK Extension A, CJK Unified Ideographs, Hangul.
pub fn is_cjk_char(c: char) -> bool {
    matches!(c as u32,
        0x3040..=0x309F |  // Hiragana
        0x30A0..=0x30FF |  // Katakana
        0x3400..=0x4DBF |  // CJK Extension A
        0x4E00..=0x9FFF |  // CJK Unified Ideographs
        0xAC00..=0xD7AF    // Hangul
    )
}

/// A lyric line's syllables, and the mistakes in its `字(pinyin)` overrides.
#[derive(Debug, Clone, Default)]
pub struct LyricTokens {
    pub syllables: Vec<Syllable>,
    pub errors: Vec<RecoverableError>,
}

/// Tokenizes a lyrics line's text into syllables. `base_offset` is the
/// absolute byte offset in the whole source document where `content` begins,
/// so each syllable's `span` can be recorded absolutely — see `Syllable::span`.
pub fn tokenize_lyrics(content: &str, base_offset: usize) -> Vec<Syllable> {
    tokenize_lyrics_with_errors(content, base_offset).syllables
}

/// [`tokenize_lyrics`], keeping the errors of any `字(pinyin)` override it
/// had to ignore.
pub fn tokenize_lyrics_with_errors(content: &str, base_offset: usize) -> LyricTokens {
    let raw = tokenize_lyrics_raw(content, base_offset);

    // Post-process: each `-` token marks the previous syllable as held.
    let mut result: Vec<Syllable> = Vec::new();
    for syllable in raw.syllables {
        if syllable.text == "-" {
            if let Some(last) = result.last_mut() {
                last.held = true;
            }
            result.push(Syllable {
                text: "-".to_string(),
                held: false,
                bracketed: false,
                pinyin: None,
                span: syllable.span,
            });
        } else {
            result.push(syllable);
        }
    }

    LyricTokens {
        syllables: result,
        errors: raw.errors,
    }
}

/// Whether `c` is a Chinese character, the only kind a `(pinyin)` attaches to.
pub(crate) fn is_han_char(c: char) -> bool {
    matches!(c as u32, 0x3400..=0x4DBF | 0x4E00..=0x9FFF)
}

/// The `(pinyin)` / `（pinyin）` written right after a Chinese character.
pub(crate) struct PinyinOverride {
    /// Where the closing parenthesis ends, relative to the line.
    pub(crate) end: usize,
    /// The reading, or the error to report when the text inside isn't one
    /// pinyin syllable.
    pub(crate) pinyin: Result<PinyinSyllable, RecoverableError>,
}

/// The override starting at `content[at..]`, if an opening parenthesis is
/// there and is closed on the same line.
pub(crate) fn pinyin_override_at(
    content: &str,
    at: usize,
    base_offset: usize,
) -> Option<PinyinOverride> {
    let rest = content.get(at..)?;
    let open = rest.chars().next().filter(|c| *c == '(' || *c == '（')?;
    let inner_start = at + open.len_utf8();
    let inner_len = content.get(inner_start..)?.find([')', '）'])?;
    let inner_end = inner_start + inner_len;
    let inner = content.get(inner_start..inner_end)?;
    let end = inner_end + content.get(inner_end..)?.chars().next()?.len_utf8();
    let pinyin = PinyinSyllable::parse(inner).ok_or_else(|| {
        RecoverableError::lyrics_pinyin_invalid(
            Span::new(base_offset + at, base_offset + end),
            inner,
        )
    });
    Some(PinyinOverride { end, pinyin })
}

/// One CJK character's syllable, with its attached reading when that is
/// valid pinyin (else the reading's error goes to `errors`).
fn cjk_syllable(
    c: char,
    span: Span,
    attached: Option<PinyinOverride>,
    errors: &mut Vec<RecoverableError>,
) -> Syllable {
    let pinyin = attached.and_then(|attached| attached.pinyin.map_err(|e| errors.push(e)).ok());
    Syllable {
        text: c.to_string(),
        held: false,
        bracketed: false,
        pinyin,
        span,
    }
}

/// The Latin word being read, by byte offsets into the lyric content.
#[derive(Default)]
struct LatinWord {
    text: String,
    start: Option<usize>,
    end: usize,
}

impl LatinWord {
    fn push(&mut self, idx: usize, c: char, char_end: usize) {
        self.start.get_or_insert(idx);
        self.text.push(c);
        self.end = char_end;
    }

    /// Ends the word, pushing it as a syllable if it has any text. It never
    /// holds leading or trailing whitespace (whitespace always flushes before
    /// it would be appended), so its own start/end bound the text exactly.
    fn flush(&mut self, base_offset: usize, raw: &mut Vec<Syllable>) {
        let trimmed = self.text.trim().to_string();
        if !trimmed.is_empty() {
            let start = self.start.unwrap_or(self.end);
            raw.push(Syllable {
                text: trimmed,
                held: false,
                bracketed: false,
                pinyin: None,
                span: Span::new(base_offset + start, base_offset + self.end),
            });
        }
        self.text.clear();
        self.start = None;
    }
}

/// The raw tokenization pass behind [`tokenize_lyrics`] — splits `content`
/// into one [`Syllable`] per CJK character or whitespace/`-`-delimited Latin
/// word, without yet resolving the "trailing `-` marks the previous syllable
/// as held" post-process (see the caller). A Chinese character's attached
/// `(pinyin)` stays part of its syllable.
fn tokenize_lyrics_raw(content: &str, base_offset: usize) -> LyricTokens {
    let mut raw: Vec<Syllable> = Vec::new();
    let mut errors: Vec<RecoverableError> = Vec::new();
    // Set past an override's closing parenthesis, to skip what it holds.
    let mut skip_until: usize = 0;
    let mut word = LatinWord::default();

    for (idx, c) in content.char_indices() {
        if idx < skip_until {
            continue;
        }
        let char_end = idx + c.len_utf8();
        if is_cjk_char(c) {
            word.flush(base_offset, &mut raw);
            let attached = is_han_char(c)
                .then(|| pinyin_override_at(content, char_end, base_offset))
                .flatten();
            if let Some(attached) = &attached {
                skip_until = attached.end;
            }
            raw.push(cjk_syllable(
                c,
                Span::new(base_offset + idx, base_offset + char_end),
                attached,
                &mut errors,
            ));
        } else if c == '-' && word.text.is_empty() {
            // Standalone `-` (surrounded by whitespace): held-syllable delimiter.
            raw.push(Syllable {
                text: "-".to_string(),
                held: false,
                bracketed: false,
                pinyin: None,
                span: Span::new(base_offset + idx, base_offset + char_end),
            });
        } else if c == '-' {
            // Trailing `-` attached to a word: syllable break across notes (e.g. `twin-`).
            word.push(idx, c, char_end);
            word.flush(base_offset, &mut raw);
        } else if c.is_whitespace() {
            word.flush(base_offset, &mut raw);
        } else {
            word.push(idx, c, char_end);
        }
    }
    word.flush(base_offset, &mut raw);

    LyricTokens {
        syllables: raw,
        errors,
    }
}

#[cfg(test)]
#[path = "utils_tests.rs"]
mod tests;
