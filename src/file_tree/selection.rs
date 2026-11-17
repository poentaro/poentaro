//! Multi-select in the file browser. The selection is a plain list of file
//! paths; a folder is checked when every file inside it is. Everything the
//! checkboxes and the Move / Delete buttons need is decided here: checkbox
//! states, toggling, the units an action applies to, and whether a move would
//! be rejected.

use super::file_ops::rename_file;
use super::folder_ops::move_folder;
use super::folder_share::{folder_delete_share_warning, ShareOverview};
use super::move_targets::move_destination;
use super::path::TreePath;
use super::store_state::{FileOperationContext, FileStoreState};
use super::tree::{FileTree, FolderContents};
use itertools::Itertools;
use std::collections::BTreeSet;

/// A row's checkbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckState {
    Unchecked,
    Checked,
    /// A folder with some, but not all, of its files selected.
    Mixed,
}

/// One thing an action applies to: a file, or a whole folder whose files are
/// all selected (so moving it keeps its structure). Folders inside a selected
/// folder are not listed again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionUnit {
    pub path: String,
    pub is_folder: bool,
    /// Files in the unit: 1 for a file.
    pub file_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionMove {
    pub from: String,
    pub to: String,
    pub is_folder: bool,
}

/// The moves that put the selection into a folder, in the order to apply
/// them, with the first reason they would be rejected, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionMovePlan {
    /// Units already in the folder are left out (and moving nothing is itself
    /// an `error`, so the dialog can stay disabled).
    pub moves: Vec<SelectionMove>,
    pub error: Option<String>,
}

/// What the "Delete" confirmation shows for the selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionDeleteSummary {
    pub units: Vec<SelectionUnit>,
    /// One line per unit, e.g. `Hymns/Amazing.poentaro` or `Hymns/Advent/ (2 files)`.
    pub lines: Vec<String>,
    pub message: String,
    /// What deleting does to the live links involved; `None` when nothing is shared.
    pub share_warning: Option<String>,
}

fn is_selected(selected: &[String], path: &TreePath) -> bool {
    selected.iter().any(|chosen| chosen == path.as_str())
}

fn file_paths(contents: &FolderContents) -> Vec<&TreePath> {
    contents
        .files
        .iter()
        .map(|file| &file.path)
        .chain(
            contents
                .folders
                .iter()
                .flat_map(|folder| file_paths(&folder.contents)),
        )
        .collect()
}

pub(super) fn file_check(path: &TreePath, selected: &[String]) -> CheckState {
    if is_selected(selected, path) {
        CheckState::Checked
    } else {
        CheckState::Unchecked
    }
}

/// A folder with no files can never be checked.
pub(super) fn folder_check(contents: &FolderContents, selected: &[String]) -> CheckState {
    let files = file_paths(contents);
    match files
        .iter()
        .filter(|path| is_selected(selected, path))
        .count()
    {
        0 => CheckState::Unchecked,
        chosen if chosen == files.len() => CheckState::Checked,
        _ => CheckState::Mixed,
    }
}

impl FileTree {
    /// Every file path, for "Select all".
    pub fn select_all(&self) -> Vec<String> {
        file_paths(&self.root)
            .into_iter()
            .map(|path| path.as_str().to_string())
            .collect()
    }

    /// `selected` after clicking the checkbox of `key` (a file, or a folder
    /// meaning all files inside it): a file flips; a folder selects all of
    /// its files, or clears them when they were all selected already. Paths
    /// that no longer exist are dropped; an unknown `key` changes nothing.
    pub fn toggle_selection(&self, selected: &[String], key: &str) -> Vec<String> {
        let all = file_paths(&self.root);
        let affected: BTreeSet<&str> = TreePath::parse(key)
            .map(|key| {
                all.iter()
                    .filter(|path| **path == &key || path.is_within(&key))
                    .map(|path| path.as_str())
                    .collect()
            })
            .unwrap_or_default();
        let kept = all
            .iter()
            .map(|path| path.as_str())
            .filter(|path| selected.iter().any(|chosen| chosen == path));
        let all_affected_selected = !affected.is_empty()
            && affected
                .iter()
                .all(|path| selected.iter().any(|chosen| chosen == path));
        kept.filter(|path| !(all_affected_selected && affected.contains(path)))
            .chain(affected.iter().copied().filter(|_| !all_affected_selected))
            .unique()
            .sorted()
            .map(str::to_string)
            .collect()
    }

    /// What an action on `selected` applies to, in display order: every folder
    /// whose files are all selected (unless an enclosing folder is too), then
    /// the remaining selected files.
    pub fn selection_units(&self, selected: &[String]) -> Vec<SelectionUnit> {
        units_in(&self.root, selected)
    }
}

