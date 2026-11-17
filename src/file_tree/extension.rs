//! The extension every saved file ends in (from `file-extension.json`, which
//! the web app reads too), and the rename of files saved before the app was
//! called Poentaro, which end in the legacy `.jianpu`.

use std::collections::BTreeMap;

use super::path::TreePath;
use super::store_state::{FileOperationContext, FileStoreState};

include!(concat!(env!("OUT_DIR"), "/file_extension_generated.rs"));

/// `name` as the user sees it: without its extension, current or legacy.
pub fn strip_file_extension(name: &str) -> &str {
    name.strip_suffix(FILE_EXTENSION)
        .or_else(|| name.strip_suffix(LEGACY_FILE_EXTENSION))
        .unwrap_or(name)
}

/// `path` with a legacy extension swapped for the current one, or `None`
/// when it doesn't end in the legacy extension.
pub fn current_extension_path(path: &str) -> Option<String> {
    path.strip_suffix(LEGACY_FILE_EXTENSION)
        .map(|stem| format!("{stem}{FILE_EXTENSION}"))
}

/// `state` with every file, active or binned, that ends in the legacy
/// extension renamed to end in the current one, keeping its content and id.
/// A file whose new path is already taken (by a file, folder, demo file or
/// bin entry) keeps its legacy name. `None` when nothing needs renaming.
pub fn rename_legacy_extensions(
    state: &FileStoreState,
    context: &FileOperationContext,
) -> Option<FileStoreState> {
    let reserved = state.reserved_paths(context);
    let renames: BTreeMap<&str, String> = state
        .user_files
        .keys()
        .chain(state.bin.keys())
        .filter_map(|path| {
            let renamed = current_extension_path(path)?;
            let taken = TreePath::parse(&renamed).map_or(true, |parsed| reserved.contains(&parsed));
            (!taken).then_some((path.as_str(), renamed))
        })
        .collect();
    if renames.is_empty() {
        return None;
    }
    let rename = |path: &String| renames.get(path.as_str()).unwrap_or(path).clone();
    let rename_keys = |entries: &BTreeMap<String, String>| {
        entries
            .iter()
            .map(|(path, value)| (rename(path), value.clone()))
            .collect()
    };
    Some(FileStoreState {
        active: rename(&state.active),
        user_files: rename_keys(&state.user_files),
        bin: rename_keys(&state.bin),
        file_ids: rename_keys(&state.file_ids),
        folders: state.folders.clone(),
    })
}
