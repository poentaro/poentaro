//! The payload of a static folder share link (`#share-folder=...`): a
//! snapshot of a whole folder subtree. This module owns the JSON shape and
//! its validation; compressing and base64url-encoding it (brotli lives in the
//! wasm crate) is done by `poentaro-wasm`. The single-file `#share=` payload
//! (`{filename, content}`) is unrelated and decodes as before.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::path::TreePath;
use super::store_state::FileStoreState;
use super::tree::FileTree;

/// Longest allowed static folder link, counted in characters of the part
/// after `#share-folder=` (the compressed snapshot as base64url). A link
/// longer than this is not offered: chat apps and mail clients often cut or
/// refuse links beyond a few thousand characters, and the dialog must never
/// hand out a link that might not open. 8,000 leaves room for a folder of a
/// dozen typical songs; bigger folders use a synced link.
pub const MAX_STATIC_FOLDER_LINK_CHARS: usize = 8_000;

/// A snapshot whose uncompressed JSON is longer than this is reported as too
/// long without compressing it (brotli at quality 11 is slow on megabytes,
/// and text this large cannot shrink below the limit above in practice).
pub const MAX_UNCOMPRESSED_SNAPSHOT_BYTES: usize = 250_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotFile {
    /// Path relative to the shared folder.
    pub path: String,
    pub content: String,
}

/// A folder subtree captured at one moment. Paths are relative to the shared
/// folder, which is named by `folder` (its own name, one segment).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderSnapshot {
    pub folder: String,
    pub files: Vec<SnapshotFile>,
    /// Relative paths of explicitly created folders that hold no file.
    #[serde(rename = "emptyFolders", default)]
    pub empty_folders: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    NotJson,
    /// `folder` is not a single valid path segment.
    InvalidFolderName,
    /// A relative path is invalid, or the whole path would be too long.
    InvalidPath {
        path: String,
    },
    /// Two files, or a file and a folder, claim the same path.
    Conflict {
        path: String,
    },
}

/// Captures `folder` from the store; `None` if it does not exist. Files and
/// explicit empty folders are sorted by path.
pub fn folder_snapshot(state: &FileStoreState, folder: &TreePath) -> Option<FolderSnapshot> {
    let files = state.files_within(folder);
    let explicit = state.explicit_folders_within(folder);
    if files.is_empty() && explicit.is_empty() {
        return None;
    }
    let relative = |path: &TreePath| {
        path.as_str()
            .get(folder.as_str().len() + 1..)
            .map(str::to_string)
    };
    let mut captured: Vec<SnapshotFile> = files
        .iter()
        .filter_map(|path| {
            Some(SnapshotFile {
                path: relative(path)?,
                content: state.user_files.get(path.as_str())?.clone(),
            })
        })
        .collect();
    captured.sort_by(|a, b| a.path.cmp(&b.path));
    let mut empty_folders: Vec<String> = explicit
        .iter()
        .filter(|path| *path != folder)
        .filter(|path| !files.iter().any(|file| file.is_within(path)))
        .filter_map(relative)
        .collect();
    empty_folders.sort();
    Some(FolderSnapshot {
        folder: folder.name().to_string(),
        files: captured,
        empty_folders,
    })
}

impl FolderSnapshot {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Parses and validates a snapshot: the folder name is one valid
    /// segment, every path (prefixed by the folder name) is a valid path of
    /// at most [`super::MAX_PATH_BYTES`] bytes, and no path is both a file
    /// and a folder.
    pub fn from_json(json: &str) -> Result<Self, SnapshotError> {
        let snapshot: Self = serde_json::from_str(json).map_err(|_| SnapshotError::NotJson)?;
        snapshot.validate()?;
        Ok(snapshot)
    }

    pub(super) fn validate(&self) -> Result<(), SnapshotError> {
        let root = TreePath::parse(&self.folder)
            .ok()
            .filter(|root| root.parent().is_none())
            .ok_or(SnapshotError::InvalidFolderName)?;
        let full = |relative: &str| {
            TreePath::parse(&format!("{}/{relative}", root.as_str())).map_err(|_| {
                SnapshotError::InvalidPath {
                    path: relative.to_string(),
                }
            })
        };
        let file_paths = self
            .files
            .iter()
            .map(|file| full(&file.path))
            .collect::<Result<Vec<_>, _>>()?;
        let folder_paths = self
            .empty_folders
            .iter()
            .map(|path| full(path))
            .collect::<Result<Vec<_>, _>>()?;
        let mut seen = HashSet::new();
        if let Some(duplicate) = file_paths.iter().find(|path| !seen.insert((*path).clone())) {
            return Err(SnapshotError::Conflict {
                path: duplicate.as_str().to_string(),
            });
        }
        FileTree::build(&file_paths, &folder_paths).map_err(|error| match error {
            super::tree::FileTreeError::FileFolderConflict { path } => SnapshotError::Conflict {
                path: path.as_str().to_string(),
            },
        })?;
        Ok(())
    }
}

/// Base64url without padding, the form the link carries.
pub fn base64url_encode(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Inverse of [`base64url_encode`]; `None` for anything else.
pub fn base64url_decode(text: &str) -> Option<Vec<u8>> {
    URL_SAFE_NO_PAD.decode(text).ok()
}
