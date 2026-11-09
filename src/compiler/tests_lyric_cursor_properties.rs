//! Property tests for how a lyric part follows its target's notes: random
//! melodies with random ties, rests and lyric-less measures, checked against
//! the notes the compiled syllables land on.

use itertools::Itertools;
use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};

use crate::compiler::{compile, types::*};
use crate::grouper::group;
use crate::lyric_spans::list_lyric_spans_from_source;
use crate::parser::parse;

const BEATS_PER_MEASURE: usize = 4;
const COLUMNS_PER_BEAT: u32 = 4;

/// One quarter-note beat of the melody.
#[derive(Debug, Clone, Copy)]
struct Beat {
    /// `0` is a rest.
    degree: u8,
    /// Tied into the next beat, which then repeats this degree.
    tied: bool,
}

#[derive(Debug, Clone)]
struct GeneratedMeasure {
    beats: Vec<Beat>,
    /// Whether the lyric part sings this measure or leaves it out.
    sung: bool,
    /// Whether a second part plays a note here, so a resting melody is hidden.
    other_part_plays: bool,
}

#[derive(Debug, Clone)]
struct GeneratedScore {
    measures: Vec<GeneratedMeasure>,
}

impl Arbitrary for GeneratedScore {
    fn arbitrary(g: &mut Gen) -> Self {
        let measure_count = usize::arbitrary(g) % 8 + 1;
        let beat_count = measure_count * BEATS_PER_MEASURE;
        let mut beats: Vec<Beat> = Vec::with_capacity(beat_count);
        for index in 0..beat_count {
            let degree = match beats.last() {
                Some(previous) if previous.tied => previous.degree,
                _ if u8::arbitrary(g) % 4 == 0 => 0,
                _ => u8::arbitrary(g) % 7 + 1,
            };
            let is_last = index + 1 == beat_count;
            let tied = degree != 0 && !is_last && bool::arbitrary(g);
            beats.push(Beat { degree, tied });
        }
        let measures = beats
            .chunks(BEATS_PER_MEASURE)
            .map(|beats| GeneratedMeasure {
                beats: beats.to_vec(),
                sung: bool::arbitrary(g),
                other_part_plays: u8::arbitrary(g) % 4 == 0,
            })
            .collect();
        GeneratedScore { measures }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        // Dropping trailing measures keeps every tie pointing at a real beat,
        // as long as the new last beat is untied.
        let shrunk = (1..self.measures.len())
            .rev()
            .map(|len| {
                let mut measures = self.measures[..len].to_vec();
                if let Some(last) = measures.last_mut().and_then(|m| m.beats.last_mut()) {
                    last.tied = false;
                }
                GeneratedScore { measures }
            })
            .collect_vec();
        Box::new(shrunk.into_iter())
    }
}

impl GeneratedScore {
    /// Beat indices within each measure that take a syllable: every note that
    /// isn't the continuation of a tie.
    fn slots(&self) -> Vec<Vec<usize>> {
        let mut entering_tied = false;
        self.measures
            .iter()
            .map(|measure| {
                measure
                    .beats
                    .iter()
                    .enumerate()
                    .filter_map(|(index, beat)| {
                        let is_slot = beat.degree != 0 && !entering_tied;
                        entering_tied = beat.tied;
                        is_slot.then_some(index)
                    })
                    .collect()
            })
            .collect()
    }

    fn source(&self) -> String {
        let slots = self.slots();
        let score = self
            .measures
            .iter()
            .zip(&slots)
            .enumerate()
            .map(|(measure_index, (measure, slots))| {
                let melody = measure
                    .beats
                    .iter()
                    .map(|beat| format!("{}{}", beat.degree, if beat.tied { "~" } else { "" }))
                    .join(" ");
                let other = if measure.other_part_plays {
                    "1 - - -"
                } else {
                    "0 0 0 0"
                };
                let mut lines = vec![format!("[M] {melody}"), format!("[B] {other}")];
                if measure.sung && !slots.is_empty() {
                    let syllables = slots
                        .iter()
                        .map(|beat| syllable(measure_index, *beat))
                        .join(" ");
                    lines.push(format!("[L] {syllables}"));
                }
                lines.join("\n")
            })
            .join("\n\n");
        format!(
            r#"parts ["M": notes, "L": lyrics [M], "B": notes],

score """
    time=4/4
    {score}
    """,
"#
        )
    }
}

fn syllable(measure_index: usize, beat: usize) -> String {
    let letter = |n: usize| char::from(b'a' + (n % 26) as u8);
    format!("{}{}", letter(measure_index), letter(beat))
}

struct CompiledLyric {
    column: u32,
    text: String,
    note_id: usize,
}

