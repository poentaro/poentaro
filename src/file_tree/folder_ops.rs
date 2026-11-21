//! Folder operations over [`FileStoreState`]. A folder is a path prefix, so
//! moving one rewrites the key of every file (and explicit empty folder)
//! under it; file ids travel with their files.

use super::file_ops::reject_reserved_leaf;
use super::path::TreePath;
use super::store_state::{FileOperationContext, FileStoreError, FileStoreState};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathKind {
    File,
    /// An explicitly created folder (implied folders have no entry of their
    /// own and are covered by the file paths beneath them).
    ExplicitFolder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathChange {
    pub kind: PathKind,
    pub old_path: TreePath,
    pub new_path: TreePath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderMoveOutcome {
    pub state: FileStoreState,
    /// Every stored path that moved; empty when `from == to`.
    pub changes: Vec<PathChange>,
}

/// Adds an explicit empty folder `name` (one path segment, trimmed) inside
/// `parent` (`None` = top level). It may not collide with a file, folder, demo
/// file or bin entry (`Collision`). The active file does not change.
pub fn create_folder(
    state: &FileStoreState,
    parent: Option<&TreePath>,
    name: &str,
    context: &FileOperationContext,
) -> Result<FileStoreState, FileStoreError> {
    let folder = TreePath::child(parent, name.trim())?;
    reject_reserved_leaf(&folder)?;
    if let Some(path) = state.ancestor_that_is_file(&folder, context) {
        return Err(FileStoreError::AncestorIsFile { path });
    }
    if state.reserved_paths(context).contains(&folder) {
        return Err(FileStoreError::Collision { path: folder });
    }
    let mut next = state.clone();
    next.folders.push(folder);
    Ok(next)
}

/// Renames/moves folder `from` to `to` as one all-or-nothing batch.
///
/// Rules (any violation changes nothing and returns the error):
/// - `from` must exist (it has a file, or is/contains an explicit folder);
/// - `to` may not be `from`'s own descendant (`MoveIntoItself`);
/// - no parent of `to` may be a file (`AncestorIsFile`);
/// - `to`, and every resulting file/folder path, must be a valid path within
///   1024 bytes, and must not already be a file, folder, demo file or bin
///   entry (`Collision`) outside the moved subtree. Folders never merge:
///   moving onto an existing folder is a collision, so "move into folder B"
///   is `to = B/<name>`.
///
/// Bin entries are not moved. The active file follows its file.
pub fn move_folder(
    state: &FileStoreState,
    from: &TreePath,
    to: &TreePath,
    context: &FileOperationContext,
) -> Result<FolderMoveOutcome, FileStoreError> {
    let files = state.files_within(from);
    let explicit = state.explicit_folders_within(from);
    if files.is_empty() && explicit.is_empty() {
        return Err(FileStoreError::NotFound {
            path: from.as_str().to_string(),
        });
    }
    if from == to {
        return Ok(FolderMoveOutcome {
            state: state.clone(),
            changes: Vec::new(),
        });
    }
    reject_reserved_leaf(to)?;
    if to.is_within(from) {
        return Err(FileStoreError::MoveIntoItself {
            folder: from.clone(),
            destination: to.clone(),
        });
    }
    if let Some(path) = state.ancestor_that_is_file(to, context) {
        return Err(FileStoreError::AncestorIsFile { path });
    }
    let changes = changes_for(from, to, &files, &explicit)?;
    let mut blocked = state.reserved_paths(context);
    blocked.retain(|path| path != from && !path.is_within(from));
    if let Some(change) = std::iter::once(to)
        .chain(changes.iter().map(|change| &change.new_path))
        .find(|path| blocked.contains(*path))
    {
        return Err(FileStoreError::Collision {
            path: change.clone(),
        });
    }
    Ok(FolderMoveOutcome {
        state: apply(state, &changes),
        changes,
    })
}

/// Bins every file at or inside `folder` individually, each under its own
/// full path as the bin key, and removes the folder along with any explicit
/// (including empty) subfolders. Fails with `NotFound` if there is nothing
/// to delete. If the active file was inside, the first remaining file (else
/// the first demo) becomes active.
pub fn delete_folder(
    state: &FileStoreState,
    folder: &TreePath,
    context: &FileOperationContext,
) -> Result<FileStoreState, FileStoreError> {
    let files = state.files_within(folder);
    let explicit = state.explicit_folders_within(folder);
    if files.is_empty() && explicit.is_empty() {
        return Err(FileStoreError::NotFound {
            path: folder.as_str().to_string(),
        });
    }
    let mut next = state.clone();
    files.iter().for_each(|file| {
        if let Some(content) = next.user_files.remove(file.as_str()) {
            next.bin.insert(file.as_str().to_string(), content);
        }
    });
    next.folders.retain(|path| !explicit.contains(path));
    if files.iter().any(|file| file.as_str() == state.active) {
        next.active = next.fallback_active(context);
    }
    Ok(next)
}

fn changes_for(
    from: &TreePath,
    to: &TreePath,
    files: &[TreePath],
    explicit: &[TreePath],
) -> Result<Vec<PathChange>, FileStoreError> {
    let change = |kind: PathKind| {
        move |old: &TreePath| -> Result<PathChange, FileStoreError> {
            Ok(PathChange {
                kind,
                old_path: old.clone(),
                new_path: old.rebased(from, to)?,
            })
        }
    };
    files
        .iter()
        .map(change(PathKind::File))
        .chain(explicit.iter().map(change(PathKind::ExplicitFolder)))
        .collect()
}

fn rekey<T: Clone>(
    renames: &HashMap<&str, &str>,
    map: &BTreeMap<String, T>,
) -> BTreeMap<String, T> {
    map.iter()
        .map(|(key, value)| {
            let new_key = renames.get(key.as_str()).copied().unwrap_or(key);
            (new_key.to_string(), value.clone())
        })
        .collect()
}

fn apply(state: &FileStoreState, changes: &[PathChange]) -> FileStoreState {
    let renames: HashMap<&str, &str> = changes
        .iter()
        .filter(|change| change.kind == PathKind::File)
        .map(|change| (change.old_path.as_str(), change.new_path.as_str()))
        .collect();
    let folder_renames: HashMap<&TreePath, &TreePath> = changes
        .iter()
        .filter(|change| change.kind == PathKind::ExplicitFolder)
        .map(|change| (&change.old_path, &change.new_path))
        .collect();
    FileStoreState {
        active: renames
            .get(state.active.as_str())
            .map_or_else(|| state.active.clone(), |new| new.to_string()),
        user_files: rekey(&renames, &state.user_files),
        // Bin keys stay: binned files remember where they were, not where
        // their former folder went.
        bin: state.bin.clone(),
        file_ids: rekey(&renames, &state.file_ids),
        folders: state
            .folders
            .iter()
            .map(|path| (*folder_renames.get(path).unwrap_or(&path)).clone())
            .collect(),
    }
}
