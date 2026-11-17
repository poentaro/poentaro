//! Stream → measure chunks, re-barred by each measure's beat capacity.

use itertools::Itertools;

use crate::error::{Span, Spanned};
use crate::parser::score::timed_parser::{GroupStack, TimedLexToken};
use crate::parser::score::token_parser::TimedLineParse;

use super::diagnostics::{
    error_at, forbidden_character_errors, has_error, inline_directive_errors, placeholder_offsets,
    stream_diagnostic, AllowedMarkers,
};
use super::editable::TimedGrammar;
use super::timing::{normalize_whitespace, rest_fill, timed_layout, TimedCluster, TimedLayout};
use super::{StreamDiagnostic, FOLLOW_PLACEHOLDER};

/// Capacity used past the last measure group when no capacity is known at all.
const DEFAULT_CAPACITY: u32 = 16;

pub(super) struct SplitStream {
    /// Whitespace-normalized chunk per measure.
    pub(super) chunks: Vec<String>,
    pub(super) measure_starts: Vec<usize>,
    pub(super) diagnostics: Vec<StreamDiagnostic>,
    /// An error that re-barring can't cause or fix, so the stream should be
    /// left as typed. A note or tuplet crossing a barline doesn't count: the
    /// re-bar itself can move it there, and it stays a diagnostic.
    pub(super) blocks_rebarring: bool,
    /// No-lyrics placeholders pushed past the last slot by syllables typed
    /// before them, from the end of the last measure: the formatter deletes
    /// them, and Apply ignores them.
    pub(super) dropped_placeholders: Option<Span>,
}

/// Where a measure starts in the stream: the first cluster's index and the
/// byte offset the measure's chunk is cut at.
struct MeasureCut {
    cluster_index: usize,
    offset: usize,
    /// The measure is a follow part's `@`: it holds no clusters.
    is_placeholder: bool,
}

struct Layout {
    cuts: Vec<MeasureCut>,
    /// Quarter-beats of rest needed to complete the last measure.
    trailing_rest: u32,
}

/// One thing that takes up time in the stream, in stream order.
#[derive(Clone, Copy)]
enum StreamItem<'a> {
    Cluster {
        index: usize,
        cluster: &'a TimedCluster,
    },
    Placeholder {
        offset: usize,
    },
}

impl StreamItem<'_> {
    fn start(&self) -> usize {
        match self {
            Self::Cluster { cluster, .. } => cluster.start,
            Self::Placeholder { offset } => *offset,
        }
    }
}

pub(super) fn split_timed_stream(
    stream: &str,
    grammar: TimedGrammar,
    capacities: &[u32],
) -> SplitStream {
    let mut diagnostics = forbidden_character_errors(
        stream,
        AllowedMarkers {
            placeholder: grammar.is_follow,
            barlines: false,
        },
    );
    let placeholders = if grammar.is_follow {
        placeholder_offsets(stream)
    } else {
        Vec::new()
    };
    let sanitized = without_forbidden_characters(stream);
    let mut groups = GroupStack::default();
    let parsed = match grammar.kind.parse(&sanitized, &mut groups) {
        Ok(parsed) => parsed,
        Err(error) => {
            let span = error.span().copied().unwrap_or(Span::new(0, stream.len()));
            diagnostics.push(error_at(span, error.message()));
            return SplitStream {
                chunks: Vec::new(),
                measure_starts: Vec::new(),
                diagnostics,
                blocks_rebarring: true,
                dropped_placeholders: None,
            };
        }
    };
    let tokens = grammar.kind.lex(&sanitized).unwrap_or_default();
    let timed = timed_layout(&parsed.events);
    diagnostics.extend(parse_errors(&parsed, &groups, &timed));
    let mut blocks_rebarring = has_error(&diagnostics);
    let capacity_of = |measure: usize| {
        capacities
            .get(measure)
            .or(capacities.last())
            .copied()
            .unwrap_or(DEFAULT_CAPACITY)
            * timed.multiplier
    };
    let layout = lay_out(
        &timed.clusters,
        &placeholders,
        &tokens,
        capacity_of,
        &mut diagnostics,
    );
    let trailing_rest = layout.trailing_rest / timed.multiplier;
    if layout.trailing_rest % timed.multiplier != 0 {
        blocks_rebarring = true;
        diagnostics.push(error_at(
            Span::new(stream.len(), stream.len()),
            "the stream ends partway through a tuplet beat",
        ));
    }
    let ranges = measure_ranges(stream, &layout.cuts);
    SplitStream {
        chunks: measure_chunks(
            stream,
            &ranges,
            &layout.cuts,
            trailing_rest,
            &mut diagnostics,
        ),
        measure_starts: ranges
            .iter()
            .map(|range| first_non_whitespace(stream, range.start))
            .collect(),
        diagnostics,
        blocks_rebarring,
        dropped_placeholders: None,
    }
}

