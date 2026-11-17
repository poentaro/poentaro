//! Each cloud-outbox export called once through the `Guest` impl.
use super::*;

fn empty_queue() -> Queue {
    Queue {
        lanes: Vec::new(),
        revisions: Vec::new(),
    }
}

fn queue_with_save(content: &str) -> Queue {
    Component::outbox_enqueue(
        empty_queue(),
        "file-1".to_string(),
        Message::SaveContent(SaveContentMessage {
            content: content.to_string(),
        }),
        "message-1".to_string(),
        10,
    )
}

#[test]
fn enqueue_creates_a_lane_with_the_message() {
    let queue = queue_with_save("1 2 3");
    assert_eq!(queue.lanes.len(), 1);
    assert_eq!(queue.lanes[0].messages.len(), 1);
}

#[test]
fn begin_send_marks_the_head_in_flight() {
    let begun = Component::outbox_begin_send(queue_with_save("1 2 3"), 20);
    assert_eq!(
        begun.request.map(|request| request.message_id),
        Some("message-1".to_string())
    );
    assert_eq!(begun.queue.lanes[0].in_flight.as_deref(), Some("message-1"));
}

#[test]
fn next_wake_is_none_without_waiting_lanes() {
    assert_eq!(Component::outbox_next_wake_ms(queue_with_save("1")), None);
}

#[test]
fn a_transient_result_makes_the_lane_wait_and_appear_in_the_summary() {
    let begun = Component::outbox_begin_send(queue_with_save("1 2 3"), 20);
    let queue = Component::outbox_on_result(
        begun.queue,
        "file-1".to_string(),
        "message-1".to_string(),
        SendResult::Transient(TransientReason {
            reason: "offline".to_string(),
        }),
        30,
        0.0,
    );
    assert!(Component::outbox_next_wake_ms(queue.clone()).is_some());
    let summary = Component::outbox_summarize(queue, 31);
    assert_eq!(summary.total_pending, 1);
    assert!(summary.has_unsynced);
}

#[test]
fn resolve_discard_drops_the_lane() {
    let queue = Component::outbox_resolve(
        queue_with_save("1"),
        "file-1".to_string(),
        Resolution::Discard,
    );
    assert!(queue.lanes.is_empty());
}

#[test]
fn resolve_signed_in_leaves_other_lanes_alone() {
    let queue = Component::outbox_resolve_signed_in(queue_with_save("1"));
    assert_eq!(queue.lanes.len(), 1);
}

#[test]
fn record_revision_is_kept_in_the_queue() {
    let queue = Component::outbox_record_revision(empty_queue(), "file-1".to_string(), 7);
    assert_eq!(queue.revisions.len(), 1);
    assert_eq!(queue.revisions[0].revision, 7);
}

#[test]
fn encode_then_decode_round_trips_the_queue() {
    let records = Component::outbox_encode_queue(queue_with_save("1 2 3"));
    let decoded = Component::outbox_decode_queue(records).ok();
    assert_eq!(decoded.map(|queue| queue.lanes.len()), Some(1));
}

#[test]
fn decode_rejects_an_unsupported_version() {
    let records = vec![StoredRecord {
        key: "meta:version".to_string(),
        value: "999".to_string(),
    }];
    assert!(matches!(
        Component::outbox_decode_queue(records),
        Err(DecodeError::UnsupportedVersion(_))
    ));
}

#[test]
fn merge_three_way_merges_edits_on_different_lines() {
    let outcome = Component::merge_three_way(
        "score \"\"\"\n[M] 1\n\n[M] 2\n\n[M] 3\n\"\"\"\n".to_string(),
        "score \"\"\"\n[M] 5\n\n[M] 2\n\n[M] 3\n\"\"\"\n".to_string(),
        "score \"\"\"\n[M] 1\n\n[M] 2\n\n[M] 6\n\"\"\"\n".to_string(),
    );
    assert!(matches!(outcome, MergeOutcome::Clean(_)));
}

#[test]
fn merge_three_way_reports_markers_for_the_same_line() {
    let outcome = Component::merge_three_way(
        "a\nb\n".to_string(),
        "x\nb\n".to_string(),
        "y\nb\n".to_string(),
    );
    assert!(matches!(
        outcome,
        MergeOutcome::Conflicted(MergeConflictedDetails {
            has_markers: true,
            ..
        })
    ));
}
