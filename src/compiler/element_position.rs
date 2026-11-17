//! Addressing one element of a `CompileResult`, for passes that pick
//! something for each element by looking across the whole score (the guitar
//! chord view's voicings, the harmonica-tab view's holes) and then write the
//! picks back.

use std::collections::HashMap;

use super::types::{ColumnElement, CompileResult, MeasureBlock, MeasureRow};

/// Where one element sits in a `CompileResult`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct ElementPosition {
    pub(super) block: usize,
    pub(super) row: usize,
    pub(super) element: usize,
}

/// One element, where it sits, and the part its row belongs to.
pub(super) struct PositionedElement<'a> {
    pub(super) position: ElementPosition,
    pub(super) source_part_index: usize,
    pub(super) element: &'a ColumnElement,
}

/// Every element of `blocks`, in written order.
pub(super) fn positioned_elements(
    blocks: &[MeasureBlock],
) -> impl Iterator<Item = PositionedElement<'_>> {
    blocks.iter().enumerate().flat_map(|(block_index, block)| {
        block
            .rows
            .iter()
            .enumerate()
            .flat_map(move |(row_index, row)| {
                row.elements
                    .iter()
                    .enumerate()
                    .map(move |(element_index, element)| PositionedElement {
                        position: ElementPosition {
                            block: block_index,
                            row: row_index,
                            element: element_index,
                        },
                        source_part_index: row.source_part_index,
                        element,
                    })
            })
    })
}

/// `result` with each element that has a pick in `picks` passed through
/// `apply` along with it.
pub(super) fn apply_picks<Pick>(
    result: CompileResult,
    mut picks: HashMap<ElementPosition, Pick>,
    apply: impl Fn(ColumnElement, Pick) -> ColumnElement,
) -> CompileResult {
    CompileResult {
        blocks: result
            .blocks
            .into_iter()
            .enumerate()
            .map(|(block_index, block)| MeasureBlock {
                rows: block
                    .rows
                    .into_iter()
                    .enumerate()
                    .map(|(row_index, row)| MeasureRow {
                        elements: row
                            .elements
                            .into_iter()
                            .enumerate()
                            .map(|(element_index, element)| {
                                let position = ElementPosition {
                                    block: block_index,
                                    row: row_index,
                                    element: element_index,
                                };
                                match picks.remove(&position) {
                                    Some(pick) => apply(element, pick),
                                    None => element,
                                }
                            })
                            .collect(),
                        ..row
                    })
                    .collect(),
                ..block
            })
            .collect(),
        ..result
    }
}