/// What the line parser and the timing layout found wrong with the stream.
fn parse_errors(
    parsed: &TimedLineParse,
    groups: &GroupStack,
    timed: &TimedLayout,
) -> Vec<StreamDiagnostic> {
    let lex_errors = parsed
        .lex_errors
        .iter()
        .map(|error| error_at(error.span, error.message()));
    let unclosed_groups = groups
        .frames
        .iter()
        .map(|frame| error_at(frame.open_paren_span, "this `(` is never closed"));
    let orphan_extensions = timed
        .orphan_extensions
        .iter()
        .map(|&span| error_at(span, "this `-` has no note before it to extend"));
    lex_errors
        .chain(parsed.chord_errors.iter().map(stream_diagnostic))
        .chain(inline_directive_errors(&parsed.events))
        .chain(unclosed_groups)
        .chain(orphan_extensions)
        .collect()
}

/// `|` and `@` become spaces so the line parser never sees them; offsets stay put.
fn without_forbidden_characters(stream: &str) -> String {
    stream
        .chars()
        .map(|character| {
            if matches!(character, '|' | '@') {
                ' '
            } else {
                character
            }
        })
        .collect()
}

/// Each measure's whitespace-normalized chunk; the last one is completed
/// with `trailing_rest` quarter-beats of rests.
fn measure_chunks(
    stream: &str,
    ranges: &[Span],
    cuts: &[MeasureCut],
    trailing_rest: u32,
    diagnostics: &mut Vec<StreamDiagnostic>,
) -> Vec<String> {
    let last_index = ranges.len().saturating_sub(1);
    ranges
        .iter()
        .zip(cuts)
        .enumerate()
        .map(|(index, (range, cut))| {
            let written =
                normalize_whitespace(stream.get(range.start..range.end).unwrap_or_default());
            if cut.is_placeholder && written != FOLLOW_PLACEHOLDER {
                diagnostics.push(error_at(
                    *range,
                    "`@` stands for a whole measure; nothing else can share its measure",
                ));
            }
            match (index == last_index, rest_fill(trailing_rest)) {
                (true, rest) if !rest.is_empty() => format!("{written} {rest}").trim().to_string(),
                _ => written,
            }
        })
        .collect()
}

/// The stream range each measure's chunk is cut from; the first one starts
/// at the stream's start so nothing before the first cluster is lost.
fn measure_ranges(stream: &str, cuts: &[MeasureCut]) -> Vec<Span> {
    let ends = cuts
        .iter()
        .skip(1)
        .map(|cut| cut.offset)
        .chain([stream.len()]);
    cuts.iter()
        .zip(ends)
        .enumerate()
        .map(|(index, (cut, end))| Span::new(if index == 0 { 0 } else { cut.offset }, end))
        .collect()
}

