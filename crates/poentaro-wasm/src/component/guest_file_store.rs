//! Bodies of the file-store `Guest` methods: convert WIT values to
//! `poentaro::file_tree` types, call it, convert back. No logic
//! beyond conversion.
#![allow(clippy::needless_pass_by_value)]
use super::*;
use poentaro::file_tree as tree;
use std::collections::BTreeMap;

pub(super) fn file_store_move_targets(
    state: FileStoreState,
    excluded_folder: Option<String>,
) -> Vec<MoveTarget> {
    let excluded = excluded_folder
        .as_deref()
        .and_then(|raw| tree::TreePath::parse(raw).ok());
    tree::move_targets(&state.into(), excluded.as_ref())
        .into_iter()
        .map(|target| MoveTarget {
            path: target.path,
            name: target.name,
            depth: u32::try_from(target.depth).unwrap_or(u32::MAX),
        })
        .collect()
}

pub(super) fn file_store_move_destination(
    source_path: String,
    folder: Option<String>,
) -> Option<String> {
    let folder = parse_optional(folder.as_deref()).ok()?;
    tree::move_destination(&source_path, folder.as_ref())
        .ok()
        .map(|path| path.as_str().to_string())
}

pub(super) fn file_store_classify_cloud_listing(
    entries: Vec<CloudListedEntry>,
) -> CloudListingClassification {
    let entries: Vec<tree::ListedEntry> = entries
        .into_iter()
        .map(|entry| tree::ListedEntry {
            id: entry.id,
            name: entry.name,
            trashed: entry.trashed,
        })
        .collect();
    let classified = tree::classify_cloud_listing(&entries);
    let convert = |placeholders: Vec<tree::FolderPlaceholder>| -> Vec<FolderPlaceholder> {
        placeholders
            .into_iter()
            .map(|placeholder| FolderPlaceholder {
                folder: placeholder.folder.as_str().to_string(),
                file_id: placeholder.file_id,
            })
            .collect()
    };
    CloudListingClassification {
        folders: convert(classified.folders),
        trashed_folders: convert(classified.trashed_folders),
        hidden_file_ids: classified.hidden_file_ids,
    }
}

pub(super) fn file_store_is_placeholder_path(path: String) -> bool {
    tree::is_placeholder_path(&path)
}

pub(super) fn file_store_placeholder_path(folder: String) -> Option<String> {
    let folder = tree::TreePath::parse(&folder).ok()?;
    tree::placeholder_path(&folder)
        .ok()
        .map(|path| path.as_str().to_string())
}

pub(super) fn file_store_name_after_collision(rejected: String) -> Option<String> {
    let rejected = tree::TreePath::parse(&rejected).ok()?;
    tree::path_after_collision(&rejected)
        .ok()
        .map(|path| path.as_str().to_string())
}

pub(super) fn file_store_rename_error(
    state: FileStoreState,
    from_path: String,
    to_path: String,
    context: FileOperationContext,
) -> Option<String> {
    tree::rename_error(&state.into(), &from_path, &to_path, &context.into())
}

pub(super) fn file_store_folder_move_error(
    state: FileStoreState,
    from_path: String,
    to_path: String,
    context: FileOperationContext,
) -> Option<String> {
    tree::folder_move_error(&state.into(), &from_path, &to_path, &context.into())
}

pub(super) fn file_store_folders_to_reveal(path: String) -> Vec<String> {
    tree::folders_to_reveal(&path)
}

pub(super) fn file_store_bin_entries(state: FileStoreState) -> Vec<BinEntry> {
    tree::bin_entries(&state.into())
        .into_iter()
        .map(|entry| BinEntry {
            key: entry.key,
            name: entry.name,
            location: entry.location,
        })
        .collect()
}

pub(super) fn file_store_duplicate_file(
    state: FileStoreState,
    source: String,
    context: FileOperationContext,
) -> FileStoreResponse {
    let outcome = parse(&source)
        .and_then(|source| tree::duplicate_file(&state.into(), &source, &context.into()));
    store_response(outcome)
}

pub(super) fn file_store_rename_file(
    state: FileStoreState,
    from_path: String,
    to_path: String,
    context: FileOperationContext,
) -> FileStoreResponse {
    store_response(tree::rename_file(
        &state.into(),
        &from_path,
        &to_path,
        &context.into(),
    ))
}

