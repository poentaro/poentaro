//! Pure, D1-free view logic for a Synced Share. A share (see
//! `live-share-worker/migrations/0004_shares_reference_files.sql`) is a
//! pointer to a cloud `files` row, not a copy of its content: the viewer
//! reads the file itself, so the only write path is the owner's ordinary
//! autosave (`POST /files/{id}/content`). Ownership is enforced by the
//! owner-scoped SQL in `queries/`, the same way the `/files/*` routes work,
//! so nothing here needs to compare identities.

use crate::protocol::SyncedDoc;

/// What `queries/get_share_view.sql` returns for a share id: the share's
/// own state plus the fields of the file it points at. Never carries the
/// file's `id` or `owner_user_id` -- the query doesn't select them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareView {
    /// `None` while the share is live; set when the owner pressed "Stop
    /// Sync".
    pub ended_at: Option<i64>,
    /// The pointed-at file's `trashed_at` -- a binned file reads as an
    /// ended share until it's restored.
    pub trashed_at: Option<i64>,
    pub filename: String,
    pub content: String,
    pub revision: i64,
}

/// A share id with no `shares` row reads the same as one that was shared
/// and then stopped -- both are "nothing to show a viewer".
pub fn empty_doc() -> SyncedDoc {
    SyncedDoc {
        ended: true,
        filename: String::new(),
        content: String::new(),
        revision: 0,
        folder: None,
    }
}

/// The last segment of the file's path (`Hymns/Advent/O Come.poentaro` ->
/// `O Come.poentaro`): a viewer of one shared file gets its name for the title
/// and download, never the owner's folder names.
fn file_name_only(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

/// Builds the public wire shape for `GET /shares/{share_id}`. An ended share
/// (stopped, or its file is in the bin) returns empty `filename`/`content`,
/// so a stopped link can never leak what the file says now -- only `ended`
/// survives.
pub fn to_public_doc(view: Option<&ShareView>) -> SyncedDoc {
    match view {
        None => empty_doc(),
        Some(view) if view.ended_at.is_some() || view.trashed_at.is_some() => empty_doc(),
        Some(view) => SyncedDoc {
            ended: false,
            filename: file_name_only(&view.filename),
            content: view.content.clone(),
            revision: view.revision,
            folder: None,
        },
    }
}
