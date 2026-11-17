use cloud_outbox::history::*;
use cloud_outbox::types::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const DAY_MS: u64 = 24 * 60 * 60 * 1000;

macro_rules! expect {
    ($actual:expr, $expected:expr $(,)?) => {{
        let actual = $actual;
        let expected = $expected;
        if actual == expected {
            Ok::<(), Box<dyn std::error::Error>>(())
        } else {
            Err(format!("expected {expected:?}, got {actual:?}").into())
        }
    }};
}

fn save(content: &str) -> Message {
    Message::SaveContent {
        content: content.into(),
    }
}

fn queued(message_id: &str, created_at_ms: u64, message: Message, tries: usize) -> QueuedMessage {
    QueuedMessage {
        message_id: message_id.into(),
        created_at_ms,
        message,
        attempts: (0..tries)
            .map(|index| Attempt {
                at_ms: created_at_ms + index as u64,
                outcome: AttemptOutcome::Transient {
                    reason: "offline".into(),
                },
            })
            .collect(),
    }
}

fn lane(file_id: &str, status: LaneStatus, messages: Vec<QueuedMessage>) -> Lane {
    Lane {
        file_id: file_id.into(),
        status,
        in_flight: None,
        messages,
    }
}

fn queue_of(lanes: Vec<Lane>) -> Queue {
    Queue {
        lanes,
        revisions: vec![],
    }
}

fn names() -> Vec<FileName> {
    vec![
        FileName {
            file_id: "f1".into(),
            name: "Ode to Joy.poentaro".into(),
        },
        FileName {
            file_id: "f2".into(),
            name: "Canon in D.poentaro".into(),
        },
    ]
}

fn sent_record_for(
    file_id: &str,
    message_id: &str,
    content: &str,
    sent_at_ms: u64,
) -> StoredRecord {
    let queue = queue_of(vec![lane(
        file_id,
        LaneStatus::Draining,
        vec![queued(message_id, sent_at_ms - 5, save(content), 1)],
    )]);
    record_sent(&queue, &[], file_id, message_id, 7, sent_at_ms)
        .put
        .unwrap_or_else(|| StoredRecord {
            key: "missing".into(),
            value: String::new(),
        })
}

#[test]
fn a_delivery_is_recorded_with_its_payload_and_failed_tries() -> TestResult {
    let queue = queue_of(vec![lane(
        "f1",
        LaneStatus::Draining,
        vec![queued("m1", 100, save("1 2 3"), 2)],
    )]);
    let write = record_sent(&queue, &[], "f1", "m1", 9, 500);
    let record = write.put.ok_or("no record")?;
    let expected_key = format!("sent:{:020}:m1", 500);
    expect!(record.key.as_str(), expected_key.as_str())?;
    let rows = rows(
        &queue_of(vec![]),
        &[record],
        &names(),
        &HistoryFilter::everything(),
        600,
    );
    let row = rows.first().ok_or("no row")?;
    expect!(row.status, HistoryStatus::Sent)?;
    expect!(row.revision, Some(9))?;
    expect!(row.attempts.len(), 3)?;
    expect!(
        row.attempts.last().map(|attempt| attempt.outcome.clone()),
        Some(AttemptOutcome::Succeeded)
    )?;
    expect!(row.at_ms, 500)?;
    expect!(row.file_name.as_str(), "Ode to Joy.poentaro")
}

#[test]
fn a_delivery_on_the_first_try_logs_exactly_one_succeeded_attempt() -> TestResult {
    let queue = queue_of(vec![lane(
        "f1",
        LaneStatus::Draining,
        vec![queued("m1", 100, save("1 2 3"), 0)],
    )]);
    let record = record_sent(&queue, &[], "f1", "m1", 9, 500)
        .put
        .ok_or("no record")?;
    let rows = rows(
        &queue_of(vec![]),
        &[record],
        &names(),
        &HistoryFilter::everything(),
        600,
    );
    let row = rows.first().ok_or("no row")?;
    expect!(
        row.attempts.clone(),
        vec![Attempt {
            at_ms: 500,
            outcome: AttemptOutcome::Succeeded
        }]
    )
}

