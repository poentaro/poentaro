use poentaro_singer::SungSyllable;

use crate::audio_source::{singing_plan_for_measure_range_from_source, MeasureRangeSelection};
use crate::error::IrrecoverableError;

const SOURCE: &str = r#"parts ["Soprano" [S]: notes { instrument: soprano }, "Alto" [A]: notes { instrument: alto }, "Words" [w]: lyrics [S, A]],
score """
time=2/4 key=C4 bpm=60
[S] 1 2
[A] 1 2
[w] 我 们
""",
"#;

fn tracks(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| name.to_string()).collect()
}

/// The voices that sing when playing the first measure with `enabled`
/// parts and `visible` parts, each with the words it sings.
fn sung(enabled: &[&str], visible: Option<&[&str]>) -> Result<Vec<Sung>, IrrecoverableError> {
    let enabled = tracks(enabled);
    let visible = visible.map(tracks);
    let plan = singing_plan_for_measure_range_from_source(
        SOURCE,
        "test.poentaro",
        &MeasureRangeSelection {
            range: 0..=0,
            extend_to_last_occurrence: false,
            respect_sequence: false,
            sequence_entry_range: None,
        },
        Some(&enabled),
        visible.as_deref(),
        &[],
    )?;
    Ok(plan
        .parts
        .iter()
        .map(|part| {
            let words = part
                .notes
                .iter()
                .filter_map(|note| match &note.syllable {
                    SungSyllable::Word(word) | SungSyllable::Pinyin(word) => Some(word.clone()),
                    SungSyllable::Held | SungSyllable::Silent => None,
                })
                .collect();
            Sung {
                voice: format!("{:?}", part.voice),
                words,
            }
        })
        .collect())
}

/// A voice that sang, with the words it sang.
#[derive(Debug, PartialEq)]
struct Sung {
    voice: String,
    words: Vec<String>,
}

fn sings(voice: &str) -> Sung {
    Sung {
        voice: voice.to_string(),
        words: tracks(&["我", "们"]),
    }
}

#[test]
fn selected_notes_of_a_singing_part_sing_their_words() -> Result<(), IrrecoverableError> {
    assert_eq!(sung(&["S"], None)?, vec![sings("Soprano")]);
    Ok(())
}

#[test]
fn selected_lyrics_alone_sing_on_every_part_they_are_sung_on() -> Result<(), IrrecoverableError> {
    assert_eq!(sung(&["w"], None)?, vec![sings("Soprano"), sings("Alto")]);
    Ok(())
}

#[test]
fn selected_lyrics_with_some_of_their_singers_sing_on_those_only() -> Result<(), IrrecoverableError>
{
    assert_eq!(sung(&["S", "w"], None)?, vec![sings("Soprano")]);
    Ok(())
}

#[test]
fn lyrics_alone_never_bring_in_a_hidden_singer() -> Result<(), IrrecoverableError> {
    assert_eq!(sung(&["w"], Some(&["S", "w"]))?, vec![sings("Soprano")]);
    assert_eq!(sung(&["w"], Some(&["w"]))?, vec![]);
    Ok(())
}