fn lay_out(
    clusters: &[TimedCluster],
    placeholders: &[usize],
    tokens: &[Spanned<TimedLexToken>],
    capacity_of: impl Fn(usize) -> u32,
    diagnostics: &mut Vec<StreamDiagnostic>,
) -> Layout {
    let items = clusters
        .iter()
        .enumerate()
        .map(|(index, cluster)| StreamItem::Cluster { index, cluster })
        .merge_by(
            placeholders
                .iter()
                .map(|&offset| StreamItem::Placeholder { offset }),
            |left, right| left.start() <= right.start(),
        );
    let mut cuts: Vec<MeasureCut> = Vec::new();
    let mut filled = 0;
    let mut next_cluster_index = 0;
    for stream_item in items {
        let starts_measure = cuts.is_empty() || filled >= capacity_of(cuts.len() - 1);
        let (index, cluster) = match stream_item {
            StreamItem::Placeholder { offset } => {
                if starts_measure {
                    cuts.push(MeasureCut {
                        cluster_index: next_cluster_index,
                        offset,
                        is_placeholder: true,
                    });
                    filled = capacity_of(cuts.len() - 1);
                } else {
                    diagnostics.push(error_at(
                        Span::new(offset, offset + FOLLOW_PLACEHOLDER.len()),
                        format!(
                            "`@` stands for a whole measure, but it starts partway through measure {}",
                            cuts.len()
                        ),
                    ));
                }
                continue;
            }
            StreamItem::Cluster { index, cluster } => (index, cluster),
        };
        next_cluster_index = index + 1;
        if starts_measure {
            if let Some(error) = tuplet_crossing_error(clusters, index, cuts.len()) {
                diagnostics.push(error);
            }
            cuts.push(MeasureCut {
                cluster_index: index,
                offset: cut_offset(cluster.start, tokens),
                is_placeholder: false,
            });
            filled = 0;
        }
        let capacity = capacity_of(cuts.len() - 1);
        if filled + cluster.duration > capacity {
            diagnostics.push(error_at(
                Span::new(cluster.start, cluster.end),
                format!(
                    "this note crosses the barline of measure {}; split it with a tie (`~`)",
                    cuts.len()
                ),
            ));
            filled = capacity;
        } else {
            filled += cluster.duration;
        }
    }
    diagnostics.extend(extension_in_tuplet_measure_errors(clusters, &cuts));
    let trailing_rest = cuts
        .len()
        .checked_sub(1)
        .map_or(0, |last| capacity_of(last).saturating_sub(filled));
    Layout {
        cuts,
        trailing_rest,
    }
}

/// A tuplet whose members land on both sides of a barline.
fn tuplet_crossing_error(
    clusters: &[TimedCluster],
    index: usize,
    measure: usize,
) -> Option<StreamDiagnostic> {
    let previous = clusters.get(index.checked_sub(1)?)?.tuplet?;
    let current = clusters.get(index)?.tuplet?;
    if previous.id != current.id {
        return None;
    }
    let members: Vec<&TimedCluster> = clusters
        .iter()
        .filter(|cluster| cluster.tuplet.is_some_and(|tuplet| tuplet.id == current.id))
        .collect();
    let start = members.first()?.start;
    let end = members.last()?.end;
    Some(error_at(
        Span::new(start, end),
        format!("this tuplet crosses the barline of measure {measure}"),
    ))
}

/// The real parser counts `-` on a tuplet measure's rescaled grid, so a
/// measure mixing the two could be re-barred differently than it renders.
fn extension_in_tuplet_measure_errors(
    clusters: &[TimedCluster],
    cuts: &[MeasureCut],
) -> Vec<StreamDiagnostic> {
    let ends = cuts
        .iter()
        .skip(1)
        .map(|cut| cut.cluster_index)
        .chain([clusters.len()]);
    cuts.iter()
        .zip(ends)
        .filter_map(|(cut, end)| {
            let measure = clusters.get(cut.cluster_index..end)?;
            let has_tuplet = measure.iter().any(|cluster| cluster.tuplet.is_some());
            let extended = measure.iter().find(|cluster| cluster.has_extension)?;
            has_tuplet.then(|| {
                error_at(
                    Span::new(extended.start, extended.end),
                    "a `-` extension can't share a measure with a tuplet in a part stream",
                )
            })
        })
        .collect()
}

/// Moves a cut back over the `(` / `N:{` that open right before a cluster,
/// so they travel with the measure they open.
fn cut_offset(cluster_start: usize, tokens: &[Spanned<TimedLexToken>]) -> usize {
    tokens
        .iter()
        .rev()
        .skip_while(|token| token.span.start >= cluster_start)
        .take_while(|token| {
            matches!(
                token.value,
                TimedLexToken::LParen | TimedLexToken::LBrace { .. }
            )
        })
        .last()
        .map_or(cluster_start, |token| token.span.start)
}

fn first_non_whitespace(stream: &str, from: usize) -> usize {
    stream
        .get(from..)
        .and_then(|rest| rest.find(|character: char| !character.is_whitespace()))
        .map_or(from, |offset| from + offset)
}

#[cfg(test)]
#[path = "split_tests.rs"]
mod tests;