fn units_in(contents: &FolderContents, selected: &[String]) -> Vec<SelectionUnit> {
    contents
        .folders
        .iter()
        .flat_map(|folder| {
            if folder_check(&folder.contents, selected) == CheckState::Checked {
                vec![SelectionUnit {
                    path: folder.path.as_str().to_string(),
                    is_folder: true,
                    file_count: file_paths(&folder.contents).len(),
                }]
            } else {
                units_in(&folder.contents, selected)
            }
        })
        .chain(
            contents
                .files
                .iter()
                .filter(|file| is_selected(selected, &file.path))
                .map(|file| SelectionUnit {
                    path: file.path.as_str().to_string(),
                    is_folder: false,
                    file_count: 1,
                }),
        )
        .collect()
}

/// Every file path of the store, for "Select all"; empty if the store's
/// paths conflict.
pub fn select_all(state: &FileStoreState) -> Vec<String> {
    state
        .tree()
        .map(|tree| tree.select_all())
        .unwrap_or_default()
}

/// `selected` after clicking the checkbox of `key`; unchanged if the store's
/// paths conflict. See [`FileTree::toggle_selection`].
pub fn toggle_selection(state: &FileStoreState, selected: &[String], key: &str) -> Vec<String> {
    state.tree().map_or_else(
        |_| selected.to_vec(),
        |tree| tree.toggle_selection(selected, key),
    )
}

/// The units of `selected`; empty if the store's paths conflict.
pub fn selection_units(state: &FileStoreState, selected: &[String]) -> Vec<SelectionUnit> {
    state
        .tree()
        .map(|tree| tree.selection_units(selected))
        .unwrap_or_default()
}

/// The moves that put the selection into `folder` (`None` = top level), each
/// unit keeping its own name, checked one after the other against the store
/// so a clash between two selected items is caught too.
pub fn selection_move_plan(
    state: &FileStoreState,
    selected: &[String],
    folder: Option<&str>,
    context: &FileOperationContext,
) -> SelectionMovePlan {
    let moves = plan_moves(state, selected, folder);
    match moves {
        Err(error) => SelectionMovePlan {
            moves: vec![],
            error: Some(error),
        },
        Ok(moves) => {
            let applied = moves.iter().try_fold(state.clone(), |current, planned| {
                apply_move(&current, planned, context)
            });
            let error = applied.err().or_else(|| {
                moves
                    .is_empty()
                    .then(|| "Everything selected is already in that folder.".to_string())
            });
            SelectionMovePlan { moves, error }
        }
    }
}

fn plan_moves(
    state: &FileStoreState,
    selected: &[String],
    folder: Option<&str>,
) -> Result<Vec<SelectionMove>, String> {
    let tree = state.tree().map_err(|error| error.to_string())?;
    let destination = folder
        .map(TreePath::parse)
        .transpose()
        .map_err(|error| error.to_string())?;
    tree.selection_units(selected)
        .into_iter()
        .map(|unit| {
            let to = move_destination(&unit.path, destination.as_ref())
                .map_err(|error| error.to_string())?;
            Ok(SelectionMove {
                from: unit.path,
                to: to.as_str().to_string(),
                is_folder: unit.is_folder,
            })
        })
        .filter_ok(|planned| planned.from != planned.to)
        .collect()
}

fn apply_move(
    state: &FileStoreState,
    planned: &SelectionMove,
    context: &FileOperationContext,
) -> Result<FileStoreState, String> {
    if planned.is_folder {
        let from = TreePath::parse(&planned.from).map_err(|error| error.to_string())?;
        let to = TreePath::parse(&planned.to).map_err(|error| error.to_string())?;
        move_folder(state, &from, &to, context)
            .map(|outcome| outcome.state)
            .map_err(|error| error.to_string())
    } else {
        rename_file(state, &planned.from, &planned.to, context).map_err(|error| error.to_string())
    }
}

/// The confirmation for deleting the selection (files and whole folders move
/// to the bin).
pub fn selection_delete_summary(
    state: &FileStoreState,
    selected: &[String],
    shares: &ShareOverview,
) -> SelectionDeleteSummary {
    let units = selection_units(state, selected);
    let lines = units
        .iter()
        .map(|unit| {
            if unit.is_folder {
                format!("{}/ ({})", unit.path, files_phrase(unit.file_count))
            } else {
                unit.path.clone()
            }
        })
        .collect();
    let total: usize = units.iter().map(|unit| unit.file_count).sum();
    let message = match total {
        0 => "Nothing is selected.".to_string(),
        1 => "1 file will move to the bin. You can restore it from there.".to_string(),
        count => format!("{count} files will move to the bin. You can restore them from there."),
    };
    let share_warning = units
        .iter()
        .filter(|unit| unit.is_folder)
        .filter_map(|unit| TreePath::parse(&unit.path).ok())
        .filter_map(|folder| folder_delete_share_warning(shares, &folder))
        .join(" ");
    SelectionDeleteSummary {
        units,
        lines,
        message,
        share_warning: (!share_warning.is_empty()).then_some(share_warning),
    }
}

fn files_phrase(count: usize) -> String {
    match count {
        1 => "1 file".to_string(),
        count => format!("{count} files"),
    }
}