pub(super) fn file_store_delete_file(
    state: FileStoreState,
    path: String,
    context: FileOperationContext,
) -> FileStoreResponse {
    store_response(tree::delete_file(&state.into(), &path, &context.into()))
}

pub(super) fn file_store_restore_file(
    state: FileStoreState,
    bin_key: String,
    context: FileOperationContext,
) -> FileStoreResponse {
    store_response(tree::restore_file(&state.into(), &bin_key, &context.into()))
}

pub(super) fn file_store_delete_folder(
    state: FileStoreState,
    folder: String,
    context: FileOperationContext,
) -> FileStoreResponse {
    let outcome = parse(&folder)
        .and_then(|folder| tree::delete_folder(&state.into(), &folder, &context.into()));
    store_response(outcome)
}

pub(super) fn file_store_move_folder(
    state: FileStoreState,
    from_path: String,
    to_path: String,
    context: FileOperationContext,
) -> FolderMoveResponse {
    let outcome = parse(&from_path).and_then(|from| {
        let to = parse(&to_path)?;
        tree::move_folder(&state.into(), &from, &to, &context.into())
    });
    match outcome {
        Ok(moved) => FolderMoveResponse::Ok(FolderMoveSuccess {
            state: moved.state.into(),
            changes: moved.changes.into_iter().map(Into::into).collect(),
        }),
        Err(error) => FolderMoveResponse::Err(error.into()),
    }
}

pub(super) fn file_store_tree(state: FileStoreState) -> FileTreeResponse {
    let state: tree::FileStoreState = state.into();
    match state.tree() {
        Ok(file_tree) => FileTreeResponse::Ok(
            file_tree
                .rows()
                .into_iter()
                .map(|row| FileTreeRow {
                    path: row.path.as_str().to_string(),
                    name: row.name,
                    depth: u32::try_from(row.depth).unwrap_or(u32::MAX),
                    is_folder: row.is_folder,
                })
                .collect(),
        ),
        Err(error) => FileTreeResponse::Err(error.to_string()),
    }
}

pub(super) fn file_store_tree_view(
    state: FileStoreState,
    expanded: Vec<String>,
    demo_file_names: Vec<String>,
    shares: ShareOverview,
    query: String,
    selected: Vec<String>,
) -> FileTreeViewResponse {
    let shares: tree::ShareOverview = shares.into();
    let active = state.active.clone();
    let state: tree::FileStoreState = state.into();
    match state.tree() {
        Ok(file_tree) => FileTreeViewResponse::Ok(
            file_tree
                .view_rows(&tree::TreeViewInput {
                    active: &active,
                    expanded: &expanded,
                    demo_file_names: &demo_file_names,
                    shares: &shares,
                    query: &query,
                    selected: &selected,
                })
                .into_iter()
                .map(view_row)
                .collect(),
        ),
        Err(error) => FileTreeViewResponse::Err(error.to_string()),
    }
}

pub(super) fn view_row(row: tree::TreeViewRow) -> FileTreeViewRow {
    FileTreeViewRow {
        key: row.key,
        name: row.name,
        depth: u32::try_from(row.depth).unwrap_or(u32::MAX),
        kind: match row.kind {
            tree::TreeViewRowKind::Folder => FileTreeViewRowKind::Folder,
            tree::TreeViewRowKind::File => FileTreeViewRowKind::File,
            tree::TreeViewRowKind::DemoFolder => FileTreeViewRowKind::DemoFolder,
            tree::TreeViewRowKind::DemoFile => FileTreeViewRowKind::DemoFile,
            tree::TreeViewRowKind::EmptyFolderHint => FileTreeViewRowKind::EmptyFolderHint,
        },
        expanded: row.expanded,
        file_count: u32::try_from(row.file_count).unwrap_or(u32::MAX),
        is_active: row.is_active,
        share_id: row.share_id,
        shared_via: row.shared_via.map(Into::into),
        name_match: row.name_match.map(|found| NameMatch {
            start: u32::try_from(found.start).unwrap_or(u32::MAX),
            end: u32::try_from(found.end).unwrap_or(u32::MAX),
        }),
        check: row.check.map(check_state),
        ancestor_keys: row.ancestor_keys,
    }
}

