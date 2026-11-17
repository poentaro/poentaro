//! Pure, D1-free logic for sharing a whole folder. A folder share (see
//! `live-share-worker/migrations/0005_folder_shares.sql`) names a folder by
//! its path prefix, not by file ids: the viewer reads every active file whose
//! name starts with `<prefix>/` at that moment, so files added, edited,
//! renamed, moved in or out and deleted show up on the viewer's next load.
//! What lives here: validating the folder path, the size limits, building the
//! viewer's tree, and deciding which folder shares a batch rename moves.

use std::collections::{BTreeSet, HashMap};

use poentaro::file_tree::{is_placeholder_path, TreePath, PLACEHOLDER_LEAF};

use crate::api_error::ApiError;
use crate::protocol::{RenameBatchMove, SharedFolder, SharedFolderFile, SyncedDoc};
use crate::share::empty_doc;

/// Most files (placeholders included) one folder share may cover. A viewer
/// reads the whole tree in one response, so the size is bounded instead of
/// silently truncated: starting such a share, or loading one that has grown
/// past it, is refused with `ApiError::SharedFolderTooLarge`.
pub const MAX_SHARED_FOLDER_FILES: i64 = 300;

/// Most bytes of file content one folder share may cover (10 MiB).
pub const MAX_SHARED_FOLDER_BYTES: i64 = 10 * 1024 * 1024;

/// What `queries/get_folder_files_stats.sql` counts under a folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolderStats {
    pub file_count: i64,
    pub content_bytes: i64,
}

/// What `queries/get_folder_share_header.sql` returns for a folder share.
/// `owner_user_id` is only for reading the folder's files; it never goes on
/// the wire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderShareHeader {
    pub ended_at: Option<i64>,
    pub owner_user_id: String,
    pub folder_prefix: String,
}

/// One active file under the shared folder, as stored (full path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderFileRow {
    pub name: String,
    pub content: String,
    pub revision: i64,
}

/// Validates the folder path of a folder-share request: the shared
/// `TreePath` rules, and the reserved `.keep` placeholder name is not a
/// folder name.
pub fn validate_folder_path(folder: &str) -> Result<TreePath, ApiError> {
    let path = TreePath::parse(folder).map_err(|error| ApiError::InvalidName {
        message: error.to_string(),
    })?;
    if path.segments().any(|segment| segment == PLACEHOLDER_LEAF) {
        return Err(ApiError::InvalidName {
            message: format!("`{PLACEHOLDER_LEAF}` is reserved"),
        });
    }
    Ok(path)
}

/// Refuses a folder that is over the limits.
pub fn check_size(stats: FolderStats) -> Result<(), ApiError> {
    if stats.file_count > MAX_SHARED_FOLDER_FILES || stats.content_bytes > MAX_SHARED_FOLDER_BYTES {
        Err(ApiError::SharedFolderTooLarge {
            message: format!(
                "a shared folder holds at most {MAX_SHARED_FOLDER_FILES} files and {MAX_SHARED_FOLDER_BYTES} bytes of content; this one has {} files and {} bytes",
                stats.file_count, stats.content_bytes
            ),
        })
    } else {
        Ok(())
    }
}

/// The viewer's response for a folder share. A stopped share keeps only
/// `ended`. A live share lists every file by its
/// path relative to the shared folder (the owner's ancestor folder names
/// never leak) in path order, and `empty_folders`: the relative folders kept
/// only by a `.keep` placeholder, with no file anywhere under them (the
/// viewer unions them with the folders its files imply). `.keep` rows are
/// never listed as files. A folder that has no files at all is a live, empty
/// tree (the owner may repopulate it); the share does not end.
///
/// `filename` repeats the folder name and `content` is empty so an older
/// client that knows only single-file shares shows an empty file instead of
/// failing; `revision` is 0 -- each file carries its own.
pub fn to_public_folder_doc(header: &FolderShareHeader, rows: &[FolderFileRow]) -> SyncedDoc {
    if header.ended_at.is_some() {
        return empty_doc();
    }
    let name = last_segment(&header.folder_prefix).to_string();
    let relative = |row: &FolderFileRow| -> String {
        row.name
            .strip_prefix(&header.folder_prefix)
            .and_then(|rest| rest.strip_prefix('/'))
            .unwrap_or(&row.name)
            .to_string()
    };
    let files: Vec<SharedFolderFile> = rows
        .iter()
        .map(|row| (relative(row), row))
        .filter(|(path, _)| !is_placeholder_path(path) && path != PLACEHOLDER_LEAF)
        .map(|(path, row)| SharedFolderFile {
            path,
            content: row.content.clone(),
            revision: row.revision,
        })
        .collect();
    let empty_folders: Vec<String> = rows
        .iter()
        .map(relative)
        .filter_map(|path| path.strip_suffix("/.keep").map(str::to_string))
        .filter(|folder| {
            !files
                .iter()
                .any(|file| file.path.starts_with(&format!("{folder}/")))
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    SyncedDoc {
        ended: false,
        filename: name.clone(),
        content: String::new(),
        revision: 0,
        folder: Some(SharedFolder {
            name,
            files,
            empty_folders,
        }),
    }
}

fn last_segment(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// A folder share whose prefix a batch rename changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixRewrite {
    pub from: String,
    pub to: String,
}

/// An active file of the caller, by id and current name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveFileName {
    pub id: String,
    pub name: String,
}

/// Which of the caller's folder shares (`prefixes`) a batch rename moves.
/// The batch carries no folder information, only files, so a folder counts
/// as moved when the batch moves every one of its active files (`current`:
/// the active files under the prefixes, by current name) to the same new
/// folder with the rest of each path unchanged (`Hymns/a/x` -> `Songs/a/x`).
/// That is exactly what a folder rename or move produces; moving only some
/// of the files, or moving them out to differing places, leaves the share
/// where it is (it follows the prefix, not the files). Matching is by whole
/// path segments: `Hymns` does not cover `Hymns 2/...`. A nested shared
/// folder moves with its parent and gets its own rewrite. A folder with no
/// active file cannot be told from the batch and is not rewritten.
pub fn plan_prefix_rewrites(
    prefixes: &[String],
    moves: &[RenameBatchMove],
    current: &[ActiveFileName],
) -> Vec<PrefixRewrite> {
    let new_names: HashMap<&str, &str> = moves
        .iter()
        .map(|rename| (rename.id.as_str(), rename.name.as_str()))
        .collect();
    prefixes
        .iter()
        .filter_map(|prefix| {
            let destination = common_destination(prefix, &new_names, current)?;
            (destination != *prefix).then(|| PrefixRewrite {
                from: prefix.clone(),
                to: destination,
            })
        })
        .collect()
}

/// The one folder every active file under `prefix` is moved into, keeping
/// the rest of its path; `None` when some file stays, goes elsewhere or
/// changes more than its folder.
fn common_destination(
    prefix: &str,
    new_names: &HashMap<&str, &str>,
    current: &[ActiveFileName],
) -> Option<String> {
    let destinations: Vec<Option<String>> = current
        .iter()
        .filter_map(|file| {
            let rest = file.name.strip_prefix(prefix)?.strip_prefix('/')?;
            Some((file, rest))
        })
        .map(|(file, rest)| {
            let new_name = new_names.get(file.id.as_str())?;
            new_name
                .strip_suffix(&format!("/{rest}"))
                .map(str::to_string)
        })
        .collect();
    let first = destinations.first()?.as_ref()?;
    let valid = destinations.iter().all(|d| d.as_ref() == Some(first));
    (valid && TreePath::parse(first).is_ok()).then(|| first.clone())
}
