//! State transitions applied on send results and user resolutions.

use crate::backoff::backoff_ms;
use crate::batch_adjust::adjust_batch;
use crate::types::{
    Attempt, AttemptOutcome, FileRevision, Lane, LaneStatus, Message, NameAdjustment, Queue,
    QueuedMessage, Resolution, SendResult, BATCH_LANE_PREFIX,
};

fn with_lane(queue: Queue, file_id: &str, update: impl FnOnce(Lane) -> Lane) -> Queue {
    let mut update = Some(update);
    Queue {
        lanes: queue
            .lanes
            .into_iter()
            .map(|lane| match update.take() {
                Some(update) if lane.file_id == file_id => update(lane),
                other => {
                    update = other;
                    lane
                }
            })
            .collect(),
        revisions: queue.revisions,
    }
}

fn set_revision(revisions: Vec<FileRevision>, file_id: &str, revision: i64) -> Vec<FileRevision> {
    let mut remaining: Vec<FileRevision> = revisions
        .into_iter()
        .filter(|entry| entry.file_id != file_id)
        .collect();
    remaining.push(FileRevision {
        file_id: file_id.to_string(),
        revision,
    });
    remaining
}

/// Record `outcome` on the head message and clear the in-flight marker.
fn halt_with_attempt(
    lane: Lane,
    now_ms: u64,
    outcome: AttemptOutcome,
    status: impl FnOnce(&Lane) -> LaneStatus,
) -> Lane {
    let mut lane = lane;
    if let Some(head) = lane.messages.first_mut() {
        head.attempts.push(Attempt {
            at_ms: now_ms,
            outcome,
        });
    }
    let status = status(&lane);
    Lane {
        in_flight: None,
        status,
        ..lane
    }
}

fn trailing_transient_count(message: &QueuedMessage) -> u32 {
    message
        .attempts
        .iter()
        .rev()
        .take_while(|attempt| matches!(attempt.outcome, AttemptOutcome::Transient { .. }))
        .count()
        .try_into()
        .unwrap_or(u32::MAX)
}

fn rename_head(lane: &mut Lane, suggested_name: &str) -> bool {
    match lane.messages.first_mut().map(|head| &mut head.message) {
        Some(Message::CreateFile { name, .. }) => {
            *name = suggested_name.to_string();
            true
        }
        Some(Message::RenameFile { to }) => {
            *to = suggested_name.to_string();
            true
        }
        _ => false,
    }
}

/// A batch refused as a whole is resent with the rejected destinations
/// adjusted: all of it or none of it is ever applied. A batch left with no
/// move is done.
fn apply_names_taken(
    lane: Lane,
    adjustments: Vec<NameAdjustment>,
    now_ms: u64,
    jitter_unit: f64,
) -> Lane {
    let mut lane = lane;
    let adjusted = lane
        .messages
        .first()
        .and_then(|head| adjust_batch(&head.message, &adjustments));
    match adjusted {
        Some(Message::RenameFiles { moves, .. }) if moves.is_empty() => {
            apply_result(lane, SendResult::Ok { revision: 0 }, now_ms, jitter_unit)
        }
        Some(message) => {
            if let Some(head) = lane.messages.first_mut() {
                head.message = message;
            }
            halt_with_attempt(
                lane,
                now_ms,
                AttemptOutcome::NamesTaken { adjustments },
                |_| LaneStatus::Draining,
            )
        }
        None => apply_result(
            lane,
            SendResult::Permanent {
                status: 409,
                reason: "name_taken".to_string(),
            },
            now_ms,
            jitter_unit,
        ),
    }
}

fn apply_result(lane: Lane, result: SendResult, now_ms: u64, jitter_unit: f64) -> Lane {
    match result {
        SendResult::Ok { .. } => {
            let mut lane = lane;
            if !lane.messages.is_empty() {
                lane.messages.remove(0);
            }
            let status = if lane.messages.is_empty() {
                LaneStatus::Idle
            } else {
                LaneStatus::Draining
            };
            Lane {
                in_flight: None,
                status,
                ..lane
            }
        }
        SendResult::Transient { reason } => {
            halt_with_attempt(lane, now_ms, AttemptOutcome::Transient { reason }, |lane| {
                let failures = lane.messages.first().map_or(1, trailing_transient_count);
                LaneStatus::Waiting {
                    retry_at_ms: now_ms.saturating_add(backoff_ms(failures, jitter_unit)),
                }
            })
        }
        SendResult::Unauthorized => {
            halt_with_attempt(lane, now_ms, AttemptOutcome::Unauthorized, |_| {
                LaneStatus::NeedsSignIn
            })
        }
        SendResult::Conflict { current_revision } => halt_with_attempt(
            lane,
            now_ms,
            AttemptOutcome::RevisionConflict { current_revision },
            |_| LaneStatus::NeedsMerge { current_revision },
        ),
        SendResult::NameTaken { suggested_name } => {
            let mut lane = lane;
            if rename_head(&mut lane, &suggested_name) {
                halt_with_attempt(
                    lane,
                    now_ms,
                    AttemptOutcome::NameTaken { suggested_name },
                    |_| LaneStatus::Draining,
                )
            } else {
                apply_result(
                    lane,
                    SendResult::Permanent {
                        status: 409,
                        reason: "name_taken".to_string(),
                    },
                    now_ms,
                    jitter_unit,
                )
            }
        }
        SendResult::NamesTaken { adjustments } => {
            apply_names_taken(lane, adjustments, now_ms, jitter_unit)
        }
        SendResult::Permanent { status, reason } => {
            let failed = LaneStatus::Failed {
                reason: reason.clone(),
            };
            halt_with_attempt(
                lane,
                now_ms,
                AttemptOutcome::Permanent { status, reason },
                |_| failed,
            )
        }
    }
}

