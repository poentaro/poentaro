use crate::ast::parsed::TupletInfo;
use crate::compiler::types::TupletSpan;

/// A tuplet-bracket run currently being accumulated: the first and most
/// recent column tagged with the same `TupletInfo`, within one part slice.
/// Never carried across measures — tuplets can't span lines (see the `crate::tuplet`
/// docs), unlike `PendingSlurOpen` in `slur_chains.rs`.
pub(super) struct PendingTupletSpan {
    from_column: u32,
    to_column: u32,
    tuplet: TupletInfo,
}

pub(super) struct TupletSpanContext<'a> {
    pub(super) current: &'a mut Option<PendingTupletSpan>,
    pub(super) tuplet_spans: &'a mut Vec<TupletSpan>,
    pub(super) measure_index: usize,
    pub(super) part_index: usize,
}

fn flush(context: &mut TupletSpanContext<'_>) {
    if let Some(pending) = context.current.take() {
        context.tuplet_spans.push(TupletSpan {
            part_index: context.part_index,
            measure_index: context.measure_index,
            from_column: pending.from_column,
            to_column: pending.to_column,
            label: pending.tuplet.num.to_string(),
        });
    }
}

/// Called once per note/rest/chord-note/percussion-hit compiled, in source
/// order, with the column it was placed at and its (possibly `None`)
/// `TupletInfo` tag. Groups contiguous same-ratio tags into one `TupletSpan`;
/// any break (an untagged event, or a different ratio) flushes the run so
/// far.
///
/// Known limitation: two directly-adjacent tuplets sharing the same ratio
/// (no untagged event between them) aren't distinguished and merge into a
/// single bracket. Acceptable for now — it takes unusual back-to-back
/// same-ratio tuplet phrasing to trigger, and `TupletInfo` alone (just
/// `{num, den}`, no per-bracket identity) can't tell them apart.
pub(super) fn record_tuplet_tag(
    context: &mut TupletSpanContext<'_>,
    column: u32,
    tuplet: Option<TupletInfo>,
) {
    let Some(tuplet) = tuplet else {
        flush(context);
        return;
    };
    match context.current {
        Some(pending) if pending.tuplet == tuplet => {
            pending.to_column = column;
        }
        _ => {
            flush(context);
            *context.current = Some(PendingTupletSpan {
                from_column: column,
                to_column: column,
                tuplet,
            });
        }
    }
}

/// Flush any still-open tuplet span at the end of a part slice — a tuplet
/// that runs to the very end of a measure has no following untagged event
/// to trigger the flush in `record_tuplet_tag`.
pub(super) fn finish_tuplet_spans(context: &mut TupletSpanContext<'_>) {
    flush(context);
}
