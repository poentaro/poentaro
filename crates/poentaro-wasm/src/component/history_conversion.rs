//! Conversions between the WIT outbox-history types and the pure
//! `cloud_outbox::history` types. Conversion only.
use super::*;
use cloud_outbox::history as ch;

impl From<ch::MessageKind> for MessageKind {
    fn from(kind: ch::MessageKind) -> Self {
        match kind {
            ch::MessageKind::Create => MessageKind::Create,
            ch::MessageKind::Edit => MessageKind::Edit,
            ch::MessageKind::Rename => MessageKind::Rename,
            ch::MessageKind::Trash => MessageKind::Trash,
            ch::MessageKind::Restore => MessageKind::Restore,
        }
    }
}

impl From<MessageKind> for ch::MessageKind {
    fn from(kind: MessageKind) -> Self {
        match kind {
            MessageKind::Create => ch::MessageKind::Create,
            MessageKind::Edit => ch::MessageKind::Edit,
            MessageKind::Rename => ch::MessageKind::Rename,
            MessageKind::Trash => ch::MessageKind::Trash,
            MessageKind::Restore => ch::MessageKind::Restore,
        }
    }
}

impl From<ch::HistoryStatus> for HistoryStatus {
    fn from(status: ch::HistoryStatus) -> Self {
        match status {
            ch::HistoryStatus::Pending => HistoryStatus::Pending,
            ch::HistoryStatus::Retrying => HistoryStatus::Retrying,
            ch::HistoryStatus::NeedsMerge => HistoryStatus::NeedsMerge,
            ch::HistoryStatus::NeedsSignIn => HistoryStatus::NeedsSignIn,
            ch::HistoryStatus::Failed => HistoryStatus::Failed,
            ch::HistoryStatus::Sent => HistoryStatus::Sent,
        }
    }
}

impl From<HistoryStatus> for ch::HistoryStatus {
    fn from(status: HistoryStatus) -> Self {
        match status {
            HistoryStatus::Pending => ch::HistoryStatus::Pending,
            HistoryStatus::Retrying => ch::HistoryStatus::Retrying,
            HistoryStatus::NeedsMerge => ch::HistoryStatus::NeedsMerge,
            HistoryStatus::NeedsSignIn => ch::HistoryStatus::NeedsSignIn,
            HistoryStatus::Failed => ch::HistoryStatus::Failed,
            HistoryStatus::Sent => ch::HistoryStatus::Sent,
        }
    }
}

impl From<StatusFilter> for ch::StatusFilter {
    fn from(filter: StatusFilter) -> Self {
        match filter {
            StatusFilter::Any => ch::StatusFilter::Any,
            StatusFilter::Only(status) => ch::StatusFilter::Only(status.into()),
            StatusFilter::Problem => ch::StatusFilter::Problem,
        }
    }
}

impl From<AttemptsFilter> for ch::AttemptsFilter {
    fn from(filter: AttemptsFilter) -> Self {
        match filter {
            AttemptsFilter::Any => ch::AttemptsFilter::Any,
            AttemptsFilter::Untried => ch::AttemptsFilter::None,
            AttemptsFilter::TwoOrMore => ch::AttemptsFilter::TwoOrMore,
        }
    }
}

impl From<HistoryFilter> for ch::HistoryFilter {
    fn from(filter: HistoryFilter) -> Self {
        ch::HistoryFilter {
            query: filter.query,
            status: filter.status.into(),
            kind: filter.kind.map(Into::into),
            file_id: filter.file_id,
            max_age_ms: filter.max_age_ms,
            min_size_bytes: filter.min_size_bytes,
            attempts: filter.attempts.into(),
        }
    }
}

impl From<FileName> for ch::FileName {
    fn from(name: FileName) -> Self {
        ch::FileName {
            file_id: name.file_id,
            name: name.name,
        }
    }
}

impl From<ch::HistoryRow> for HistoryRow {
    fn from(row: ch::HistoryRow) -> Self {
        HistoryRow {
            message_id: row.message_id,
            file_id: row.file_id,
            file_name: row.file_name,
            kind: row.kind.into(),
            status: row.status.into(),
            size_bytes: row.size_bytes,
            at_ms: row.at_ms,
            attempts: row.attempts.into_iter().map(Into::into).collect(),
            retry_in_ms: row.retry_in_ms,
            revision: row.revision,
            payload_json: row.payload_json,
            payload_matches_query: row.payload_matches_query,
        }
    }
}

impl From<ch::HistoryWrite> for HistoryWrite {
    fn from(write: ch::HistoryWrite) -> Self {
        HistoryWrite {
            put: write.put.map(Into::into),
            deletes: write.deletes,
        }
    }
}