pub(super) fn file_store_active_label(
    active: String,
    demo_file_names: Vec<String>,
) -> FileActiveLabel {
    let label = tree::active_label(&active, &demo_file_names);
    FileActiveLabel {
        folder_names: label.folder_names,
        file_name: label.file_name,
        folder_path: label.folder_path,
    }
}

fn parse(raw: &str) -> Result<tree::TreePath, tree::FileStoreError> {
    Ok(tree::TreePath::parse(raw)?)
}

pub(super) fn parse_optional(
    raw: Option<&str>,
) -> Result<Option<tree::TreePath>, tree::FileStoreError> {
    raw.map(parse).transpose()
}

pub(super) fn store_response(
    outcome: Result<tree::FileStoreState, tree::FileStoreError>,
) -> FileStoreResponse {
    match outcome {
        Ok(state) => FileStoreResponse::Ok(state.into()),
        Err(error) => FileStoreResponse::Err(error.into()),
    }
}

fn entries(map: BTreeMap<String, String>) -> Vec<StringEntry> {
    map.into_iter()
        .map(|(key, value)| StringEntry { key, value })
        .collect()
}

fn map_of(entries: Vec<StringEntry>) -> BTreeMap<String, String> {
    entries
        .into_iter()
        .map(|entry| (entry.key, entry.value))
        .collect()
}

impl From<FileStoreState> for tree::FileStoreState {
    fn from(state: FileStoreState) -> Self {
        Self {
            active: state.active,
            user_files: map_of(state.user_files),
            bin: map_of(state.bin),
            file_ids: map_of(state.file_ids),
            // A malformed stored folder path is dropped rather than failing
            // every operation; the files themselves are unaffected.
            folders: state
                .folders
                .iter()
                .filter_map(|raw| tree::TreePath::parse(raw).ok())
                .collect(),
        }
    }
}

impl From<tree::FileStoreState> for FileStoreState {
    fn from(state: tree::FileStoreState) -> Self {
        Self {
            active: state.active,
            user_files: entries(state.user_files),
            bin: entries(state.bin),
            file_ids: entries(state.file_ids),
            folders: state
                .folders
                .iter()
                .map(|folder| folder.as_str().to_string())
                .collect(),
        }
    }
}

impl From<FileOperationContext> for tree::FileOperationContext {
    fn from(context: FileOperationContext) -> Self {
        Self {
            demo_files: context
                .demo_files
                .into_iter()
                .filter_map(|demo| {
                    Some(tree::DemoFile {
                        path: tree::TreePath::parse(&demo.path).ok()?,
                        content: demo.content,
                    })
                })
                .collect(),
            new_file_id: context.new_file_id,
        }
    }
}

impl From<tree::PathChange> for PathChange {
    fn from(change: tree::PathChange) -> Self {
        Self {
            kind: match change.kind {
                tree::PathKind::File => PathKind::File,
                tree::PathKind::ExplicitFolder => PathKind::ExplicitFolder,
            },
            old_path: change.old_path.as_str().to_string(),
            new_path: change.new_path.as_str().to_string(),
        }
    }
}

impl From<tree::FileStoreError> for FileStoreFailure {
    fn from(error: tree::FileStoreError) -> Self {
        let kind = match &error {
            tree::FileStoreError::InvalidPath { .. } => FileStoreErrorKind::InvalidPath,
            tree::FileStoreError::NotFound { .. } => FileStoreErrorKind::NotFound,
            tree::FileStoreError::Collision { .. } => FileStoreErrorKind::Collision,
            tree::FileStoreError::AncestorIsFile { .. } => FileStoreErrorKind::AncestorIsFile,
            tree::FileStoreError::MoveIntoItself { .. } => FileStoreErrorKind::MoveIntoItself,
            tree::FileStoreError::DemoFileReadOnly { .. } => FileStoreErrorKind::DemoFileReadOnly,
            tree::FileStoreError::ReservedName { .. } => FileStoreErrorKind::ReservedName,
            tree::FileStoreError::InvalidSharedFolder { .. } => FileStoreErrorKind::InvalidPath,
        };
        Self {
            kind,
            message: error.to_string(),
        }
    }
}
