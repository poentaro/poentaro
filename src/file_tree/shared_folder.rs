//! A shared folder seen from the viewer's side (a synced `?s=` link or a
//! static `#share-folder=` link): a read-only [`FolderSnapshot`] with paths
//! relative to the shared folder. The viewer's list is rooted at that folder
//! (the folder itself is not a row), has no demo files or share markers, and
//! importing copies it into the viewer's own store under a unique name.

use super::folder_share::ShareOverview;
use super::folder_snapshot::FolderSnapshot;
use super::path::TreePath;
use super::placeholder::PLACEHOLDER_LEAF;
use super::store_state::{FileOperationContext, FileStoreError, FileStoreState};
use super::unique_name::unique_folder_path;
use super::view::{TreeViewInput, TreeViewRow};

/// The snapshot as a store keyed by the relative paths, with `active` as the
/// open file. This is what the viewer's tree is derived from; it is never
/// persisted.
fn viewer_state(snapshot: &FolderSnapshot, active: &str) -> Result<FileStoreState, FileStoreError> {
    snapshot
        .validate()
        .map_err(|error| FileStoreError::InvalidSharedFolder {
            reason: format!("{error:?}"),
        })?;
    Ok(FileStoreState {
        active: active.to_string(),
        user_files: snapshot
            .files
            .iter()
            .filter(|file| !is_placeholder(&file.path))
            .map(|file| (file.path.clone(), file.content.clone()))
            .collect(),
        folders: snapshot
            .empty_folders
            .iter()
            .filter_map(|folder| TreePath::parse(folder).ok())
            .collect(),
        ..FileStoreState::default()
    })
}

/// The file the viewer opens first: the first file in tree order (folders
/// before files, each alphabetical, depth-first), which is the first file row
/// of the fully expanded list. `None` for a folder holding no file.
pub fn shared_folder_first_file(snapshot: &FolderSnapshot) -> Option<String> {
    let rows = viewer_state(snapshot, "").ok()?.tree().ok()?.rows();
    rows.into_iter()
        .find(|row| !row.is_folder)
        .map(|row| row.path.as_str().to_string())
}

/// The visible rows of the viewer's file list, rooted at the shared folder:
/// Collapsed folders hide their contents, and the viewer has no `Demo/` row
/// or share markers.
pub fn shared_folder_view(
    snapshot: &FolderSnapshot,
    active: &str,
    expanded: &[String],
) -> Result<Vec<TreeViewRow>, FileStoreError> {
    let state = viewer_state(snapshot, active)?;
    let tree = state
        .tree()
        .map_err(|error| FileStoreError::InvalidSharedFolder {
            reason: error.to_string(),
        })?;
    Ok(tree.view_rows(&TreeViewInput {
        active,
        expanded,
        demo_file_names: &[],
        shares: &ShareOverview::default(),
        query: "",
        selected: &[],
    }))
}

/// Copies the shared folder into `state` as a new top-level folder named like
/// the shared one, or `Name 2`, `Name 3`, ... when that name is taken by a
/// file, folder, demo file or the bin. Nothing existing is touched. Files
/// keep their relative paths and take their ids from `file_ids` in snapshot
/// order (falling back to ids derived from `context.new_file_id`); the first
/// file in tree order becomes active. A folder without files is added as an
/// explicit folder, as are its empty subfolders.
pub fn import_shared_folder(
    state: &FileStoreState,
    snapshot: &FolderSnapshot,
    file_ids: &[String],
    context: &FileOperationContext,
) -> Result<FileStoreState, FileStoreError> {
    let first_file = shared_folder_first_file(snapshot);
    viewer_state(snapshot, "")?;
    let desired = TreePath::parse(&snapshot.folder)?;
    let root = unique_folder_path(&desired, &state.reserved_paths(context))?;
    let full = |relative: &str| TreePath::parse(&format!("{}/{relative}", root.as_str()));
    let mut next = state.clone();
    for (index, file) in snapshot
        .files
        .iter()
        .filter(|file| !is_placeholder(&file.path))
        .enumerate()
    {
        let path = full(&file.path)?.as_str().to_string();
        let id = file_ids
            .get(index)
            .cloned()
            .unwrap_or_else(|| format!("{}-{index}", context.new_file_id));
        next.user_files.insert(path.clone(), file.content.clone());
        next.file_ids.insert(path, id);
    }
    let empty_folders = snapshot
        .empty_folders
        .iter()
        .map(|folder| full(folder))
        .collect::<Result<Vec<_>, _>>()?;
    let root_as_folder =
        (snapshot.files.is_empty() && empty_folders.is_empty()).then(|| root.clone());
    next.folders
        .extend(empty_folders.into_iter().chain(root_as_folder));
    if let Some(first) = first_file {
        next.active = full(&first)?.as_str().to_string();
    }
    Ok(next)
}

/// Whether a relative file path is a `.keep` placeholder, which never counts
/// as a file (the worker already leaves them out; a static link never has any).
fn is_placeholder(relative: &str) -> bool {
    relative.rsplit('/').next() == Some(PLACEHOLDER_LEAF)
}
