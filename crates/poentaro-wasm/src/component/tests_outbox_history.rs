//! The outbox-history exports called once through the `Guest` impl.
use super::*;

fn no_filter() -> HistoryFilter {
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

fn queue_with_save(content: &str) -> Queue {
    Component::outbox_enqueue(
        Queue {
            lanes: Vec::new(),
            revisions: Vec::new(),
        },
        "file-1".to_string(),
        Message::SaveContent(SaveContentMessage {
            content: content.to_string(),
        }),
        "message-1".to_string(),
        10,
    )
}

#[test]
fn a_queued_message_is_listed_as_pending_with_its_file_name() {
    let rows = Component::outbox_history_rows(
        queue_with_save("1 2 3"),
        Vec::new(),
        vec![FileName {
            file_id: "file-1".to_string(),
            name: "Ode.poentaro".to_string(),
        }],
        no_filter(),
        20,
    );
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].file_name, "Ode.poentaro");
    assert!(matches!(rows[0].status, HistoryStatus::Pending));
}

#[test]
fn a_delivery_is_recorded_and_then_listed_as_sent() {
    let queue = queue_with_save("1 2 3");
    let write = Component::outbox_record_sent(
        queue,
        Vec::new(),
        "file-1".to_string(),
        "message-1".to_string(),
        5,
        30,
    );
    let record = write.put.expect("the message was queued");
    let rows = Component::outbox_history_rows(
        Queue {
            lanes: Vec::new(),
            revisions: Vec::new(),
        },
        vec![record],
        Vec::new(),
        no_filter(),
        40,
    );
    assert_eq!(rows.len(), 1);
    assert!(matches!(rows[0].status, HistoryStatus::Sent));
    assert_eq!(rows[0].revision, Some(5));
}
