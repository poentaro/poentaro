use crate::compiler::types::TupletSpan;
use crate::grid_layout::layout::MUSIC_START_COL;
use crate::grid_layout::types::{GridContent, GridElement, HAlign, VAlign};
use std::collections::HashMap;

use super::slur_placement::MeasurePlacement;

/// Resolves each `TupletSpan` to a `GridElement::TupletBracket`, keyed by
/// `(system_index, part_index)` like `resolve_slur_spans`'s arc map. Unlike
/// slur/tie arcs, a tuplet span always resolves within a single measure
/// (tuplets can't cross a line/system break — see the `crate::tuplet`
/// docs), so this only ever needs the same-system case of
/// `resolve_slur_spans` — no `TieOrSlurTail`/`TieOrSlurHead` equivalent.
pub(crate) fn resolve_tuplet_spans(
    tuplet_spans: &[TupletSpan],
    measure_placements: &[MeasurePlacement],
) -> HashMap<(usize, usize), Vec<GridElement>> {
    let mut bracket_map: HashMap<(usize, usize), Vec<GridElement>> = HashMap::new();

    for span in tuplet_spans {
        let Some(placement) = measure_placements.get(span.measure_index) else {
            continue;
        };
        let from_abs_col = MUSIC_START_COL + placement.column_offset + span.from_column;
        let to_abs_col = MUSIC_START_COL + placement.column_offset + span.to_column;
        let column_span = to_abs_col.saturating_sub(from_abs_col) + 1;
        bracket_map
            .entry((placement.system_index, span.part_index))
            .or_default()
            .push(GridElement {
                column: from_abs_col,
                column_span,
                halign: HAlign::Start,
                valign: VAlign::Center,
                content: GridContent::TupletBracket {
                    label: span.label.clone(),
                },
            });
    }

    bracket_map
}
