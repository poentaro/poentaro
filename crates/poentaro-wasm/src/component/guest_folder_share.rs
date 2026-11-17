//! Bodies of the folder-sharing `Guest` methods: convert WIT values to
//! `poentaro::file_tree` types, call it, convert back. No logic
//! beyond conversion.
#![allow(clippy::needless_pass_by_value)]
use super::*;
use poentaro::file_tree as tree;

pub(super) fn file_store_folder_share_info(
    state: FileStoreState,
    shares: ShareOverview,
    folder: String,
) -> Option<FolderShareInfo> {
    let folder = tree::TreePath::parse(&folder).ok()?;
    tree::folder_share_info(&state.into(), &shares.into(), &folder).map(|info| FolderShareInfo {
        name: info.name,
        file_count: u32::try_from(info.file_count).unwrap_or(u32::MAX),
        folder_count: u32::try_from(info.folder_count).unwrap_or(u32::MAX),
        share_id: info.share_id,
        shared_via: info.shared_via.map(Into::into),
        own_link_warning: info.own_link_warning,
    })
}

pub(super) fn file_store_folder_static_share(
    state: FileStoreState,
    folder: String,
) -> Option<FolderStaticShare> {
    let folder = tree::TreePath::parse(&folder).ok()?;
    let snapshot = tree::folder_snapshot(&state.into(), &folder)?;
    let suffix = crate::folder_share_payload::encode_folder_snapshot(&snapshot);
    Some(FolderStaticShare {
        file_count: u32::try_from(snapshot.files.len()).unwrap_or(u32::MAX),
        folder_count: u32::try_from(subfolder_count(&snapshot)).unwrap_or(u32::MAX),
        too_long: suffix.is_none(),
        suffix,
    })
}

pub(super) fn decode_folder_share_suffix(suffix: String) -> Option<FolderSnapshot> {
    crate::folder_share_payload::decode_folder_snapshot(&suffix).map(|snapshot| FolderSnapshot {
        folder: snapshot.folder,
        files: snapshot
            .files
            .into_iter()
            .map(|file| SnapshotFile {
                path: file.path,
                content: file.content,
            })
            .collect(),
        empty_folders: snapshot.empty_folders,
    })
}

/// Folders inside the snapshot: every ancestor of a file, plus each empty
/// folder and its ancestors.
fn subfolder_count(snapshot: &tree::FolderSnapshot) -> usize {
    let parse = |relative: &String| tree::TreePath::parse(relative).ok();
    let above_files = snapshot
        .files
        .iter()
        .filter_map(|file| parse(&file.path))
        .flat_map(|path| path.ancestors());
    let empty_folders = snapshot
        .empty_folders
        .iter()
        .filter_map(parse)
        .flat_map(|path| {
            path.ancestors()
                .into_iter()
                .chain([path])
                .collect::<Vec<_>>()
        });
    above_files
        .chain(empty_folders)
        .map(|path| path.as_str().to_string())
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

impl From<ShareOverview> for tree::ShareOverview {
    fn from(overview: ShareOverview) -> Self {
        Self {
            files: overview
                .files
                .into_iter()
                .map(|link| tree::FileShareLink {
                    share_id: link.share_id,
                    file_id: link.file_id,
                    name: link.name,
                })
                .collect(),
            folders: overview
                .folders
                .into_iter()
                .map(|link| tree::FolderShareLink {
                    share_id: link.share_id,
                    folder: link.folder,
                })
                .collect(),
        }
    }
}

impl From<tree::SharedVia> for SharedVia {
    fn from(via: tree::SharedVia) -> Self {
        Self {
            folder: via.folder,
            name: via.name,
            share_id: via.share_id,
        }
    }
}

pub(super) fn file_store_folder_delete_summary(
    state: FileStoreState,
    folder: String,
    shares: ShareOverview,
) -> Option<FolderDeleteSummary> {
    let folder = tree::TreePath::parse(&folder).ok()?;
    tree::folder_delete_summary(&state.into(), &folder, &shares.into()).map(|summary| {
        FolderDeleteSummary {
            file_count: u32::try_from(summary.file_count).unwrap_or(u32::MAX),
            message: summary.message,
            share_warning: summary.share_warning,
        }
    })
}

impl From<FolderSnapshot> for tree::FolderSnapshot {
    fn from(snapshot: FolderSnapshot) -> Self {
        Self {
            folder: snapshot.folder,
            files: snapshot
                .files
                .into_iter()
                .map(|file| tree::SnapshotFile {
                    path: file.path,
                    content: file.content,
                })
                .collect(),
            empty_folders: snapshot.empty_folders,
        }
    }
}

pub(super) fn shared_folder_first_file(snapshot: FolderSnapshot) -> Option<String> {
    tree::shared_folder_first_file(&snapshot.into())
}

pub(super) fn shared_folder_view(
    snapshot: FolderSnapshot,
    active: String,
    expanded: Vec<String>,
) -> FileTreeViewResponse {
    match tree::shared_folder_view(&snapshot.into(), &active, &expanded) {
        Ok(rows) => FileTreeViewResponse::Ok(rows.into_iter().map(view_row).collect()),
        Err(error) => FileTreeViewResponse::Err(error.to_string()),
    }
}

pub(super) fn file_store_import_shared_folder(
    state: FileStoreState,
    snapshot: FolderSnapshot,
    file_ids: Vec<String>,
    context: FileOperationContext,
) -> FileStoreResponse {
    store_response(tree::import_shared_folder(
        &state.into(),
        &snapshot.into(),
        &file_ids,
        &context.into(),
    ))
}
