//! Random part streams for the formatter's property tests: tokens from a
//! fixture's part, joined by random whitespace, with random cursors.

use itertools::Itertools;
use quickcheck::{Arbitrary, Gen};

use super::{format_part_stream, FormattedPartStream, StreamCursor};

pub(super) const NOTES_AND_LYRICS: &str = r#"parts ["Melody" [M]: notes, "Words" [W]: lyrics [M], "Accompaniment" [A]: follow [M]],

score """
    [M] 1 2 3 4
    [W] a b c d

    [M] 5 6 - 1
    [W] e f g

    [M] 3~ 3 5 6
    [A] 1 1 1 1
    """,
"#;

const CHORDS_AND_WORDS: &str = r#"parts ["Chords" [C]: chords, "Words" [W]: lyrics [C]],

score """
    [C] 1 - 4 -
    [W] (天) 空  好 (藍)

    [C] 0 5 - -
    [W] 我 （在）想 你
    """,
"#;

/// A part to generate streams for, and the tokens they're made of. Some
/// tokens are deliberately errors, so streams with errors are covered too.
pub(super) struct StreamFixture {
    pub(super) source: &'static str,
    pub(super) abbreviation: &'static str,
    tokens: &'static [&'static str],
}

const FIXTURES: &[StreamFixture] = &[
    StreamFixture {
        source: NOTES_AND_LYRICS,
        abbreviation: "M",
        tokens: &[
            "1",
            "2",
            "3",
            "5",
            "0",
            "-",
            "1'",
            "5,",
            "1.",
            "2_",
            "3_",
            "(1",
            "2)",
            "4~",
            "4",
            "3:{1 2 3}",
            "|",
            "(",
        ],
    },
    StreamFixture {
        source: NOTES_AND_LYRICS,
        abbreviation: "W",
        tokens: &["a", "b", "你", "(在)", "-", "_", "|"],
    },
    StreamFixture {
        source: NOTES_AND_LYRICS,
        abbreviation: "A",
        tokens: &["@", "1", "2", "-", "0", "5"],
    },
    StreamFixture {
        source: CHORDS_AND_WORDS,
        abbreviation: "W",
        tokens: &[
            "(天)",
            "空",
            "好",
            "(藍)",
            "|",
            "我",
            "（在）想",
            "_",
            "(",
            ")",
        ],
    },
];

const SEPARATORS: &[&str] = &["", " ", " ", " ", "  ", "\n", " \n ", "\n\n", "\t"];

/// One token of a generated stream and the whitespace before it.
#[derive(Debug, Clone)]
struct StreamPiece {
    separator: &'static str,
    token: &'static str,
}

#[derive(Debug, Clone)]
pub(super) struct GeneratedStream {
    fixture: usize,
    pieces: Vec<StreamPiece>,
    trailing: &'static str,
    /// Cursor positions, as a fraction of the stream's characters (per mille).
    cursor_positions: Vec<u16>,
}

impl GeneratedStream {
    pub(super) fn fixture(&self) -> &'static StreamFixture {
        &FIXTURES[self.fixture]
    }

    pub(super) fn text(&self) -> String {
        self.pieces
            .iter()
            .map(|piece| format!("{}{}", piece.separator, piece.token))
            .chain([self.trailing.to_string()])
            .collect()
    }

    pub(super) fn cursors(&self) -> Vec<StreamCursor> {
        let text = self.text();
        let boundaries = text
            .char_indices()
            .map(|(offset, _)| offset)
            .chain([text.len()])
            .collect_vec();
        self.cursor_positions
            .iter()
            .map(|&position| StreamCursor {
                offset: boundaries[usize::from(position) * (boundaries.len() - 1) / 1000],
            })
            .collect()
    }

    pub(super) fn formatted(&self) -> FormattedPartStream {
        let fixture = self.fixture();
        format_part_stream(
            fixture.source,
            fixture.abbreviation,
            &self.text(),
            &self.cursors(),
        )
    }
}

impl Arbitrary for GeneratedStream {
    fn arbitrary(generator: &mut Gen) -> Self {
        let fixture = usize::arbitrary(generator) % FIXTURES.len();
        let length = usize::arbitrary(generator) % 24;
        let pieces = (0..length)
            .map(|_| StreamPiece {
                separator: generator.choose(SEPARATORS).copied().unwrap_or(" "),
                token: generator
                    .choose(FIXTURES[fixture].tokens)
                    .copied()
                    .unwrap_or("0"),
            })
            .collect();
        let cursor_count = usize::arbitrary(generator) % 3;
        GeneratedStream {
            fixture,
            pieces,
            trailing: generator.choose(SEPARATORS).copied().unwrap_or(""),
            cursor_positions: (0..cursor_count)
                .map(|_| u16::arbitrary(generator) % 1001)
                .collect(),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let without_one_piece = (0..self.pieces.len()).map({
            let stream = self.clone();
            move |index| {
                let mut shrunk = stream.clone();
                shrunk.pieces.remove(index);
                shrunk
            }
        });
        let with_single_spaces = (0..self.pieces.len())
            .filter({
                let stream = self.clone();
                move |&index| stream.pieces[index].separator != " "
            })
            .map({
                let stream = self.clone();
                move |index| {
                    let mut shrunk = stream.clone();
                    shrunk.pieces[index].separator = " ";
                    shrunk
                }
            });
        let without_cursors = (!self.cursor_positions.is_empty()).then(|| GeneratedStream {
            cursor_positions: Vec::new(),
            ..self.clone()
        });
        let without_trailing = (!self.trailing.is_empty()).then(|| GeneratedStream {
            trailing: "",
            ..self.clone()
        });
        Box::new(
            without_cursors
                .into_iter()
                .chain(without_trailing)
                .chain(without_one_piece)
                .chain(with_single_spaces),
        )
    }
}
