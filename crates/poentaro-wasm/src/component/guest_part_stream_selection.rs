//! Real bodies for the part focus editor's selection `Guest` methods:
//! byte ranges between a part stream and its draft source. Kept at the
//! exact by-value parameter types the `Guest` trait requires (see
//! `mod.rs`'s doc comment), so `needless_pass_by_value` is relaxed here.
#![allow(clippy::needless_pass_by_value)]

use super::*;

pub(super) fn part_stream_ranges_to_source(
    source: String,
    abbreviation: String,
    edited_stream: String,
    ranges: Vec<Span>,
) -> Vec<Span> {
    let ranges = ranges.into_iter().map(span_from_wit).collect::<Vec<_>>();
    poentaro::part_stream::part_stream_ranges_to_source(
        &source,
        &abbreviation,
        &edited_stream,
        &ranges,
    )
    .into_iter()
    .map(span_to_wit)
    .collect()
}

pub(super) fn source_ranges_to_part_stream(
    source: String,
    abbreviation: String,
    edited_stream: String,
    ranges: Vec<Span>,
) -> Vec<Span> {
    let ranges = ranges.into_iter().map(span_from_wit).collect::<Vec<_>>();
    poentaro::part_stream::source_ranges_to_part_stream(
        &source,
        &abbreviation,
        &edited_stream,
        &ranges,
    )
    .into_iter()
    .map(span_to_wit)
    .collect()
}

fn span_from_wit(span: Span) -> poentaro::error::Span {
    poentaro::error::Span::new(span.start as usize, span.end as usize)
}

fn span_to_wit(span: poentaro::error::Span) -> Span {
    Span {
        start: span.start as u32,
        end: span.end as u32,
    }
}
