//! Bodies of the cloud-outbox `Guest` methods: convert WIT values to
//! `cloud_outbox` types, call the crate, convert back. No logic beyond
//! conversion, plus the one parser-backed `accepts` closure for the merge.
#![allow(clippy::needless_pass_by_value)]
use super::*;
use cloud_outbox::{codec, enqueue, history, merge, scheduling, summary, transitions};

pub(super) fn outbox_enqueue(
    queue: Queue,
    file_id: String,
    message: Message,
    message_id: String,
    now_ms: u64,
) -> Queue {
    enqueue::enqueue(queue.into(), &file_id, message.into(), &message_id, now_ms).into()
}

pub(super) fn outbox_begin_send(queue: Queue, now_ms: u64) -> BeginSend {
    let begun = scheduling::begin_send(queue.into(), now_ms);
    BeginSend {
        queue: begun.queue.into(),
        request: begun.request.map(Into::into),
    }
}

pub(super) fn outbox_next_wake_ms(queue: Queue) -> Option<u64> {
    scheduling::next_wake_ms(&queue.into())
}

pub(super) fn outbox_on_result(
    queue: Queue,
    file_id: String,
    message_id: String,
    send_result: SendResult,
    now_ms: u64,
    jitter_unit: f64,
) -> Queue {
    transitions::on_result(
        queue.into(),
        &file_id,
        &message_id,
        send_result.into(),
        now_ms,
        jitter_unit,
    )
    .into()
}

pub(super) fn outbox_resolve(queue: Queue, file_id: String, resolution: Resolution) -> Queue {
    transitions::resolve(queue.into(), &file_id, resolution.into()).into()
}

pub(super) fn outbox_resolve_signed_in(queue: Queue) -> Queue {
    transitions::resolve_signed_in(queue.into()).into()
}

pub(super) fn outbox_record_revision(queue: Queue, file_id: String, revision: i64) -> Queue {
    transitions::record_revision(queue.into(), &file_id, revision).into()
}

pub(super) fn outbox_encode_queue(queue: Queue) -> Vec<StoredRecord> {
    codec::encode_queue(&queue.into())
        .into_iter()
        .map(Into::into)
        .collect()
}

pub(super) fn outbox_decode_queue(records: Vec<StoredRecord>) -> Result<Queue, DecodeError> {
    let records: Vec<_> = records.into_iter().map(Into::into).collect();
    codec::decode_queue(&records)
        .map(Into::into)
        .map_err(Into::into)
}

pub(super) fn outbox_summarize(queue: Queue, now_ms: u64) -> QueueSummary {
    summary::summarize(&queue.into(), now_ms).into()
}

pub(super) fn outbox_record_sent(
    queue_before: Queue,
    existing: Vec<StoredRecord>,
    file_id: String,
    message_id: String,
    revision: i64,
    now_ms: u64,
) -> HistoryWrite {
    let existing: Vec<_> = existing.into_iter().map(Into::into).collect();
    history::record_sent(
        &queue_before.into(),
        &existing,
        &file_id,
        &message_id,
        revision,
        now_ms,
    )
    .into()
}

pub(super) fn outbox_history_rows(
    queue: Queue,
    sent_records: Vec<StoredRecord>,
    file_names: Vec<FileName>,
    filter: HistoryFilter,
    now_ms: u64,
) -> Vec<HistoryRow> {
    let sent_records: Vec<_> = sent_records.into_iter().map(Into::into).collect();
    let file_names: Vec<_> = file_names.into_iter().map(Into::into).collect();
    history::rows(
        &queue.into(),
        &sent_records,
        &file_names,
        &filter.into(),
        now_ms,
    )
    .into_iter()
    .map(Into::into)
    .collect()
}

pub(super) fn merge_three_way(base: String, mine: String, theirs: String) -> MergeOutcome {
    merge::merge_three_way(&base, &mine, &theirs, |text| {
        poentaro::document::anyx_errors(text, &[]).is_empty()
            && poentaro::parser::parse(text, "merge.poentaro", &[]).is_ok()
    })
    .into()
}