#[test]
fn a_stored_record_whose_log_does_not_end_in_succeeded_is_hidden() -> TestResult {
    let legacy = StoredRecord {
        key: format!("sent:{:020}:old", 500),
        value: r#"{"messageId":"old","fileId":"f1","createdAtMs":100,"sentAtMs":500,"revision":3,"message":{"kind":"saveContent","content":"1 2 3"},"attempts":[]}"#.into(),
    };
    let rows = rows(
        &queue_of(vec![]),
        &[legacy],
        &names(),
        &HistoryFilter::everything(),
        600,
    );
    expect!(rows.len(), 0)
}

#[test]
fn a_delivery_of_an_unknown_message_records_nothing() -> TestResult {
    let write = record_sent(&queue_of(vec![]), &[], "f1", "missing", 1, 500);
    expect!(write.put, None)
}

#[test]
fn records_older_than_the_retention_window_are_deleted() -> TestResult {
    let now = 10 * DAY_MS;
    let old = sent_record_for("f1", "old", "x", now - RETENTION_MS - 1);
    let fresh = sent_record_for("f1", "fresh", "x", now - 1000);
    expect!(expired_keys(&[old.clone(), fresh], now), vec![old.key])
}

#[test]
fn only_the_newest_entries_up_to_the_cap_are_kept() -> TestResult {
    let now = 10 * DAY_MS;
    let records: Vec<_> = (0..MAX_SENT_ENTRIES as u64 + 3)
        .map(|index| sent_record_for("f1", &format!("m{index}"), "x", now - 10_000 + index))
        .collect();
    let deleted = expired_keys(&records, now);
    expect!(deleted.len(), 3)?;
    expect!(
        deleted.contains(&records[0].key) && deleted.contains(&records[2].key),
        true
    )
}

#[test]
fn the_byte_budget_drops_older_entries_but_keeps_the_newest() -> TestResult {
    let now = 10 * DAY_MS;
    let big = "x".repeat(MAX_SENT_BYTES / 2 + 1);
    let oldest = sent_record_for("f1", "a", &big, now - 3000);
    let middle = sent_record_for("f1", "b", &big, now - 2000);
    let newest = sent_record_for("f1", "c", &big, now - 1000);
    let deleted = expired_keys(&[oldest.clone(), middle.clone(), newest], now);
    expect!(deleted, vec![middle.key, oldest.key])
}

#[test]
fn other_keys_are_never_deleted() -> TestResult {
    let foreign = StoredRecord {
        key: "base:f1".into(),
        value: "x".into(),
    };
    expect!(expired_keys(&[foreign], u64::MAX), Vec::<String>::new())
}

#[test]
fn recording_a_delivery_also_reports_the_expired_keys() -> TestResult {
    let now = 10 * DAY_MS;
    let old = sent_record_for("f1", "old", "x", now - RETENTION_MS - 1);
    let queue = queue_of(vec![lane(
        "f1",
        LaneStatus::Draining,
        vec![queued("m1", now - 5, save("y"), 0)],
    )]);
    let write = record_sent(&queue, std::slice::from_ref(&old), "f1", "m1", 2, now);
    expect!(write.deletes, vec![old.key])
}

#[test]
fn queued_messages_show_the_lane_status_on_the_head_only() -> TestResult {
    let queue = queue_of(vec![lane(
        "f1",
        LaneStatus::Waiting { retry_at_ms: 1500 },
        vec![
            queued("m1", 100, save("a"), 2),
            queued("m2", 200, save("b"), 0),
        ],
    )]);
    let found = rows(&queue, &[], &names(), &HistoryFilter::everything(), 1000);
    let statuses: Vec<_> = found
        .iter()
        .map(|row| (row.message_id.as_str(), row.status, row.retry_in_ms))
        .collect();
    expect!(
        statuses,
        vec![
            ("m2", HistoryStatus::Pending, None),
            ("m1", HistoryStatus::Retrying, Some(500)),
        ]
    )
}

