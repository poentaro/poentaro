//! Bodies of the file-store `Guest` methods that carry files saved by an
//! older build or origin over to the current one (see "Origin migration" and
//! "Rename to Poentaro" in `ARCHITECTURE.md`). No logic beyond conversion.
#![allow(clippy::needless_pass_by_value)]
use super::*;
use poentaro::file_tree as tree;

pub(super) fn file_store_merge_migrated(
    current: FileStoreState,
    carried: FileStoreState,
    context: FileOperationContext,
) -> FileStoreResponse {
    store_response(tree::merge_migrated_file_store(
        &current.into(),
        &carried.into(),
        &context.into(),
    ))
}

pub(super) fn file_store_rename_legacy_extensions(
    state: FileStoreState,
    context: FileOperationContext,
) -> Option<FileStoreState> {
    tree::rename_legacy_extensions(&state.into(), &context.into()).map(Into::into)
}
