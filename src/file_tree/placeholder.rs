//! The placeholder file that keeps an explicit empty folder alive in the
//! cloud. The cloud store only knows files (slash-separated names), so an
//! empty folder is persisted as the file `<folder>/.keep` with empty content.
//! The leaf name has no `.poentaro` extension, so a user file (which always gets
//! one) can never collide with it. Placeholders never reach
//! [`FileStoreState`](super::FileStoreState): the cloud listing is split by
//! [`classify_cloud_listing`] into real files, folders and hidden rows.

use super::path::{PathError, TreePath};

/// The reserved leaf name of a placeholder file.
pub const PLACEHOLDER_LEAF: &str = ".keep";

/// True when `path` names a placeholder file: a `.keep` leaf inside a folder.
pub fn is_placeholder_path(path: &str) -> bool {
    path.rsplit_once('/')
        .is_some_and(|(folder, leaf)| leaf == PLACEHOLDER_LEAF && !folder.is_empty())
}

/// True when `name` is the reserved leaf, which users may not use as a file
/// or folder name.
pub(super) fn is_reserved_leaf(name: &str) -> bool {
    name == PLACEHOLDER_LEAF
}

/// The placeholder file's path for `folder`.
pub fn placeholder_path(folder: &TreePath) -> Result<TreePath, PathError> {
    TreePath::child(Some(folder), PLACEHOLDER_LEAF)
}

/// One row of the cloud's file listing, reduced to what classification needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedEntry {
    pub id: String,
    pub name: String,
    pub trashed: bool,
}

/// A placeholder row: the folder it keeps, and the file id to rename, trash
/// or restore when that folder is renamed, deleted or created again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderPlaceholder {
    pub folder: TreePath,
    pub file_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ListingClassification {
    /// Explicit folders, from the active placeholder rows.
    pub folders: Vec<FolderPlaceholder>,
    /// Placeholder rows of deleted folders. They stay hidden, but still hold
    /// their name in the cloud (names are unique among trashed rows too), so
    /// creating the folder again restores one of these instead.
    pub trashed_folders: Vec<FolderPlaceholder>,
    /// Ids of every placeholder row, active or trashed; these rows are not
    /// files and are dropped before the listing becomes a file store.
    pub hidden_file_ids: Vec<String>,
}

/// Splits the placeholder rows out of a cloud listing. A trashed placeholder
/// (its folder was deleted) is hidden and keeps nothing alive.
pub fn classify_cloud_listing(entries: &[ListedEntry]) -> ListingClassification {
    let placeholders_where = |trashed: bool| -> Vec<FolderPlaceholder> {
        entries
            .iter()
            .filter(|entry| entry.trashed == trashed && is_placeholder_path(&entry.name))
            .filter_map(|entry| {
                let folder = TreePath::parse(&entry.name).ok()?.parent()?;
                Some(FolderPlaceholder {
                    folder,
                    file_id: entry.id.clone(),
                })
            })
            .collect()
    };
    let hidden_file_ids = entries
        .iter()
        .filter(|entry| is_placeholder_path(&entry.name))
        .map(|entry| entry.id.clone())
        .collect();
    ListingClassification {
        folders: placeholders_where(false),
        trashed_folders: placeholders_where(true),
        hidden_file_ids,
    }
}