fn mixed() -> (Queue, Vec<StoredRecord>) {
    let queue = queue_of(vec![
        lane(
            "f1",
            LaneStatus::NeedsMerge {
                current_revision: 4,
            },
            vec![queued("conflicted", 900, save("5 3 4 5"), 1)],
        ),
        lane(
            "f2",
            LaneStatus::Draining,
            vec![queued(
                "rename",
                950,
                Message::RenameFile {
                    to: "Canon.poentaro".into(),
                },
                0,
            )],
        ),
    ]);
    let sent = vec![
        sent_record_for("f1", "sent-big", &"x".repeat(4000), 800),
        sent_record_for("f2", "sent-small", "1 2 3", 700),
    ];
    (queue, sent)
}

fn ids_with(filter: &HistoryFilter) -> Vec<String> {
    let (queue, sent) = mixed();
    rows(&queue, &sent, &names(), filter, 1000)
        .into_iter()
        .map(|row| row.message_id)
        .collect()
}

#[test]
fn everything_is_listed_newest_first() -> TestResult {
    expect!(
        ids_with(&HistoryFilter::everything()),
        vec!["rename", "conflicted", "sent-big", "sent-small"]
    )
}

#[test]
fn the_status_filter_selects_one_status_or_any_problem() -> TestResult {
    let only_sent = HistoryFilter {
        status: StatusFilter::Only(HistoryStatus::Sent),
        ..HistoryFilter::everything()
    };
    expect!(ids_with(&only_sent), vec!["sent-big", "sent-small"])?;
    let problems = HistoryFilter {
        status: StatusFilter::Problem,
        ..HistoryFilter::everything()
    };
    expect!(ids_with(&problems), vec!["conflicted"])
}

#[test]
fn kind_file_age_size_and_attempt_filters_narrow_the_list() -> TestResult {
    let kind = HistoryFilter {
        kind: Some(MessageKind::Rename),
        ..HistoryFilter::everything()
    };
    expect!(ids_with(&kind), vec!["rename"])?;
    let file = HistoryFilter {
        file_id: Some("f1".into()),
        ..HistoryFilter::everything()
    };
    expect!(ids_with(&file), vec!["conflicted", "sent-big"])?;
    let recent = HistoryFilter {
        max_age_ms: Some(250),
        ..HistoryFilter::everything()
    };
    expect!(ids_with(&recent), vec!["rename", "conflicted", "sent-big"])?;
    let large = HistoryFilter {
        min_size_bytes: Some(1000),
        ..HistoryFilter::everything()
    };
    expect!(ids_with(&large), vec!["sent-big"])?;
    let untried = HistoryFilter {
        attempts: AttemptsFilter::None,
        ..HistoryFilter::everything()
    };
    expect!(ids_with(&untried), vec!["rename"])
}

#[test]
fn the_query_matches_file_names_and_payloads_case_insensitively() -> TestResult {
    let by_name = HistoryFilter {
        query: "CANON".into(),
        ..HistoryFilter::everything()
    };
    expect!(ids_with(&by_name), vec!["rename", "sent-small"])?;
    let by_payload = HistoryFilter {
        query: "5 3 4".into(),
        ..HistoryFilter::everything()
    };
    expect!(ids_with(&by_payload), vec!["conflicted"])
}

#[test]
fn payload_matches_are_flagged_so_the_row_can_open() -> TestResult {
    let (queue, sent) = mixed();
    let filter = HistoryFilter {
        query: "canon".into(),
        ..HistoryFilter::everything()
    };
    let flags: Vec<_> = rows(&queue, &sent, &names(), &filter, 1000)
        .into_iter()
        .map(|row| (row.message_id, row.payload_matches_query))
        .collect();
    expect!(
        flags,
        vec![
            ("rename".to_string(), true),
            ("sent-small".to_string(), false)
        ]
    )
}

#[test]
fn the_payload_is_the_full_message_with_ids() -> TestResult {
    let (queue, sent) = mixed();
    let found = rows(&queue, &sent, &names(), &HistoryFilter::everything(), 1000);
    let row = found
        .iter()
        .find(|row| row.message_id == "rename")
        .ok_or("no row")?;
    let value: serde_json::Value = serde_json::from_str(&row.payload_json)?;
    expect!(value["kind"].as_str(), Some("renameFile"))?;
    expect!(value["to"].as_str(), Some("Canon.poentaro"))?;
    expect!(value["fileId"].as_str(), Some("f2"))
}
