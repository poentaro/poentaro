//! The New file and New folder `Guest` methods: convert WIT values to
//! `poentaro::file_tree` types, call it, convert back.
#![allow(clippy::needless_pass_by_value)]
use super::*;
use poentaro::file_tree as tree;

pub(super) fn file_store_create_folder(
    state: FileStoreState,
    parent: Option<String>,
    name: String,
    context: FileOperationContext,
) -> FileStoreResponse {
    let outcome = parse_optional(parent.as_deref()).and_then(|parent| {
        tree::create_folder(&state.into(), parent.as_ref(), &name, &context.into())
    });
    store_response(outcome)
}

pub(super) fn file_store_create_folder_error(
    state: FileStoreState,
    parent: Option<String>,
    name: String,
    context: FileOperationContext,
) -> Option<String> {
    tree::create_folder_error(&state.into(), parent.as_deref(), &name, &context.into())
}

pub(super) fn file_store_create_file(
    state: FileStoreState,
    folder: Option<String>,
    name: String,
    content: String,
    context: FileOperationContext,
) -> FileStoreResponse {
    let outcome = parse_optional(folder.as_deref()).and_then(|folder| {
        tree::create_file(
            &state.into(),
            folder.as_ref(),
            &name,
            &content,
            &context.into(),
        )
    });
    store_response(outcome)
}

pub(super) fn file_store_create_file_error(
    state: FileStoreState,
    folder: Option<String>,
    name: String,
    context: FileOperationContext,
) -> Option<String> {
    tree::create_file_error(&state.into(), folder.as_deref(), &name, &context.into())
}
