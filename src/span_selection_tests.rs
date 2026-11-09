//! Property tests for re-deriving a selection after parts are hidden: random
//! multi-part scores, random hidden parts and random selected byte ranges,
//! listed through the real span pipeline.

use std::collections::BTreeSet;

use itertools::Itertools;
use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};

use super::*;
use crate::lyric_spans::list_lyric_spans_from_source;
use crate::note_spans::list_note_spans_from_source;

/// A part's note or syllable, named in a way that survives hiding other
/// parts: `source_part_index` is renumbered then, the source bytes are not.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SelectedToken {
    part_abbreviation: Option<String>,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone)]
struct GeneratedCase {
    source: String,
    abbreviations: Vec<String>,
    hidden: Vec<String>,
    ranges: Vec<ByteRange>,
}

fn choose<T: Clone>(gen: &mut Gen, options: &[T]) -> T {
    gen.choose(options).expect("options are not empty").clone()
}

fn below(gen: &mut Gen, bound: usize) -> usize {
    usize::arbitrary(gen) % bound
}

impl Arbitrary for GeneratedCase {
    fn arbitrary(gen: &mut Gen) -> Self {
        let note_part_count = 1 + below(gen, 3);
        let measure_count = 1 + below(gen, 3);
        let note_parts: Vec<String> = (1..=note_part_count).map(|n| format!("P{n}")).collect();
        let lyric_parts: Vec<(String, String)> = note_parts
            .iter()
            .filter(|_| bool::arbitrary(gen))
            .map(|target| (format!("L{target}"), target.clone()))
            .collect();

        let declarations = note_parts
            .iter()
            .map(|abbreviation| format!("\"Part {abbreviation}\" [{abbreviation}] : notes"))
            .chain(lyric_parts.iter().map(|(abbreviation, target)| {
                format!("\"Words {abbreviation}\" [{abbreviation}] : lyrics [{target}]")
            }))
            .join(", ");

        let measures = (0..measure_count)
            .map(|_| {
                note_parts
                    .iter()
                    .flat_map(|abbreviation| {
                        // The first note is never a rest: an all-rest part
                        // is hidden only while another part has notes, so
                        // hiding that other part would rightly reveal it.
                        let notes: Vec<char> = std::iter::once(choose(gen, &['1', '5']))
                            .chain(
                                (1..4).map(|_| choose(gen, &['0', '1', '2', '3', '5', '6', '7'])),
                            )
                            .collect();
                        let sung = notes.iter().filter(|note| **note != '0').count();
                        let note_line = format!("[{abbreviation}] {}", notes.iter().join(" "));
                        let lyric_line = lyric_parts
                            .iter()
                            .find(|(_, target)| target == abbreviation)
                            .filter(|_| sung > 0)
                            .map(|(lyrics, _)| {
                                format!("[{lyrics}] {}", (0..sung).map(|_| "la").join(" "))
                            });
                        std::iter::once(note_line).chain(lyric_line)
                    })
                    .join("\n")
            })
            .join("\n\n");

        let source = format!("parts [{declarations}],\n\nscore \"\"\"\n{measures}\n\"\"\",\n");
        let abbreviations: Vec<String> = note_parts
            .into_iter()
            .chain(
                lyric_parts
                    .into_iter()
                    .map(|(abbreviation, _)| abbreviation),
            )
            .collect();
        let hidden = abbreviations
            .iter()
            .filter(|_| bool::arbitrary(gen))
            .cloned()
            .collect();
        let ranges = (0..below(gen, 4))
            .map(|_| {
                let start = below(gen, source.len());
                let end = (start + below(gen, 40)).min(source.len());
                ByteRange {
                    start_byte: start as u32,
                    end_byte: end as u32,
                }
            })
            .collect();
        GeneratedCase {
            source,
            abbreviations,
            hidden,
            ranges,
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let case = self.clone();
        let fewer_ranges = (0..self.ranges.len()).map(move |skipped| GeneratedCase {
            ranges: case
                .ranges
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != skipped)
                .map(|(_, range)| *range)
                .collect(),
            ..case.clone()
        });
        let case = self.clone();
        let fewer_hidden = (0..self.hidden.len()).map(move |skipped| GeneratedCase {
            hidden: case
                .hidden
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != skipped)
                .map(|(_, abbreviation)| abbreviation.clone())
                .collect(),
            ..case.clone()
        });
        Box::new(fewer_ranges.chain(fewer_hidden))
    }
}

impl GeneratedCase {
    /// The parts left drawn, as the web app passes them: `None` when every
    /// part is drawn.
    fn visible_tracks(&self) -> Option<Vec<String>> {
        (!self.hidden.is_empty()).then(|| {
            self.abbreviations
                .iter()
                .filter(|abbreviation| !self.hidden.contains(abbreviation))
                .cloned()
                .collect()
        })
    }

