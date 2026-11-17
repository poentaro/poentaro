//! Bodies of the multi-select `Guest` methods: convert WIT values to
//! `poentaro::file_tree` types, call it, convert back. No logic
//! beyond conversion.
#![allow(clippy::needless_pass_by_value)]
use super::*;
use poentaro::file_tree as tree;

pub(super) fn check_state(check: tree::CheckState) -> CheckState {
    match check {
        tree::CheckState::Unchecked => CheckState::Unchecked,
        tree::CheckState::Checked => CheckState::Checked,
        tree::CheckState::Mixed => CheckState::Mixed,
    }
}

pub(super) fn file_store_selection_all(state: FileStoreState) -> Vec<String> {
    tree::select_all(&state.into())
}

pub(super) fn file_store_selection_toggle(
    state: FileStoreState,
    selected: Vec<String>,
    key: String,
) -> Vec<String> {
    tree::toggle_selection(&state.into(), &selected, &key)
}

fn unit(unit: tree::SelectionUnit) -> SelectionUnit {
    SelectionUnit {
        path: unit.path,
        is_folder: unit.is_folder,
        file_count: u32::try_from(unit.file_count).unwrap_or(u32::MAX),
    }
}

pub(super) fn file_store_selection_units(
    state: FileStoreState,
    selected: Vec<String>,
) -> Vec<SelectionUnit> {
    tree::selection_units(&state.into(), &selected)
        .into_iter()
        .map(unit)
        .collect()
}

pub(super) fn file_store_selection_move_plan(
    state: FileStoreState,
    selected: Vec<String>,
    folder: Option<String>,
    context: FileOperationContext,
) -> SelectionMovePlan {
    let plan =
        tree::selection_move_plan(&state.into(), &selected, folder.as_deref(), &context.into());
    SelectionMovePlan {
        moves: plan
            .moves
            .into_iter()
            .map(|planned| SelectionMove {
                from_path: planned.from,
                to_path: planned.to,
                is_folder: planned.is_folder,
            })
            .collect(),
        error: plan.error,
    }
}

pub(super) fn file_store_selection_delete_summary(
    state: FileStoreState,
    selected: Vec<String>,
    shares: ShareOverview,
) -> SelectionDeleteSummary {
    let summary = tree::selection_delete_summary(&state.into(), &selected, &shares.into());
    SelectionDeleteSummary {
        units: summary.units.into_iter().map(unit).collect(),
        lines: summary.lines,
        message: summary.message,
        share_warning: summary.share_warning,
    }
}
