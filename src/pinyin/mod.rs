//! One pinyin syllable, as written in a lyric's `字(pinyin)` override (see
//! "Pinyin for a polyphonic character" in `syntax.md`): tone marks (`háng`),
//! a tone number (`hang2`) or no tone (`hang`) all name the same reading.

mod syllables;

use itertools::Itertools;
use syllables::SYLLABLES;

/// The tone-marked forms of each vowel, first to fourth tone.
const TONE_MARKS: [(char, [char; 4]); 6] = [
    ('a', ['ā', 'á', 'ǎ', 'à']),
    ('e', ['ē', 'é', 'ě', 'è']),
    ('i', ['ī', 'í', 'ǐ', 'ì']),
    ('o', ['ō', 'ó', 'ǒ', 'ò']),
    ('u', ['ū', 'ú', 'ǔ', 'ù']),
    ('ü', ['ǖ', 'ǘ', 'ǚ', 'ǜ']),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinyinSyllable {
    /// The syllable without its tone, `ü` written as `ü` (`lü`, `hang`).
    letters: String,
    /// 1 to 4; `None` for the neutral tone, which pinyin writes unmarked.
    tone: Option<u8>,
}

/// One written letter: its toneless form and the tone its mark carries.
struct Letter {
    base: char,
    tone: Option<u8>,
}

fn letter(c: char) -> Option<Letter> {
    let marked = TONE_MARKS.iter().find_map(|(base, marks)| {
        let index = marks.iter().position(|mark| *mark == c)?;
        Some(Letter {
            base: *base,
            tone: u8::try_from(index + 1).ok(),
        })
    });
    marked.or(match c {
        'v' | 'ü' => Some(Letter {
            base: 'ü',
            tone: None,
        }),
        'a'..='z' => Some(Letter {
            base: c,
            tone: None,
        }),
        _ => None,
    })
}

impl PinyinSyllable {
    /// `text` as one Mandarin syllable, or `None` if it isn't one. At most
    /// one tone is given, as a mark or a trailing `1`–`5` (`5` is neutral).
    pub fn parse(text: &str) -> Option<Self> {
        let lower = text.to_lowercase();
        let (written, number) = match lower.chars().last()?.to_digit(10) {
            Some(digit @ 1..=5) => (lower.get(..lower.len() - 1)?, Some(digit)),
            Some(_) => return None,
            None => (lower.as_str(), None),
        };
        let letters: Vec<Letter> = written.chars().map(letter).collect::<Option<_>>()?;
        let marks: Vec<u8> = letters.iter().filter_map(|l| l.tone).collect();
        let tone = match (marks.as_slice(), number) {
            ([], None) => None,
            ([mark], None) => Some(u32::from(*mark)),
            ([], Some(digit)) => Some(digit),
            _ => return None,
        };
        let letters: String = letters.iter().map(|l| l.base).collect();
        let key = letters.replace('ü', "v");
        SYLLABLES.binary_search(&key.as_str()).ok()?;
        Some(Self {
            letters,
            tone: tone
                .filter(|tone| *tone != 5)
                .and_then(|tone| u8::try_from(tone).ok()),
        })
    }

    /// How the syllable is printed: with its tone mark (`háng`, `lǜ`).
    pub fn marked(&self) -> String {
        let Some(tone) = self.tone else {
            return self.letters.clone();
        };
        let vowels = self
            .letters
            .char_indices()
            .filter(|(_, c)| TONE_MARKS.iter().any(|(base, _)| base == c))
            .collect_vec();
        // `a` or `e` takes the mark, then the `o` of `ou`, else the last vowel.
        let carrier = vowels
            .iter()
            .find(|(_, c)| matches!(c, 'a' | 'e'))
            .or_else(|| {
                self.letters
                    .find("ou")
                    .and_then(|at| vowels.iter().find(|(i, _)| *i == at))
            })
            .or(vowels.last())
            .map(|(i, _)| *i);
        self.letters
            .char_indices()
            .map(|(i, c)| {
                let mark = TONE_MARKS
                    .iter()
                    .find(|(base, _)| *base == c)
                    .and_then(|(_, marks)| marks.get(usize::from(tone) - 1));
                match mark {
                    Some(mark) if Some(i) == carrier => *mark,
                    _ => c,
                }
            })
            .collect()
    }

    /// The syllable with a tone number, `ü` as `v` and `5` for neutral
    /// (`hang2`, `lv4`, `de5`): the form chimera's pinyin reader takes.
    pub fn numbered(&self) -> String {
        format!(
            "{}{}",
            self.letters.replace('ü', "v"),
            self.tone.unwrap_or(5)
        )
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod lyric_tests;
