//! Data types shared by every outbox module.

use serde::{Deserialize, Serialize};

pub const QUEUE_SCHEMA_VERSION: u32 = 1;
pub const POSSIBLY_STUCK_AFTER: u32 = 10;
/// Lane id prefix of a batch message: it covers several files, so it gets a
/// lane of its own, `batch:<message id>`, instead of a file's lane.
pub const BATCH_LANE_PREFIX: &str = "batch:";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Queue {
    pub lanes: Vec<Lane>,
    pub revisions: Vec<FileRevision>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRevision {
    pub file_id: String,
    pub revision: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lane {
    pub file_id: String,
    pub status: LaneStatus,
    /// The message id currently being sent.
    pub in_flight: Option<String>,
    /// In order; the head is index 0.
    pub messages: Vec<QueuedMessage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuedMessage {
    pub message_id: String,
    pub created_at_ms: u64,
    pub message: Message,
    pub attempts: Vec<Attempt>,
}

/// One file's new path inside a `RenameFiles` batch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameMove {
    pub file_id: String,
    pub to: String,
}

/// The folder a `RenameFiles` batch moves, delivered as one folder move the
/// server plans itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FolderMove {
    pub from: String,
    pub to: String,
}

/// The file id lives on the lane, except for `RenameFiles`, which names its
/// files itself and lives on a `batch:` lane.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Message {
    CreateFile {
        name: String,
        content: String,
    },
    SaveContent {
        content: String,
    },
    RenameFile {
        to: String,
    },
    /// Renames every listed file, or none of them (one folder move).
    /// `moves` orders the batch against the files' other messages. A batch
    /// with `folder` is sent as that folder move; one queued before folders
    /// were recorded (or adjusted after `NamesTaken`) moves each file.
    RenameFiles {
        moves: Vec<RenameMove>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        folder: Option<FolderMove>,
    },
    TrashFile,
    RestoreFile,
}

/// How a rejected destination of a `RenameFiles` batch is changed: to
/// `replacement`, or (`None`) the move is dropped because the destination
/// is a folder marker that already exists.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NameAdjustment {
    pub rejected: String,
    pub replacement: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attempt {
    pub at_ms: u64,
    pub outcome: AttemptOutcome,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AttemptOutcome {
    /// The try that delivered the message: the last entry of a sent
    /// message's log.
    Succeeded,
    Transient {
        reason: String,
    },
    Unauthorized,
    #[serde(rename_all = "camelCase")]
    RevisionConflict {
        current_revision: i64,
    },
    #[serde(rename_all = "camelCase")]
    NameTaken {
        suggested_name: String,
    },
    #[serde(rename_all = "camelCase")]
    NamesTaken {
        adjustments: Vec<NameAdjustment>,
    },
    Permanent {
        status: u16,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LaneStatus {
    Idle,
    Draining,
    #[serde(rename_all = "camelCase")]
    Waiting {
        retry_at_ms: u64,
    },
    NeedsSignIn,
    #[serde(rename_all = "camelCase")]
    NeedsMerge {
        current_revision: i64,
    },
    Failed {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendRequest {
    pub file_id: String,
    pub message_id: String,
    pub message: Message,
    /// 0 if no revision recorded.
    pub expected_revision: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SendResult {
    Ok {
        revision: i64,
    },
    Transient {
        reason: String,
    },
    Unauthorized,
    #[serde(rename_all = "camelCase")]
    Conflict {
        current_revision: i64,
    },
    #[serde(rename_all = "camelCase")]
    NameTaken {
        suggested_name: String,
    },
    /// A `RenameFiles` batch was refused as a whole (nothing applied);
    /// these destinations were taken and are adjusted before the resend.
    #[serde(rename_all = "camelCase")]
    NamesTaken {
        adjustments: Vec<NameAdjustment>,
    },
    Permanent {
        status: u16,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Resolution {
    /// Waiting, NeedsMerge, Failed -> Draining.
    Retry,
    /// Drop the whole lane and its recorded revision.
    Discard,
    #[serde(rename_all = "camelCase")]
    MergedAndSave {
        content: String,
        server_revision: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredRecord {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum DecodeError {
    UnsupportedVersion { found: u32 },
    Corrupt { key: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MergeOutcome {
    Clean {
        text: String,
    },
    #[serde(rename_all = "camelCase")]
    Conflicted {
        text: String,
        has_markers: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LaneReason {
    Syncing,
    WaitingToRetry,
    PossiblyStuck,
    NeedsSignIn,
    NeedsMerge,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaneSummary {
    pub file_id: String,
    pub reason: LaneReason,
    pub pending_messages: u32,
    pub retry_in_ms: Option<u64>,
    /// The head message's attempts.
    pub attempts: Vec<Attempt>,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueSummary {
    /// Lanes with at least one message.
    pub lanes: Vec<LaneSummary>,
    pub total_pending: u32,
    /// NeedsSignIn + NeedsMerge + Failed + PossiblyStuck.
    pub lanes_needing_attention: u32,
    /// Any lane has messages, halted or not.
    pub has_unsynced: bool,
}