fn row<'a>(block: &'a MeasureBlock, id: &str) -> Option<&'a MeasureRow> {
    block.rows.iter().find(|row| row.id.0 == id)
}

fn lyrics_of(row: &MeasureRow) -> Vec<CompiledLyric> {
    row.elements
        .iter()
        .filter_map(|element| match &element.content {
            ElementContent::Lyric { text, note_id } => Some(CompiledLyric {
                column: element.column,
                text: text.clone(),
                note_id: *note_id,
            }),
            _ => None,
        })
        .collect()
}

fn check(property: fn(GeneratedScore) -> TestResult) {
    QuickCheck::new()
        .tests(1000)
        .quickcheck(property as fn(GeneratedScore) -> TestResult);
}

fn failure(score: &GeneratedScore, message: &str) -> TestResult {
    TestResult::error(format!("{message}\n{}", score.source()))
}

fn compile_source(source: &str) -> Result<CompileResult, String> {
    let document = parse(source, "test", &[]).map_err(|error| format!("{error:?}"))?;
    let score = group(document).map_err(|error| format!("{error:?}"))?;
    Ok(compile(&score))
}

#[test]
fn every_sung_measure_places_its_syllables_on_its_own_slots() {
    check(|score| {
        let compiled = match compile_source(&score.source()) {
            Ok(compiled) => compiled,
            Err(error) => return failure(&score, &error),
        };
        // No all-rest measure is sung, so rest-run merging never folds a sung
        // measure into another block.
        let compiled_lyrics: Vec<Vec<CompiledLyric>> = compiled
            .blocks
            .iter()
            .filter_map(|block| row(block, "L"))
            .map(lyrics_of)
            .filter(|lyrics| !lyrics.is_empty())
            .collect();
        let expected: Vec<Vec<(u32, String)>> = score
            .slots()
            .iter()
            .enumerate()
            .filter(|(measure_index, slots)| {
                score.measures[*measure_index].sung && !slots.is_empty()
            })
            .map(|(measure_index, slots)| {
                slots
                    .iter()
                    .map(|beat| {
                        (
                            *beat as u32 * COLUMNS_PER_BEAT,
                            syllable(measure_index, *beat),
                        )
                    })
                    .collect()
            })
            .collect();
        let actual: Vec<Vec<(u32, String)>> = compiled_lyrics
            .iter()
            .map(|lyrics| {
                lyrics
                    .iter()
                    .map(|lyric| (lyric.column, lyric.text.clone()))
                    .collect()
            })
            .collect();
        if actual == expected {
            TestResult::passed()
        } else {
            failure(&score, &format!("expected {expected:?}\n  got {actual:?}"))
        }
    });
}

#[test]
fn every_syllable_carries_the_note_id_of_the_note_under_it() {
    check(|score| {
        let compiled = match compile_source(&score.source()) {
            Ok(compiled) => compiled,
            Err(error) => return failure(&score, &error),
        };
        for (block_index, block) in compiled.blocks.iter().enumerate() {
            let (Some(melody), Some(lyrics)) = (row(block, "M"), row(block, "L")) else {
                continue;
            };
            for lyric in lyrics_of(lyrics) {
                let note_ids = melody
                    .elements
                    .iter()
                    .filter(|element| element.column == lyric.column)
                    .filter_map(|element| element.note_id)
                    .collect_vec();
                if !note_ids.contains(&lyric.note_id) {
                    return failure(
                        &score,
                        &format!(
                            "block {block_index}: {:?} at column {} has note id {}, notes there have {note_ids:?}",
                            lyric.text, lyric.column, lyric.note_id
                        ),
                    );
                }
            }
        }
        TestResult::passed()
    });
}

#[test]
fn lyric_source_spans_agree_with_compiled_note_ids() {
    check(|score| {
        let source = score.source();
        let compiled = match compile_source(&source) {
            Ok(compiled) => compiled,
            Err(error) => return failure(&score, &error),
        };
        let spans = match list_lyric_spans_from_source(&source, "test", None) {
            Ok(result) => result.spans,
            Err(error) => return failure(&score, &format!("{error:?}")),
        };
        let compiled_note_ids = compiled
            .blocks
            .iter()
            .filter_map(|block| row(block, "L"))
            .flat_map(lyrics_of)
            .map(|lyric| lyric.note_id)
            .collect_vec();
        let span_note_ids = spans.iter().map(|span| span.note_id).collect_vec();
        if compiled_note_ids == span_note_ids {
            TestResult::passed()
        } else {
            failure(
                &score,
                &format!("compiled {compiled_note_ids:?}\n  spans {span_note_ids:?}"),
            )
        }
    });
}