    fn is_hidden(&self, part_abbreviation: Option<&String>) -> bool {
        part_abbreviation.is_some_and(|abbreviation| self.hidden.contains(abbreviation))
    }

    fn failure(&self, message: &str) -> TestResult {
        TestResult::error(format!(
            "{message}\n  hidden: {:?}\n  ranges: {:?}\n  source:\n{}",
            self.hidden,
            self.ranges
                .iter()
                .map(|range| &self.source[range.start_byte as usize..range.end_byte as usize])
                .collect_vec(),
            self.source
        ))
    }
}

fn selected_note_tokens(
    case: &GeneratedCase,
    visible: Option<&[String]>,
) -> BTreeSet<SelectedToken> {
    let spans = list_note_spans_from_source(&case.source, "test.jianpu", visible)
        .expect("generated scores compile")
        .spans;
    let selection = select_notes_in_byte_ranges(&case.ranges, &spans);
    selection
        .cells
        .iter()
        .map(|cell| {
            let span = spans
                .iter()
                .find(|span| {
                    span.source_part_index == cell.source_part_index && span.note_id == cell.note_id
                })
                .expect("every selected cell comes from a listed span");
            SelectedToken {
                part_abbreviation: span.part_abbreviation.clone(),
                start: span.start.expect("selected notes have bytes"),
                end: span.end.expect("selected notes have bytes"),
            }
        })
        .collect()
}

fn selected_lyric_tokens(
    case: &GeneratedCase,
    visible: Option<&[String]>,
) -> BTreeSet<SelectedToken> {
    let spans = list_lyric_spans_from_source(&case.source, "test.jianpu", visible)
        .expect("generated scores compile")
        .spans;
    let selection = select_lyrics_in_byte_ranges(&case.ranges, &spans);
    selection
        .cells
        .iter()
        .map(|cell| {
            let span = spans
                .iter()
                .find(|span| {
                    span.source_part_index == cell.source_part_index && span.note_id == cell.note_id
                })
                .expect("every selected cell comes from a listed span");
            SelectedToken {
                part_abbreviation: span.part_abbreviation.clone(),
                start: span.start,
                end: span.end,
            }
        })
        .collect()
}

fn check(property: fn(GeneratedCase) -> TestResult) {
    QuickCheck::new()
        .tests(500)
        .quickcheck(property as fn(GeneratedCase) -> TestResult);
}

#[test]
fn hiding_parts_removes_exactly_their_notes_from_the_selection() {
    check(|case| {
        let visible = case.visible_tracks();
        let with_hidden_parts = selected_note_tokens(&case, visible.as_deref());
        let expected: BTreeSet<SelectedToken> = selected_note_tokens(&case, None)
            .into_iter()
            .filter(|token| !case.is_hidden(token.part_abbreviation.as_ref()))
            .collect();
        if with_hidden_parts == expected {
            TestResult::passed()
        } else {
            case.failure(&format!(
                "selected {with_hidden_parts:?}\n  expected {expected:?}"
            ))
        }
    });
}

#[test]
fn hiding_parts_removes_exactly_their_lyrics_from_the_selection() {
    check(|case| {
        let visible = case.visible_tracks();
        let with_hidden_parts = selected_lyric_tokens(&case, visible.as_deref());
        let expected: BTreeSet<SelectedToken> = selected_lyric_tokens(&case, None)
            .into_iter()
            .filter(|token| !case.is_hidden(token.part_abbreviation.as_ref()))
            .collect();
        if with_hidden_parts == expected {
            TestResult::passed()
        } else {
            case.failure(&format!(
                "selected {with_hidden_parts:?}\n  expected {expected:?}"
            ))
        }
    });
}

#[test]
fn a_selection_never_plays_a_hidden_part() {
    check(|case| {
        let visible = case.visible_tracks();
        let spans = list_note_spans_from_source(&case.source, "test.jianpu", visible.as_deref())
            .expect("generated scores compile")
            .spans;
        let selection = select_notes_in_byte_ranges(&case.ranges, &spans);
        match selection
            .runs
            .iter()
            .find(|run| case.is_hidden(run.part_abbreviation.as_ref()))
        {
            None => TestResult::passed(),
            Some(run) => case.failure(&format!("selection plays hidden part {run:?}")),
        }
    });
}

#[test]
fn a_bare_caret_selects_nothing() {
    let source = r#"parts ["Melody" [M]: notes],

score """
    [M] 1 2 3 4
    """,
"#;
    let spans = list_note_spans_from_source(source, "test.jianpu", None)
        .unwrap()
        .spans;
    let caret = source.find('2').unwrap() as u32;
    let selection = select_notes_in_byte_ranges(
        &[ByteRange {
            start_byte: caret,
            end_byte: caret,
        }],
        &spans,
    );
    assert_eq!(selection.cells, vec![]);
    assert_eq!(selection.runs, vec![]);
}
