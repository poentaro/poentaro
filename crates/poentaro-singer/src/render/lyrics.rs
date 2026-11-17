//! Reading a phrase's lyrics into chimera syllables, one per note.

use std::io::Read;
use std::sync::OnceLock;

use chimera::phonetics::english::EnglishG2P;
use chimera::phonetics::korean::Korean;
use chimera::phonetics::mandarin::{Mandarin, MandarinLexicon, Pinyin};
use chimera::phonetics::sing::SungNote as ChimeraNote;
use chimera::phonetics::syllables::{read_lyrics, English, FrontEnd};
use chimera::phonetics::Syllable;

use crate::{SungNote, SungSyllable};

static MANDARIN_LEXICON: OnceLock<MandarinLexicon> = OnceLock::new();
static ENGLISH: OnceLock<EnglishG2P> = OnceLock::new();

fn mandarin_lexicon() -> &'static MandarinLexicon {
    MANDARIN_LEXICON.get_or_init(|| {
        let mut text = String::new();
        // A corrupt bundle reads as an empty lexicon: characters it can't
        // look up are then read one by one.
        let read = flate2::read::GzDecoder::new(&include_bytes!("data/mandarin.txt.gz")[..])
            .read_to_string(&mut text);
        let mut lexicon = MandarinLexicon::new();
        if read.is_ok() {
            lexicon.load(&text);
        }
        lexicon
    })
}

fn read_text(text: &str) -> Vec<Syllable> {
    // No pronouncing dictionary for English yet: its letter-to-sound rules only.
    let english = ENGLISH.get_or_init(EnglishG2P::new);
    let front_ends: [&dyn FrontEnd; 4] = [
        &Pinyin,
        &English(english),
        &Korean,
        &Mandarin(mandarin_lexicon()),
    ];
    read_lyrics(text, &front_ends)
        .into_iter()
        // The Mandarin reader bends each note toward its character's spoken
        // tone; in a song the melody already sets the pitch, so sing it flat.
        .map(|syllable| Syllable {
            pitch: Vec::new(),
            ..syllable
        })
        .collect()
}

/// A phrase's lyrics read together (so a Chinese word's characters are read
/// as one word), one chimera syllable per note.
pub(super) fn chimera_notes(phrase: &[&SungNote]) -> Vec<ChimeraNote> {
    let mut syllables = read_text(&phrase_text(phrase));
    if syllables.len() != phrase.len() {
        syllables = read_note_by_note(phrase);
    }
    if let Some(last) = syllables.last_mut() {
        last.phrase_end = true;
    }
    let next_starts = phrase
        .iter()
        .skip(1)
        .map(|next| Some(next.start_seconds))
        .chain([None]);
    phrase
        .iter()
        .zip(next_starts)
        .zip(syllables)
        .map(|((note, next_start), syl)| ChimeraNote {
            midi: f64::from(note.midi),
            beats: next_start.unwrap_or(note.end_seconds) - note.start_seconds,
            syl,
        })
        .collect()
}

/// The phrase's lyrics as one line: Chinese characters run together, and
/// every other syllable, including a held `-`, stands alone.
pub(super) fn phrase_text(phrase: &[&SungNote]) -> String {
    let words = phrase.iter().map(|note| match &note.syllable {
        SungSyllable::Word(word) | SungSyllable::Pinyin(word) => word.as_str(),
        SungSyllable::Held => "-",
        SungSyllable::Silent => "",
    });
    let mut text = String::new();
    let mut previous: Option<&str> = None;
    for current in words {
        if previous.is_some_and(|previous| !(is_all_cjk(previous) && is_all_cjk(current))) {
            text.push(' ');
        }
        text.push_str(current);
        previous = Some(current);
    }
    text
}

fn is_all_cjk(word: &str) -> bool {
    word.chars().all(|c| {
        matches!(c as u32,
            0x3040..=0x309F |  // Hiragana
            0x30A0..=0x30FF |  // Katakana
            0x3400..=0x4DBF |  // CJK Extension A
            0x4E00..=0x9FFF |  // CJK Unified Ideographs
            0xAC00..=0xD7AF    // Hangul
        )
    })
}

/// Fallback when reading the phrase whole doesn't give one syllable per note
/// (erhua, or a word its front end splits differently): read each note alone.
fn read_note_by_note(phrase: &[&SungNote]) -> Vec<Syllable> {
    phrase
        .iter()
        .scan(None::<Syllable>, |previous, note| {
            let syllable = match &note.syllable {
                SungSyllable::Word(word) | SungSyllable::Pinyin(word) => {
                    read_text(word).into_iter().next()
                }
                _ => None,
            }
            .or_else(|| previous.clone().map(held))
            .unwrap_or_default();
            *previous = Some(syllable.clone());
            Some(syllable)
        })
        .collect()
}

fn held(previous: Syllable) -> Syllable {
    Syllable {
        text: "~".into(),
        onset: vec![],
        coda: vec![],
        melisma: Some(true),
        phrase_end: false,
        ..previous
    }
}
