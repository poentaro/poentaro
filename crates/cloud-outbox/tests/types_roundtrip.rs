use cloud_outbox::types::*;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn roundtrip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) -> TestResult {
    let json = serde_json::to_string(value)?;
    let back: T = serde_json::from_str(&json)?;
    if back == *value {
        Ok(())
    } else {
        Err(format!("roundtrip mismatch for {json}: got {back:?}").into())
    }
}

fn attempt(outcome: AttemptOutcome) -> Attempt {
    Attempt { at_ms: 5, outcome }
}

#[test]
fn messages_roundtrip() -> TestResult {
    for message in [
        Message::CreateFile {
            name: "a".into(),
            content: "b".into(),
        },
        Message::SaveContent {
            content: "c".into(),
        },
        Message::RenameFile { to: "d".into() },
        Message::RenameFiles {
            moves: vec![RenameMove {
                file_id: "f".into(),
                to: "A/d".into(),
            }],
            folder: None,
        },
        Message::RenameFiles {
            moves: vec![],
            folder: Some(FolderMove {
                from: "A".into(),
                to: "B/A".into(),
            }),
        },
        Message::TrashFile,
        Message::RestoreFile,
    ] {
        roundtrip(&message)?;
    }
    Ok(())
}

#[test]
fn attempt_outcomes_roundtrip() -> TestResult {
    for outcome in [
        AttemptOutcome::Transient { reason: "r".into() },
        AttemptOutcome::Unauthorized,
        AttemptOutcome::RevisionConflict {
            current_revision: 3,
        },
        AttemptOutcome::NameTaken {
            suggested_name: "n".into(),
        },
        AttemptOutcome::NamesTaken {
            adjustments: vec![
                NameAdjustment {
                    rejected: "a".into(),
                    replacement: Some("a 2".into()),
                },
                NameAdjustment {
                    rejected: "b".into(),
                    replacement: None,
                },
            ],
        },
        AttemptOutcome::Permanent {
            status: 500,
            reason: "r".into(),
        },
    ] {
        roundtrip(&attempt(outcome))?;
    }
    Ok(())
}

#[test]
fn queue_with_every_lane_status_roundtrips() -> TestResult {
    let statuses = [
        LaneStatus::Idle,
        LaneStatus::Draining,
        LaneStatus::Waiting { retry_at_ms: 9 },
        LaneStatus::NeedsSignIn,
        LaneStatus::NeedsMerge {
            current_revision: 4,
        },
        LaneStatus::Failed { reason: "x".into() },
    ];
    let lanes = statuses
        .into_iter()
        .map(|status| Lane {
            file_id: "f".into(),
            status,
            in_flight: Some("m".into()),
            messages: vec![QueuedMessage {
                message_id: "m".into(),
                created_at_ms: 1,
                message: Message::TrashFile,
                attempts: vec![attempt(AttemptOutcome::Unauthorized)],
            }],
        })
        .collect();
    roundtrip(&Queue {
        lanes,
        revisions: vec![FileRevision {
            file_id: "f".into(),
            revision: 2,
        }],
    })?;
    Ok(())
}

#[test]
fn send_types_roundtrip() -> TestResult {
    roundtrip(&SendRequest {
        file_id: "f".into(),
        message_id: "m".into(),
        message: Message::RestoreFile,
        expected_revision: 0,
    })?;
    for result in [
        SendResult::Ok { revision: 1 },
        SendResult::Transient { reason: "r".into() },
        SendResult::Unauthorized,
        SendResult::Conflict {
            current_revision: 2,
        },
        SendResult::NameTaken {
            suggested_name: "n".into(),
        },
        SendResult::Permanent {
            status: 400,
            reason: "r".into(),
        },
    ] {
        roundtrip(&result)?;
    }
    for resolution in [
        Resolution::Retry,
        Resolution::Discard,
        Resolution::MergedAndSave {
            content: "c".into(),
            server_revision: 7,
        },
    ] {
        roundtrip(&resolution)?;
    }
    Ok(())
}

#[test]
fn storage_and_merge_types_roundtrip() -> TestResult {
    roundtrip(&StoredRecord {
        key: "k".into(),
        value: "v".into(),
    })?;
    roundtrip(&DecodeError::UnsupportedVersion { found: 2 })?;
    roundtrip(&DecodeError::Corrupt { key: "k".into() })?;
    roundtrip(&MergeOutcome::Clean { text: "t".into() })?;
    roundtrip(&MergeOutcome::Conflicted {
        text: "t".into(),
        has_markers: true,
    })?;
    Ok(())
}

#[test]
fn summary_types_roundtrip() -> TestResult {
    for reason in [
        LaneReason::Syncing,
        LaneReason::WaitingToRetry,
        LaneReason::PossiblyStuck,
        LaneReason::NeedsSignIn,
        LaneReason::NeedsMerge,
        LaneReason::Failed,
    ] {
        roundtrip(&reason)?;
    }
    roundtrip(&QueueSummary {
        lanes: vec![LaneSummary {
            file_id: "f".into(),
            reason: LaneReason::PossiblyStuck,
            pending_messages: 2,
            retry_in_ms: Some(10),
            attempts: vec![attempt(AttemptOutcome::Unauthorized)],
            failure_reason: None,
        }],
        total_pending: 2,
        lanes_needing_attention: 1,
        has_unsynced: true,
    })?;
    Ok(())
}
