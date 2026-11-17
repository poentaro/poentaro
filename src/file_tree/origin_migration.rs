//! Merging the file store an origin migration carried over from the app's
//! old origin into the one the new origin already has (see "Origin
//! migration" in ARCHITECTURE.md). Usually the new origin has nothing yet
//! and simply takes the carried store whole; otherwise nothing existing is
//! touched and the carried files go under one new top-level folder.

use std::collections::{BTreeMap, HashSet};

use super::extension::rename_legacy_extensions;
use super::path::TreePath;
use super::store_state::{FileOperationContext, FileStoreError, FileStoreState};
use super::unique_name::unique_folder_path;

/// The top-level folder carried files go under when the new origin already
/// holds files (`MIGRATED_FOLDER_NAME 2`, ... when that is taken).
pub const MIGRATED_FOLDER_NAME: &str = "From jianpu-generator.pages.dev";

/// `carried` merged into `current`, after its files saved with the legacy
/// extension are renamed (see [`rename_legacy_extensions`]). A `current` with no files, bin entries or
/// folders is replaced by `carried` as is. Otherwise every carried file, bin
/// entry and explicit folder moves under a new top-level
/// [`MIGRATED_FOLDER_NAME`] folder, keeping its relative path and its id
/// (unless `current` already uses that id: then it gets one derived from
/// `context.new_file_id`). The active file stays `current`'s.
pub fn merge_migrated_file_store(
    current: &FileStoreState,
    carried: &FileStoreState,
    context: &FileOperationContext,
) -> Result<FileStoreState, FileStoreError> {
    let renamed = rename_legacy_extensions(carried, context);
    let carried = renamed.as_ref().unwrap_or(carried);
    if is_empty(current) {
        return Ok(carried.clone());
    }
    if is_empty(carried) {
        return Ok(current.clone());
    }
    let root = unique_folder_path(
        &TreePath::parse(MIGRATED_FOLDER_NAME)?,
        &current.reserved_paths(context),
    )?;
    let under_root = |path: &str| TreePath::parse(&format!("{}/{path}", root.as_str()));
    let used_ids: HashSet<&String> = current.file_ids.values().collect();
    let moved_ids = carried
        .file_ids
        .iter()
        .enumerate()
        .map(|(index, (path, id))| {
            let id = if used_ids.contains(id) {
                format!("{}-{index}", context.new_file_id)
            } else {
                id.clone()
            };
            Ok((under_root(path)?.as_str().to_string(), id))
        })
        .collect::<Result<BTreeMap<_, _>, FileStoreError>>()?;
    let moved = |entries: &BTreeMap<String, String>| {
        entries
            .iter()
            .map(|(path, content)| Ok((under_root(path)?.as_str().to_string(), content.clone())))
            .collect::<Result<BTreeMap<_, _>, FileStoreError>>()
    };
    let moved_folders = carried
        .folders
        .iter()
        .map(|folder| under_root(folder.as_str()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(FileStoreState {
        active: current.active.clone(),
        user_files: current
            .user_files
            .clone()
            .into_iter()
            .chain(moved(&carried.user_files)?)
            .collect(),
        bin: current
            .bin
            .clone()
            .into_iter()
            .chain(moved(&carried.bin)?)
            .collect(),
        file_ids: current
            .file_ids
            .clone()
            .into_iter()
            .chain(moved_ids)
            .collect(),
        folders: current
            .folders
            .iter()
            .cloned()
            .chain(moved_folders)
            .collect(),
    })
}

fn is_empty(state: &FileStoreState) -> bool {
    state.user_files.is_empty() && state.bin.is_empty() && state.folders.is_empty()
}
