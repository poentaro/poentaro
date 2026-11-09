//! Conversions from `poentaro::highlight`'s types to the WIT
//! `highlight-token` and `section-header` families, by exhaustive match so a
//! kind added on either side fails to compile here.

use super::*;
use poentaro::highlight;
use poentaro::parser::section;

fn highlight_kind_to_wit(kind: highlight::HighlightKind) -> HighlightKind {
    match kind {
        highlight::HighlightKind::SectionHeader => HighlightKind::SectionHeader,
        highlight::HighlightKind::MetadataKey => HighlightKind::MetadataKey,
        highlight::HighlightKind::PartKind => HighlightKind::PartKind,
        highlight::HighlightKind::DirectiveKey => HighlightKind::DirectiveKey,
    }
}

pub(super) fn highlight_token_to_wit(token: highlight::HighlightToken) -> HighlightToken {
    HighlightToken {
        kind: highlight_kind_to_wit(token.kind),
        span: Span {
            start: token.span.start as u32,
            end: token.span.end as u32,
        },
    }
}

fn section_kind_to_wit(kind: section::SectionKind) -> SectionKind {
    match kind {
        section::SectionKind::Metadata => SectionKind::Metadata,
        section::SectionKind::Parts => SectionKind::Parts,
        section::SectionKind::Score => SectionKind::Score,
        section::SectionKind::Sequence => SectionKind::Sequence,
        section::SectionKind::Preset => SectionKind::Preset,
    }
}

pub(super) fn section_header_to_wit(header: highlight::SectionHeader) -> SectionHeader {
    SectionHeader {
        kind: section_kind_to_wit(header.kind),
        span: Span {
            start: header.span.start as u32,
            end: header.span.end as u32,
        },
    }
}
