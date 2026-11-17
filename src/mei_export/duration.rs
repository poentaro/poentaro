//! Writing numbered durations (in sixteenths) as MEI note values.

/// A duration one MEI note can write: `@dur` plus `@dots`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SimpleDuration {
    pub(super) sixteenths: u32,
    pub(super) dur: u32,
    pub(super) dots: u32,
}

const fn simple(sixteenths: u32, dur: u32, dots: u32) -> SimpleDuration {
    SimpleDuration {
        sixteenths,
        dur,
        dots,
    }
}

/// Longest first: whole, dotted half, half, dotted quarter, quarter, dotted
/// eighth, eighth, sixteenth.
const SIMPLE_DURATIONS: [SimpleDuration; 8] = [
    simple(16, 1, 0),
    simple(12, 2, 1),
    simple(8, 2, 0),
    simple(6, 4, 1),
    simple(4, 4, 0),
    simple(3, 8, 1),
    simple(2, 8, 0),
    simple(1, 16, 0),
];

const SIXTEENTH: SimpleDuration = simple(1, 16, 0);

impl SimpleDuration {
    /// MEI `@dots`, or nothing for an undotted value.
    pub(super) fn dots_attribute(self) -> String {
        if self.dots > 0 {
            format!(" dots=\"{}\"", self.dots)
        } else {
            String::new()
        }
    }
}

/// Greedily splits a duration into values MEI can write directly, longest
/// first; the caller ties the pieces.
pub(super) fn split_duration(sixteenths: u32) -> Vec<SimpleDuration> {
    let mut remaining = sixteenths;
    let mut pieces = Vec::new();
    while remaining > 0 {
        let piece = largest_fitting(remaining);
        remaining = remaining.saturating_sub(piece.sixteenths);
        pieces.push(piece);
    }
    pieces
}

fn largest_fitting(sixteenths: u32) -> SimpleDuration {
    SIMPLE_DURATIONS
        .iter()
        .find(|duration| duration.sixteenths <= sixteenths)
        .copied()
        .unwrap_or(SIXTEENTH)
}

/// Like [`split_duration`], but a dotted value becomes its undotted value
/// plus the dot's half, for notes whose dots must not be drawn.
pub(super) fn split_undotted_duration(sixteenths: u32) -> Vec<SimpleDuration> {
    split_duration(sixteenths)
        .into_iter()
        .flat_map(|piece| match piece.dots {
            0 => vec![piece],
            _ => {
                let main = piece.sixteenths * 2 / 3;
                vec![
                    simple(main, piece.dur, 0),
                    simple(main / 2, piece.dur * 2, 0),
                ]
            }
        })
        .collect()
}
