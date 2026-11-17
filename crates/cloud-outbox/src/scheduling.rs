//! Choosing the next message to send and when to wake.

use crate::types::{Lane, LaneStatus, Message, Queue, QueuedMessage, RenameMove, SendRequest};

/// The queue after choosing a message, and the request to perform (if any).
#[derive(Debug, Clone, PartialEq)]
pub struct BeginSend {
    pub queue: Queue,
    pub request: Option<SendRequest>,
}

/// The moves of a lane's `RenameFiles` messages.
fn batch_moves(lane: &Lane) -> impl Iterator<Item = (&QueuedMessage, &[RenameMove])> {
    lane.messages
        .iter()
        .filter_map(|queued| match &queued.message {
            Message::RenameFiles { moves, .. } => Some((queued, moves.as_slice())),
            _ => None,
        })
}

fn moves_file(moves: &[RenameMove], file_id: &str) -> bool {
    moves.iter().any(|rename| rename.file_id == file_id)
}

fn is_structural(message: &Message) -> bool {
    matches!(
        message,
        Message::RenameFile { .. } | Message::TrashFile | Message::RestoreFile
    )
}

/// Whether the head of `lane` must wait for other work, so that a batch
/// rename never overtakes the file messages it depends on:
///
/// - A `RenameFiles` batch waits for every pending `CreateFile` of its files
///   (the rows must exist), for every per-file rename, trash or restore of
///   its files made strictly before it, and for an earlier batch sharing a
///   file (ties between batches break on message id).
/// - A per-file rename, trash or restore waits for a batch of its file made
///   at the same time or before it.
///
/// Content saves depend on neither: a file id is stable across renames. The
/// two directions use complementary comparisons, so they never wait on each
/// other.
fn is_blocked(queue: &Queue, lane: &Lane) -> bool {
    let Some(head) = lane.messages.first() else {
        return false;
    };
    match &head.message {
        Message::RenameFiles { moves, .. } => queue
            .lanes
            .iter()
            .filter(|other| other.file_id != lane.file_id)
            .any(|other| {
                let earlier_batch = batch_moves(other).any(|(queued, other_moves)| {
                    (queued.created_at_ms, &queued.message_id)
                        < (head.created_at_ms, &head.message_id)
                        && other_moves
                            .iter()
                            .any(|rename| moves_file(moves, &rename.file_id))
                });
                let pending_file_work = moves_file(moves, &other.file_id)
                    && other.messages.iter().any(|queued| {
                        matches!(queued.message, Message::CreateFile { .. })
                            || (is_structural(&queued.message)
                                && queued.created_at_ms < head.created_at_ms)
                    });
                earlier_batch || pending_file_work
            }),
        message if is_structural(message) => queue.lanes.iter().any(|other| {
            batch_moves(other).any(|(queued, moves)| {
                queued.created_at_ms <= head.created_at_ms && moves_file(moves, &lane.file_id)
            })
        }),
        _ => false,
    }
}

fn is_eligible(queue: &Queue, lane: &Lane, now_ms: u64) -> bool {
    !lane.messages.is_empty()
        && lane.in_flight.is_none()
        && !is_blocked(queue, lane)
        && match lane.status {
            LaneStatus::Idle | LaneStatus::Draining => true,
            LaneStatus::Waiting { retry_at_ms } => retry_at_ms <= now_ms,
            LaneStatus::NeedsSignIn | LaneStatus::NeedsMerge { .. } | LaneStatus::Failed { .. } => {
                false
            }
        }
}

/// Picks the eligible lane whose head is oldest and marks that head in flight.
pub fn begin_send(mut queue: Queue, now_ms: u64) -> BeginSend {
    let chosen = queue
        .lanes
        .iter()
        .enumerate()
        .filter(|(_, lane)| is_eligible(&queue, lane, now_ms))
        .min_by_key(|(_, lane)| lane.messages.first().map(|head| head.created_at_ms))
        .map(|(index, _)| index);
    let Some(index) = chosen else {
        return BeginSend {
            queue,
            request: None,
        };
    };
    let revisions = queue.revisions.clone();
    let Some(lane) = queue.lanes.get_mut(index) else {
        return BeginSend {
            queue,
            request: None,
        };
    };
    let expected_revision = revisions
        .iter()
        .find(|entry| entry.file_id == lane.file_id)
        .map_or(0, |entry| entry.revision);
    let request = lane.messages.first().map(|head| SendRequest {
        file_id: lane.file_id.clone(),
        message_id: head.message_id.clone(),
        message: head.message.clone(),
        expected_revision,
    });
    lane.in_flight = request.as_ref().map(|sent| sent.message_id.clone());
    lane.status = LaneStatus::Draining;
    BeginSend { queue, request }
}

/// The earliest retry time among `Waiting` lanes, if any.
pub fn next_wake_ms(queue: &Queue) -> Option<u64> {
    queue
        .lanes
        .iter()
        .filter_map(|lane| match lane.status {
            LaneStatus::Waiting { retry_at_ms } => Some(retry_at_ms),
            _ => None,
        })
        .min()
}
