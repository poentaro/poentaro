//! Where files land when the server picks their paths: a restored file
//! whose path was taken since, and every file of a moved folder. Pure, so
//! `poentaro app` and its tests share it.

use std::collections::HashSet;

use poentaro::file_tree::{unique_path, PathError, TreePath};

use crate::files::FileNameRow;
use crate::protocol::RenameBatchMove;

/// A file without its score: what `files/list` returns and what naming
/// looks at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSummaryRow {
    pub id: String,
    pub name: String,
    pub revision: i64,
    pub trashed_at: Option<i64>,
}

/// The path file `file_id` comes back to from the trash: `desired` (where
/// it was trashed from) if free, else `stem 2.ext`, `stem 3.ext`, ... in the
/// same folder. Every other file's path is taken, trashed or not (the
/// unique index covers both), and so is every folder of a file not in the
/// trash.
pub fn restored_path(
    file_id: &str,
    desired: &str,
    files: &[FileSummaryRow],
) -> Result<TreePath, PathError> {
    let desired = TreePath::parse(desired)?;
    let others: Vec<(TreePath, bool)> = files
        .iter()
        .filter(|file| file.id != file_id)
        .filter_map(|file| {
            let path = TreePath::parse(&file.name).ok()?;
            Some((path, file.trashed_at.is_none()))
        })
        .collect();
    let folders = others
        .iter()
        .filter(|(_, live)| *live)
        .flat_map(|(path, _)| path.ancestors());
    let taken: HashSet<TreePath> = others
        .iter()
        .map(|(path, _)| path.clone())
        .chain(folders)
        .collect();
    unique_path(&desired, &taken)
}

/// What moving folder `from` to `to` does, decided from the owner's files
/// that are not in the trash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FolderMovePlan {
    /// Rename these files, all or none.
    Move(Vec<RenameBatchMove>),
    /// No file is in `from`, but some are in `to`: a retry whose answer was
    /// lost, or a move onto itself.
    AlreadyMoved,
    /// No file is in `from` or `to`.
    NoSuchFolder,
    /// `to` is inside `from`.
    IntoItself,
    /// `to` already holds files. Folders never merge.
    DestinationExists,
}

pub fn plan_folder_move(from: &TreePath, to: &TreePath, live: &[FileNameRow]) -> FolderMovePlan {
    let moved: Vec<&FileNameRow> = live
        .iter()
        .filter(|file| within(&file.name, from))
        .collect();
    let at_destination = live.iter().any(|file| within(&file.name, to));
    if moved.is_empty() {
        return if at_destination {
            FolderMovePlan::AlreadyMoved
        } else {
            FolderMovePlan::NoSuchFolder
        };
    }
    if from == to {
        return FolderMovePlan::AlreadyMoved;
    }
    if within(to.as_str(), from) {
        return FolderMovePlan::IntoItself;
    }
    if at_destination {
        return FolderMovePlan::DestinationExists;
    }
    let moves = moved
        .into_iter()
        .map(|file| RenameBatchMove {
            id: file.id.clone(),
            name: format!("{}{}", to.as_str(), &file.name[from.as_str().len()..]),
        })
        .collect();
    FolderMovePlan::Move(moves)
}

/// Whether `name` is inside folder `folder`.
fn within(name: &str, folder: &TreePath) -> bool {
    name.strip_prefix(folder.as_str())
        .is_some_and(|rest| rest.starts_with('/'))
}
