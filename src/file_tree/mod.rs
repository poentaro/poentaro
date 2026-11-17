//! Pure model of the web app's file tree: files live at slash-separated
//! paths, a folder is a shared path prefix (or an explicitly created empty
//! folder). No I/O and no wasm glue here; see ARCHITECTURE.md "File tree".

mod bin_entries;
mod extension;
mod file_ops;
mod folder_ops;
mod folder_queries;
mod folder_share;
mod folder_snapshot;
mod move_targets;
mod origin_migration;
mod path;
mod placeholder;
mod selection;
mod shared_folder;
mod store_state;
mod tree;
mod unique_name;
mod view;

pub use bin_entries::{bin_entries, BinEntry};
pub use extension::{
    current_extension_path, rename_legacy_extensions, strip_file_extension, FILE_EXTENSION,
    LEGACY_FILE_EXTENSION,
};
pub use file_ops::{
    create_file, create_file_error, delete_file, duplicate_file, rename_file, restore_file,
};
pub use folder_ops::{
    create_folder, delete_folder, move_folder, FolderMoveOutcome, PathChange, PathKind,
};
pub use folder_queries::{
    create_folder_error, folder_delete_summary, folder_move_error, folders_to_reveal,
    FolderDeleteSummary,
};
pub use folder_share::{
    folder_delete_share_warning, folder_share_info, FileShareLink, FolderShareInfo,
    FolderShareLink, ShareOverview, SharedVia,
};
pub use folder_snapshot::{
    base64url_decode, base64url_encode, folder_snapshot, FolderSnapshot, SnapshotError,
    SnapshotFile, MAX_STATIC_FOLDER_LINK_CHARS, MAX_UNCOMPRESSED_SNAPSHOT_BYTES,
};
pub use move_targets::{move_destination, move_targets, rename_error, MoveTarget};
pub use origin_migration::{merge_migrated_file_store, MIGRATED_FOLDER_NAME};
pub use path::{PathError, TreePath, MAX_PATH_BYTES};
pub use placeholder::{
    classify_cloud_listing, is_placeholder_path, placeholder_path, FolderPlaceholder, ListedEntry,
    ListingClassification, PLACEHOLDER_LEAF,
};
pub use selection::{
    select_all, selection_delete_summary, selection_move_plan, selection_units, toggle_selection,
    CheckState, SelectionDeleteSummary, SelectionMove, SelectionMovePlan, SelectionUnit,
};
pub use shared_folder::{import_shared_folder, shared_folder_first_file, shared_folder_view};
pub use store_state::{DemoFile, FileOperationContext, FileStoreError, FileStoreState};
pub use tree::{FileNode, FileTree, FileTreeError, FolderContents, FolderNode, TreeRow};
pub use unique_name::{path_after_collision, unique_folder_path, unique_path};
pub use view::{
    active_label, ActiveLabel, NameMatch, TreeViewInput, TreeViewRow, TreeViewRowKind,
    DEMO_EXPANSION_KEY,
};

#[cfg(test)]
mod bin_entries_tests;
#[cfg(test)]
mod extension_tests;
#[cfg(test)]
mod file_ops_tests;
#[cfg(test)]
mod folder_ops_tests;
#[cfg(test)]
mod folder_queries_tests;
#[cfg(test)]
mod folder_share_tests;
#[cfg(test)]
mod folder_snapshot_tests;
#[cfg(test)]
mod move_targets_tests;
#[cfg(test)]
mod origin_migration_tests;
#[cfg(test)]
mod path_tests;
#[cfg(test)]
mod placeholder_tests;
#[cfg(test)]
mod selection_tests;
#[cfg(test)]
mod shared_folder_tests;
#[cfg(test)]
mod tree_tests;
#[cfg(test)]
mod unique_name_tests;
#[cfg(test)]
mod view_tests;
