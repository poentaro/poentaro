//! Conversions between the WIT outbox types and the pure `cloud_outbox`
//! types. Conversion only: every decision lives in the `cloud-outbox` crate.
use super::*;
use cloud_outbox::types as co;

impl From<co::Message> for Message {
    fn from(message: co::Message) -> Self {
        match message {
            co::Message::CreateFile { name, content } => {
                Message::CreateFile(CreateFileMessage { name, content })
            }
            co::Message::SaveContent { content } => {
                Message::SaveContent(SaveContentMessage { content })
            }
            co::Message::RenameFile { to } => Message::RenameFile(RenameFileMessage { to }),
            co::Message::RenameFiles { moves } => Message::RenameFiles(RenameFilesMessage {
                moves: moves.into_iter().map(Into::into).collect(),
            }),
            co::Message::TrashFile => Message::TrashFile,
            co::Message::RestoreFile => Message::RestoreFile,
        }
    }
}

impl From<Message> for co::Message {
    fn from(message: Message) -> Self {
        match message {
            Message::CreateFile(CreateFileMessage { name, content }) => {
                co::Message::CreateFile { name, content }
            }
            Message::SaveContent(SaveContentMessage { content }) => {
                co::Message::SaveContent { content }
            }
            Message::RenameFile(RenameFileMessage { to }) => co::Message::RenameFile { to },
            Message::RenameFiles(RenameFilesMessage { moves }) => co::Message::RenameFiles {
                moves: moves.into_iter().map(Into::into).collect(),
            },
            Message::TrashFile => co::Message::TrashFile,
            Message::RestoreFile => co::Message::RestoreFile,
        }
    }
}

impl From<co::RenameMove> for RenameMove {
    fn from(rename: co::RenameMove) -> Self {
        RenameMove {
            file_id: rename.file_id,
            to: rename.to,
        }
    }
}

impl From<RenameMove> for co::RenameMove {
    fn from(rename: RenameMove) -> Self {
        co::RenameMove {
            file_id: rename.file_id,
            to: rename.to,
        }
    }
}

impl From<co::NameAdjustment> for NameAdjustment {
    fn from(adjustment: co::NameAdjustment) -> Self {
        NameAdjustment {
            rejected: adjustment.rejected,
            replacement: adjustment.replacement,
        }
    }
}

impl From<NameAdjustment> for co::NameAdjustment {
    fn from(adjustment: NameAdjustment) -> Self {
        co::NameAdjustment {
            rejected: adjustment.rejected,
            replacement: adjustment.replacement,
        }
    }
}

impl From<co::AttemptOutcome> for AttemptOutcome {
    fn from(outcome: co::AttemptOutcome) -> Self {
        match outcome {
            co::AttemptOutcome::Succeeded => AttemptOutcome::Succeeded,
            co::AttemptOutcome::Transient { reason } => {
                AttemptOutcome::Transient(TransientReason { reason })
            }
            co::AttemptOutcome::Unauthorized => AttemptOutcome::Unauthorized,
            co::AttemptOutcome::RevisionConflict { current_revision } => {
                AttemptOutcome::RevisionConflict(RevisionConflictDetails { current_revision })
            }
            co::AttemptOutcome::NameTaken { suggested_name } => {
                AttemptOutcome::NameTaken(NameTakenDetails { suggested_name })
            }
            co::AttemptOutcome::NamesTaken { adjustments } => {
                AttemptOutcome::NamesTaken(NamesTakenDetails {
                    adjustments: adjustments.into_iter().map(Into::into).collect(),
                })
            }
            co::AttemptOutcome::Permanent { status, reason } => {
                AttemptOutcome::Permanent(PermanentDetails { status, reason })
            }
        }
    }
}

impl From<AttemptOutcome> for co::AttemptOutcome {
    fn from(outcome: AttemptOutcome) -> Self {
        match outcome {
            AttemptOutcome::Succeeded => co::AttemptOutcome::Succeeded,
            AttemptOutcome::Transient(TransientReason { reason }) => {
                co::AttemptOutcome::Transient { reason }
            }
            AttemptOutcome::Unauthorized => co::AttemptOutcome::Unauthorized,
            AttemptOutcome::RevisionConflict(RevisionConflictDetails { current_revision }) => {
                co::AttemptOutcome::RevisionConflict { current_revision }
            }
            AttemptOutcome::NameTaken(NameTakenDetails { suggested_name }) => {
                co::AttemptOutcome::NameTaken { suggested_name }
            }
            AttemptOutcome::NamesTaken(NamesTakenDetails { adjustments }) => {
                co::AttemptOutcome::NamesTaken {
                    adjustments: adjustments.into_iter().map(Into::into).collect(),
                }
            }
            AttemptOutcome::Permanent(PermanentDetails { status, reason }) => {
                co::AttemptOutcome::Permanent { status, reason }
            }
        }
    }
}

