//! Internal values shared by the worker service and persistence layers.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncedDoc {
    pub ended: bool,
    pub filename: String,
    pub content: String,
    pub revision: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<SharedFolder>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedFolder {
    /// The shared folder's own name (last segment of its path).
    pub name: String,
    /// Every active file, in path order, `.keep` placeholders excluded.
    pub files: Vec<SharedFolderFile>,
    /// Relative folders that exist only as an empty folder (a `.keep`
    /// placeholder, no file under them). Folders implied by `files` paths
    /// are not repeated here.
    pub empty_folders: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedFolderFile {
    /// Path relative to the shared folder, e.g. `Advent/O Come.poentaro`.
    pub path: String,
    pub content: String,
    pub revision: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareStatus {
    pub share_id: String,
    pub ended: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileShareEntry {
    pub share_id: String,
    pub file_id: String,
    /// The file's current path.
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderShareEntry {
    pub share_id: String,
    /// The folder's current path, no trailing slash.
    pub folder: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileVersionSummary {
    pub revision: i64,
    /// When this version was saved (epoch millis).
    pub saved_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameBatchMove {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    pub session_token: String,
    pub email: String,
}