/// Apply the outcome of sending `message_id` for `file_id`.
///
/// Results for a message that is not the lane's current in-flight message are
/// ignored.
pub fn on_result(
    queue: Queue,
    file_id: &str,
    message_id: &str,
    result: SendResult,
    now_ms: u64,
    jitter_unit: f64,
) -> Queue {
    let is_current = queue.lanes.iter().any(|lane| {
        lane.file_id == file_id
            && lane.in_flight.as_deref() == Some(message_id)
            && lane
                .messages
                .first()
                .is_some_and(|head| head.message_id == message_id)
    });
    if !is_current {
        return queue;
    }
    let is_batch = file_id.starts_with(BATCH_LANE_PREFIX);
    let revisions = match &result {
        // A batch has no revision of its own: it is not a file.
        SendResult::Ok { revision } if !is_batch => {
            set_revision(queue.revisions.clone(), file_id, *revision)
        }
        _ => queue.revisions.clone(),
    };
    let queue = with_lane(queue, file_id, |lane| {
        apply_result(lane, result, now_ms, jitter_unit)
    });
    // A delivered batch leaves nothing behind: its lane exists for that one
    // message only.
    let lanes = queue
        .lanes
        .into_iter()
        .filter(|lane| !(lane.file_id == file_id && is_batch && lane.messages.is_empty()))
        .collect();
    Queue { lanes, revisions }
}

/// Apply a user's resolution to a halted lane.
pub fn resolve(queue: Queue, file_id: &str, resolution: Resolution) -> Queue {
    match resolution {
        Resolution::Retry => with_lane(queue, file_id, |lane| {
            if matches!(
                lane.status,
                LaneStatus::Waiting { .. }
                    | LaneStatus::NeedsMerge { .. }
                    | LaneStatus::Failed { .. }
            ) {
                Lane {
                    status: LaneStatus::Draining,
                    in_flight: None,
                    ..lane
                }
            } else {
                lane
            }
        }),
        Resolution::Discard => Queue {
            lanes: queue
                .lanes
                .into_iter()
                .filter(|lane| lane.file_id != file_id)
                .collect(),
            revisions: queue
                .revisions
                .into_iter()
                .filter(|entry| entry.file_id != file_id)
                .collect(),
        },
        Resolution::MergedAndSave {
            content,
            server_revision,
        } => {
            let needs_merge = queue.lanes.iter().any(|lane| {
                lane.file_id == file_id
                    && matches!(lane.status, LaneStatus::NeedsMerge { .. })
                    && !lane.messages.is_empty()
            });
            if !needs_merge {
                return queue;
            }
            let revisions = set_revision(queue.revisions.clone(), file_id, server_revision);
            let queue = with_lane(queue, file_id, |lane| {
                let mut lane = lane;
                if let Some(head) = lane.messages.first_mut() {
                    head.message = Message::SaveContent { content };
                    head.attempts = Vec::new();
                }
                Lane {
                    status: LaneStatus::Draining,
                    in_flight: None,
                    ..lane
                }
            });
            Queue { revisions, ..queue }
        }
    }
}

/// Every lane waiting for sign-in resumes draining.
pub fn resolve_signed_in(queue: Queue) -> Queue {
    Queue {
        lanes: queue
            .lanes
            .into_iter()
            .map(|lane| {
                if lane.status == LaneStatus::NeedsSignIn {
                    Lane {
                        status: LaneStatus::Draining,
                        ..lane
                    }
                } else {
                    lane
                }
            })
            .collect(),
        revisions: queue.revisions,
    }
}

/// Record the latest known server revision of a file.
pub fn record_revision(queue: Queue, file_id: &str, revision: i64) -> Queue {
    Queue {
        revisions: set_revision(queue.revisions, file_id, revision),
        lanes: queue.lanes,
    }
}