impl From<co::Attempt> for Attempt {
    fn from(attempt: co::Attempt) -> Self {
        Attempt {
            at_ms: attempt.at_ms,
            outcome: attempt.outcome.into(),
        }
    }
}

impl From<Attempt> for co::Attempt {
    fn from(attempt: Attempt) -> Self {
        co::Attempt {
            at_ms: attempt.at_ms,
            outcome: attempt.outcome.into(),
        }
    }
}

impl From<co::QueuedMessage> for QueuedMessage {
    fn from(queued: co::QueuedMessage) -> Self {
        QueuedMessage {
            message_id: queued.message_id,
            created_at_ms: queued.created_at_ms,
            message: queued.message.into(),
            attempts: queued.attempts.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<QueuedMessage> for co::QueuedMessage {
    fn from(queued: QueuedMessage) -> Self {
        co::QueuedMessage {
            message_id: queued.message_id,
            created_at_ms: queued.created_at_ms,
            message: queued.message.into(),
            attempts: queued.attempts.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<co::LaneStatus> for LaneStatus {
    fn from(status: co::LaneStatus) -> Self {
        match status {
            co::LaneStatus::Idle => LaneStatus::Idle,
            co::LaneStatus::Draining => LaneStatus::Draining,
            co::LaneStatus::Waiting { retry_at_ms } => {
                LaneStatus::Waiting(WaitingDetails { retry_at_ms })
            }
            co::LaneStatus::NeedsSignIn => LaneStatus::NeedsSignIn,
            co::LaneStatus::NeedsMerge { current_revision } => {
                LaneStatus::NeedsMerge(NeedsMergeDetails { current_revision })
            }
            co::LaneStatus::Failed { reason } => LaneStatus::Failed(FailedDetails { reason }),
        }
    }
}

impl From<LaneStatus> for co::LaneStatus {
    fn from(status: LaneStatus) -> Self {
        match status {
            LaneStatus::Idle => co::LaneStatus::Idle,
            LaneStatus::Draining => co::LaneStatus::Draining,
            LaneStatus::Waiting(WaitingDetails { retry_at_ms }) => {
                co::LaneStatus::Waiting { retry_at_ms }
            }
            LaneStatus::NeedsSignIn => co::LaneStatus::NeedsSignIn,
            LaneStatus::NeedsMerge(NeedsMergeDetails { current_revision }) => {
                co::LaneStatus::NeedsMerge { current_revision }
            }
            LaneStatus::Failed(FailedDetails { reason }) => co::LaneStatus::Failed { reason },
        }
    }
}

impl From<co::Lane> for Lane {
    fn from(lane: co::Lane) -> Self {
        Lane {
            file_id: lane.file_id,
            status: lane.status.into(),
            in_flight: lane.in_flight,
            messages: lane.messages.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Lane> for co::Lane {
    fn from(lane: Lane) -> Self {
        co::Lane {
            file_id: lane.file_id,
            status: lane.status.into(),
            in_flight: lane.in_flight,
            messages: lane.messages.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<co::FileRevision> for FileRevision {
    fn from(entry: co::FileRevision) -> Self {
        FileRevision {
            file_id: entry.file_id,
            revision: entry.revision,
        }
    }
}

impl From<FileRevision> for co::FileRevision {
    fn from(entry: FileRevision) -> Self {
        co::FileRevision {
            file_id: entry.file_id,
            revision: entry.revision,
        }
    }
}

impl From<co::Queue> for Queue {
    fn from(queue: co::Queue) -> Self {
        Queue {
            lanes: queue.lanes.into_iter().map(Into::into).collect(),
            revisions: queue.revisions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Queue> for co::Queue {
    fn from(queue: Queue) -> Self {
        co::Queue {
            lanes: queue.lanes.into_iter().map(Into::into).collect(),
            revisions: queue.revisions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<co::SendRequest> for SendRequest {
    fn from(request: co::SendRequest) -> Self {
        SendRequest {
            file_id: request.file_id,
            message_id: request.message_id,
            message: request.message.into(),
            expected_revision: request.expected_revision,
        }
    }
}

impl From<SendResult> for co::SendResult {
    fn from(result: SendResult) -> Self {
        match result {
            SendResult::Ok(SendOkDetails { revision }) => co::SendResult::Ok { revision },
            SendResult::Transient(TransientReason { reason }) => {
                co::SendResult::Transient { reason }
            }
            SendResult::Unauthorized => co::SendResult::Unauthorized,
            SendResult::Conflict(RevisionConflictDetails { current_revision }) => {
                co::SendResult::Conflict { current_revision }
            }
            SendResult::NameTaken(NameTakenDetails { suggested_name }) => {
                co::SendResult::NameTaken { suggested_name }
            }
            SendResult::NamesTaken(NamesTakenDetails { adjustments }) => {
                co::SendResult::NamesTaken {
                    adjustments: adjustments.into_iter().map(Into::into).collect(),
                }
            }
            SendResult::Permanent(PermanentDetails { status, reason }) => {
                co::SendResult::Permanent { status, reason }
            }
        }
    }
}

impl From<Resolution> for co::Resolution {
    fn from(resolution: Resolution) -> Self {
        match resolution {
            Resolution::Retry => co::Resolution::Retry,
            Resolution::Discard => co::Resolution::Discard,
            Resolution::MergedAndSave(MergedAndSaveDetails {
                content,
                server_revision,
            }) => co::Resolution::MergedAndSave {
                content,
                server_revision,
            },
        }
    }
}

impl From<co::StoredRecord> for StoredRecord {
    fn from(record: co::StoredRecord) -> Self {
        StoredRecord {
            key: record.key,
            value: record.value,
        }
    }
}

impl From<StoredRecord> for co::StoredRecord {
    fn from(record: StoredRecord) -> Self {
        co::StoredRecord {
            key: record.key,
            value: record.value,
        }
    }
}

impl From<co::DecodeError> for DecodeError {
    fn from(error: co::DecodeError) -> Self {
        match error {
            co::DecodeError::UnsupportedVersion { found } => {
                DecodeError::UnsupportedVersion(UnsupportedVersionDetails { found })
            }
            co::DecodeError::Corrupt { key } => DecodeError::Corrupt(CorruptDetails { key }),
        }
    }
}

impl From<co::MergeOutcome> for MergeOutcome {
    fn from(outcome: co::MergeOutcome) -> Self {
        match outcome {
            co::MergeOutcome::Clean { text } => MergeOutcome::Clean(MergeCleanDetails { text }),
            co::MergeOutcome::Conflicted { text, has_markers } => {
                MergeOutcome::Conflicted(MergeConflictedDetails { text, has_markers })
            }
        }
    }
}

impl From<co::LaneReason> for LaneReason {
    fn from(reason: co::LaneReason) -> Self {
        match reason {
            co::LaneReason::Syncing => LaneReason::Syncing,
            co::LaneReason::WaitingToRetry => LaneReason::WaitingToRetry,
            co::LaneReason::PossiblyStuck => LaneReason::PossiblyStuck,
            co::LaneReason::NeedsSignIn => LaneReason::NeedsSignIn,
            co::LaneReason::NeedsMerge => LaneReason::NeedsMerge,
            co::LaneReason::Failed => LaneReason::Failed,
        }
    }
}

impl From<co::LaneSummary> for LaneSummary {
    fn from(lane: co::LaneSummary) -> Self {
        LaneSummary {
            file_id: lane.file_id,
            reason: lane.reason.into(),
            pending_messages: lane.pending_messages,
            retry_in_ms: lane.retry_in_ms,
            attempts: lane.attempts.into_iter().map(Into::into).collect(),
            failure_reason: lane.failure_reason,
        }
    }
}

impl From<co::QueueSummary> for QueueSummary {
    fn from(summary: co::QueueSummary) -> Self {
        QueueSummary {
            lanes: summary.lanes.into_iter().map(Into::into).collect(),
            total_pending: summary.total_pending,
            lanes_needing_attention: summary.lanes_needing_attention,
            has_unsynced: summary.has_unsynced,
        }
    }
}
