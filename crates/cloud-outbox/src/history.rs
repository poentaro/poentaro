//! The outbox history: messages already delivered (kept for a while, then
//! pruned) merged with the messages still queued, filtered for display.

use crate::types::{
    Attempt, AttemptOutcome, Lane, LaneStatus, Message, Queue, QueuedMessage, StoredRecord,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

pub const SENT_KEY_PREFIX: &str = "sent:";
pub const RETENTION_MS: u64 = 7 * 24 * 60 * 60 * 1000;
pub const MAX_SENT_ENTRIES: usize = 200;
pub const MAX_SENT_BYTES: usize = 5 * 1024 * 1024;

/// A delivered message as persisted under `sent:<sent_at_ms>:<message_id>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentEntry {
    pub message_id: String,
    pub file_id: String,
    pub created_at_ms: u64,
    pub sent_at_ms: u64,
    pub revision: i64,
    pub message: Message,
    /// Every try, ending with the one that succeeded.
    pub attempts: Vec<Attempt>,
}

/// What to write to the store after a delivery: the new record and the keys
/// that retention removes.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryWrite {
    pub put: Option<StoredRecord>,
    pub deletes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Create,
    Edit,
    Rename,
    Trash,
    Restore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryStatus {
    Pending,
    Retrying,
    NeedsMerge,
    NeedsSignIn,
    Failed,
    Sent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusFilter {
    Any,
    Only(HistoryStatus),
    /// Retrying, needs merge, needs sign-in or failed.
    Problem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptsFilter {
    Any,
    None,
    TwoOrMore,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistoryFilter {
    /// Case-insensitive; matched against the file name and the payload text.
    pub query: String,
    pub status: StatusFilter,
    pub kind: Option<MessageKind>,
    pub file_id: Option<String>,
    pub max_age_ms: Option<u64>,
    pub min_size_bytes: Option<u32>,
    pub attempts: AttemptsFilter,
}

impl HistoryFilter {
    pub fn everything() -> Self {
        HistoryFilter {
            query: String::new(),
            status: StatusFilter::Any,
            kind: None,
            file_id: None,
            max_age_ms: None,
            min_size_bytes: None,
            attempts: AttemptsFilter::Any,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileName {
    pub file_id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistoryRow {
    pub message_id: String,
    pub file_id: String,
    /// The file's name, or its id when the name is unknown.
    pub file_name: String,
    pub kind: MessageKind,
    pub status: HistoryStatus,
    pub size_bytes: u32,
    /// When the message was sent, or created while it is still queued.
    pub at_ms: u64,
    pub attempts: Vec<Attempt>,
    pub retry_in_ms: Option<u64>,
    /// The server revision a sent message produced.
    pub revision: Option<i64>,
    /// The full message as pretty-printed JSON.
    pub payload_json: String,
    /// The query is found in the payload text (not only in the file name).
    pub payload_matches_query: bool,
}

fn sent_key(sent_at_ms: u64, message_id: &str) -> String {
    format!("{SENT_KEY_PREFIX}{sent_at_ms:020}:{message_id}")
}

fn sent_at_of_key(key: &str) -> Option<u64> {
    key.strip_prefix(SENT_KEY_PREFIX)?
        .split(':')
        .next()?
        .parse()
        .ok()
}

fn queued_message<'a>(
    queue: &'a Queue,
    file_id: &str,
    message_id: &str,
) -> Option<&'a QueuedMessage> {
    queue
        .lanes
        .iter()
        .filter(|lane| lane.file_id == file_id)
        .flat_map(|lane| &lane.messages)
        .find(|queued| queued.message_id == message_id)
}

/// Records a delivery. `queue_before` is the queue as it was before the
/// result was applied (the message is still in it); `existing` is every
/// stored `sent:` record. Retention is applied to the existing records plus
/// the new one.
pub fn record_sent(
    queue_before: &Queue,
    existing: &[StoredRecord],
    file_id: &str,
    message_id: &str,
    revision: i64,
    now_ms: u64,
) -> HistoryWrite {
    let put = queued_message(queue_before, file_id, message_id).and_then(|queued| {
        let entry = SentEntry {
            message_id: message_id.to_string(),
            file_id: file_id.to_string(),
            created_at_ms: queued.created_at_ms,
            sent_at_ms: now_ms,
            revision,
            message: queued.message.clone(),
            attempts: queued
                .attempts
                .iter()
                .cloned()
                .chain(std::iter::once(Attempt {
                    at_ms: now_ms,
                    outcome: AttemptOutcome::Succeeded,
                }))
                .collect(),
        };
        serde_json::to_string(&entry)
            .ok()
            .map(|value| StoredRecord {
                key: sent_key(now_ms, message_id),
                value,
            })
    });
    let all = existing.iter().cloned().chain(put.clone()).collect_vec();
    let deletes = expired_keys(&all, now_ms);
    HistoryWrite { put, deletes }
}

/// Keys to delete so the kept records are recent enough, few enough and small
/// enough. The newest record is always kept.
pub fn expired_keys(records: &[StoredRecord], now_ms: u64) -> Vec<String> {
    let newest_first = records
        .iter()
        .filter(|record| record.key.starts_with(SENT_KEY_PREFIX))
        .sorted_by(|a, b| b.key.cmp(&a.key))
        .collect_vec();
    let mut kept_bytes = 0usize;
    newest_first
        .iter()
        .enumerate()
        .filter_map(|(index, record)| {
            let too_old = sent_at_of_key(&record.key)
                .is_none_or(|sent_at| now_ms.saturating_sub(sent_at) > RETENTION_MS);
            kept_bytes += record.value.len();
            let over_budget =
                index > 0 && (index >= MAX_SENT_ENTRIES || kept_bytes > MAX_SENT_BYTES);
            (too_old || over_budget).then(|| record.key.clone())
        })
        .collect()
}

fn kind_of(message: &Message) -> MessageKind {
    match message {
        Message::CreateFile { .. } => MessageKind::Create,
        Message::SaveContent { .. } => MessageKind::Edit,
        Message::RenameFile { .. } | Message::RenameFiles { .. } => MessageKind::Rename,
        Message::TrashFile => MessageKind::Trash,
        Message::RestoreFile => MessageKind::Restore,
    }
}

fn size_of(message: &Message) -> u32 {
    let bytes = match message {
        Message::CreateFile { name, content } => name.len() + content.len(),
        Message::SaveContent { content } => content.len(),
        Message::RenameFile { to } => to.len(),
        Message::RenameFiles { moves, .. } => moves.iter().map(|rename| rename.to.len()).sum(),
        Message::TrashFile | Message::RestoreFile => 0,
    };
    u32::try_from(bytes).unwrap_or(u32::MAX)
}

fn payload_json(file_id: &str, message_id: &str, message: &Message) -> String {
    let mut value = serde_json::to_value(message).unwrap_or(serde_json::Value::Null);
    if let Some(object) = value.as_object_mut() {
        object.insert("fileId".into(), file_id.into());
        object.insert("messageId".into(), message_id.into());
    }
    serde_json::to_string_pretty(&value).unwrap_or_default()
}

fn pending_status(lane: &Lane, is_head: bool) -> HistoryStatus {
    match (&lane.status, is_head) {
        (LaneStatus::Waiting { .. }, true) => HistoryStatus::Retrying,
        (LaneStatus::NeedsMerge { .. }, true) => HistoryStatus::NeedsMerge,
        (LaneStatus::NeedsSignIn, true) => HistoryStatus::NeedsSignIn,
        (LaneStatus::Failed { .. }, true) => HistoryStatus::Failed,
        _ => HistoryStatus::Pending,
    }
}

fn pending_rows(queue: &Queue, now_ms: u64) -> Vec<RawRow> {
    queue
        .lanes
        .iter()
        .flat_map(|lane| {
            lane.messages
                .iter()
                .enumerate()
                .map(move |(index, queued)| {
                    let retry_in_ms = match (&lane.status, index) {
                        (LaneStatus::Waiting { retry_at_ms }, 0) => {
                            Some(retry_at_ms.saturating_sub(now_ms))
                        }
                        _ => None,
                    };
                    RawRow {
                        message_id: queued.message_id.clone(),
                        file_id: lane.file_id.clone(),
                        message: queued.message.clone(),
                        status: pending_status(lane, index == 0),
                        at_ms: queued.created_at_ms,
                        attempts: queued.attempts.clone(),
                        retry_in_ms,
                        revision: None,
                    }
                })
        })
        .collect()
}

fn sent_rows(records: &[StoredRecord]) -> Vec<RawRow> {
    records
        .iter()
        .filter(|record| record.key.starts_with(SENT_KEY_PREFIX))
        .filter_map(|record| serde_json::from_str::<SentEntry>(&record.value).ok())
        // Records from before the log ended in `Succeeded` are hidden;
        // retention deletes them by age.
        .filter(|entry| {
            entry
                .attempts
                .last()
                .is_some_and(|attempt| attempt.outcome == AttemptOutcome::Succeeded)
        })
        .map(|entry| RawRow {
            message_id: entry.message_id,
            file_id: entry.file_id,
            message: entry.message,
            status: HistoryStatus::Sent,
            at_ms: entry.sent_at_ms,
            attempts: entry.attempts,
            retry_in_ms: None,
            revision: Some(entry.revision),
        })
        .collect()
}

struct RawRow {
    message_id: String,
    file_id: String,
    message: Message,
    status: HistoryStatus,
    at_ms: u64,
    attempts: Vec<Attempt>,
    retry_in_ms: Option<u64>,
    revision: Option<i64>,
}

fn status_matches(filter: StatusFilter, status: HistoryStatus) -> bool {
    match filter {
        StatusFilter::Any => true,
        StatusFilter::Only(wanted) => wanted == status,
        StatusFilter::Problem => matches!(
            status,
            HistoryStatus::Retrying
                | HistoryStatus::NeedsMerge
                | HistoryStatus::NeedsSignIn
                | HistoryStatus::Failed
        ),
    }
}

fn attempts_match(filter: AttemptsFilter, count: usize) -> bool {
    match filter {
        AttemptsFilter::Any => true,
        AttemptsFilter::None => count == 0,
        AttemptsFilter::TwoOrMore => count >= 2,
    }
}

/// Queued and delivered messages that pass the filter, newest first.
pub fn rows(
    queue: &Queue,
    sent_records: &[StoredRecord],
    file_names: &[FileName],
    filter: &HistoryFilter,
    now_ms: u64,
) -> Vec<HistoryRow> {
    let query = filter.query.to_lowercase();
    pending_rows(queue, now_ms)
        .into_iter()
        .chain(sent_rows(sent_records))
        .filter_map(|raw| {
            let file_name = file_names
                .iter()
                .find(|candidate| candidate.file_id == raw.file_id)
                .map_or_else(|| raw.file_id.clone(), |candidate| candidate.name.clone());
            let payload = payload_json(&raw.file_id, &raw.message_id, &raw.message);
            let in_payload = !query.is_empty() && payload.to_lowercase().contains(&query);
            let in_name = file_name.to_lowercase().contains(&query);
            let size_bytes = size_of(&raw.message);
            let kind = kind_of(&raw.message);
            let passes = (query.is_empty() || in_name || in_payload)
                && status_matches(filter.status, raw.status)
                && filter.kind.is_none_or(|wanted| wanted == kind)
                && filter
                    .file_id
                    .as_ref()
                    .is_none_or(|wanted| *wanted == raw.file_id)
                && filter
                    .max_age_ms
                    .is_none_or(|max_age| now_ms.saturating_sub(raw.at_ms) <= max_age)
                && filter.min_size_bytes.is_none_or(|min| size_bytes >= min)
                && attempts_match(filter.attempts, raw.attempts.len());
            if !passes {
                return None;
            }
            Some(HistoryRow {
                message_id: raw.message_id,
                file_id: raw.file_id,
                file_name,
                kind,
                status: raw.status,
                size_bytes,
                at_ms: raw.at_ms,
                attempts: raw.attempts,
                retry_in_ms: raw.retry_in_ms,
                revision: raw.revision,
                payload_json: payload,
                payload_matches_query: in_payload,
            })
        })
        .sorted_by(|a, b| b.at_ms.cmp(&a.at_ms))
        .collect()
}
