//! A folder move is one `RenameFiles` message on a lane of its own: its
//! enqueueing, its ordering against per-file messages, its all-or-nothing
//! name-collision handling and its persistence.
use cloud_outbox::codec::{decode_queue, encode_queue};
use cloud_outbox::enqueue::enqueue;
use cloud_outbox::scheduling::begin_send;
use cloud_outbox::transitions::on_result;
use cloud_outbox::types::*;

type TestResult = Result<(), String>;

#[allow(clippy::needless_pass_by_value)] // call sites pass temporaries
fn check<T: PartialEq + std::fmt::Debug>(actual: T, expected: T) -> TestResult {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("expected {expected:?}, got {actual:?}"))
    }
}

fn empty() -> Queue {
    Queue {
        lanes: vec![],
        revisions: vec![],
    }
}

fn create(name: &str) -> Message {
    Message::CreateFile {
        name: name.into(),
        content: String::new(),
    }
}

fn batch(moves: &[(&str, &str)]) -> Message {
    Message::RenameFiles {
        moves: moves
            .iter()
            .map(|(file_id, to)| RenameMove {
                file_id: (*file_id).into(),
                to: (*to).into(),
            })
            .collect(),
        folder: None,
    }
}

fn rename(to: &str) -> Message {
    Message::RenameFile { to: to.into() }
}

fn next_message_id(queue: Queue) -> Option<String> {
    begin_send(queue, 1_000).request.map(|r| r.message_id)
}

/// Marks the first message of `file_id`'s lane as delivered.
fn deliver(queue: Queue, file_id: &str, message_id: &str) -> Queue {
    on_result(
        queue,
        file_id,
        message_id,
        SendResult::Ok { revision: 1 },
        2_000,
        0.0,
    )
}

fn send_and_get(queue: Queue) -> Result<(Queue, SendRequest), String> {
    let begun = begin_send(queue, 1_000);
    let request = begun.request.ok_or("nothing was sent")?;
    Ok((begun.queue, request))
}

#[test]
fn a_batch_gets_a_lane_of_its_own() -> TestResult {
    let queue = enqueue(empty(), "ignored", batch(&[("a", "X/a")]), "m1", 1);
    check(
        queue
            .lanes
            .iter()
            .map(|l| l.file_id.as_str())
            .collect::<Vec<_>>(),
        vec!["batch:m1"],
    )
}

#[test]
fn a_batch_waits_for_the_creation_of_its_files() -> TestResult {
    let queue = enqueue(empty(), "a", create("A/a"), "create-a", 5);
    let queue = enqueue(queue, "b", create("A/b"), "create-b", 6);
    let queue = enqueue(queue, "", batch(&[("a", "X/a"), ("b", "X/b")]), "move", 7);

    let (queue, first) = send_and_get(queue)?;
    check(first.message_id.as_str(), "create-a")?;
    let queue = deliver(queue, "a", "create-a");
    // b is not created yet: the batch still waits, so b goes first.
    let (queue, second) = send_and_get(queue)?;
    check(second.message_id.as_str(), "create-b")?;
    let queue = deliver(queue, "b", "create-b");
    check(next_message_id(queue), Some("move".into()))
}

#[test]
fn a_batch_is_not_blocked_by_content_saves() -> TestResult {
    let queue = enqueue(
        empty(),
        "a",
        Message::SaveContent {
            content: "x".into(),
        },
        "save",
        9,
    );
    let queue = enqueue(queue, "", batch(&[("a", "X/a")]), "move", 7);
    // The older message is the batch.
    check(next_message_id(queue), Some("move".into()))
}

#[test]
fn a_batch_waits_for_an_earlier_rename_of_its_file() -> TestResult {
    let queue = enqueue(empty(), "a", rename("A/a 2"), "rename", 5);
    let queue = enqueue(queue, "", batch(&[("a", "X/a")]), "move", 7);

    let (queue, first) = send_and_get(queue)?;
    check(first.message_id.as_str(), "rename")?;
    let queue = deliver(queue, "a", "rename");
    check(next_message_id(queue), Some("move".into()))
}

#[test]
fn a_later_rename_waits_for_the_batch() -> TestResult {
    let queue = enqueue(empty(), "", batch(&[("a", "X/a")]), "move", 7);
    let queue = enqueue(queue, "a", rename("X/renamed"), "rename", 8);

    let (queue, first) = send_and_get(queue)?;
    check(first.message_id.as_str(), "move")?;
    // The batch is in flight: the rename still waits.
    check(next_message_id(queue.clone()), None)?;
    let queue = deliver(queue, "batch:move", "move");
    check(next_message_id(queue), Some("rename".into()))
}

#[test]
fn a_rename_made_in_the_same_millisecond_follows_the_batch() -> TestResult {
    let queue = enqueue(empty(), "a", rename("X/renamed"), "rename", 7);
    let queue = enqueue(queue, "", batch(&[("a", "X/a")]), "move", 7);
    check(next_message_id(queue), Some("move".into()))
}

#[test]
fn batches_sharing_a_file_are_sent_in_order() -> TestResult {
    let queue = enqueue(empty(), "", batch(&[("a", "Y/a")]), "second", 9);
    let queue = enqueue(queue, "", batch(&[("a", "X/a")]), "first", 8);
    check(next_message_id(queue), Some("first".into()))
}

#[test]
fn unrelated_batches_do_not_wait_for_each_other() -> TestResult {
    let queue = enqueue(empty(), "", batch(&[("a", "Y/a")]), "second", 9);
    let queue = enqueue(queue, "", batch(&[("b", "X/b")]), "first", 8);
    let (queue, sent) = send_and_get(queue)?;
    check(sent.message_id.as_str(), "first")?;
    check(next_message_id(queue), Some("second".into()))
}

#[test]
fn a_delivered_batch_leaves_no_lane_or_revision() -> TestResult {
    let queue = enqueue(empty(), "", batch(&[("a", "X/a")]), "move", 7);
    let (queue, _) = send_and_get(queue)?;
    let queue = deliver(queue, "batch:move", "move");
    check(queue, empty())
}

fn names_taken(queue: Queue, adjustments: Vec<NameAdjustment>) -> Queue {
    on_result(
        queue,
        "batch:move",
        "move",
        SendResult::NamesTaken { adjustments },
        2_000,
        0.0,
    )
}

fn moves_of(queue: &Queue) -> Vec<RenameMove> {
    match queue.lanes.first().and_then(|lane| lane.messages.first()) {
        Some(QueuedMessage {
            message: Message::RenameFiles { moves, .. },
            ..
        }) => moves.clone(),
        _ => vec![],
    }
}

#[test]
fn a_collision_adjusts_only_the_taken_destinations_and_resends_the_batch() -> TestResult {
    let queue = enqueue(empty(), "", batch(&[("a", "X/a"), ("b", "X/b")]), "move", 7);
    let (queue, _) = send_and_get(queue)?;
    let queue = names_taken(
        queue,
        vec![NameAdjustment {
            rejected: "X/a".into(),
            replacement: Some("X/a 2".into()),
        }],
    );

    check(
        moves_of(&queue),
        vec![
            RenameMove {
                file_id: "a".into(),
                to: "X/a 2".into(),
            },
            RenameMove {
                file_id: "b".into(),
                to: "X/b".into(),
            },
        ],
    )?;
    check(
        queue
            .lanes
            .first()
            .map(|lane| (lane.status.clone(), lane.in_flight.clone())),
        Some((LaneStatus::Draining, None)),
    )?;
    // The whole batch goes out again.
    check(next_message_id(queue), Some("move".into()))
}

#[test]
fn a_dropped_move_leaves_the_rest_of_the_batch() -> TestResult {
    let queue = enqueue(
        empty(),
        "",
        batch(&[("a", "X/a"), ("k", "X/.keep")]),
        "move",
        7,
    );
    let (queue, _) = send_and_get(queue)?;
    let queue = names_taken(
        queue,
        vec![NameAdjustment {
            rejected: "X/.keep".into(),
            replacement: None,
        }],
    );
    check(
        moves_of(&queue),
        vec![RenameMove {
            file_id: "a".into(),
            to: "X/a".into(),
        }],
    )
}

#[test]
fn dropping_every_move_completes_the_batch() -> TestResult {
    let queue = enqueue(empty(), "", batch(&[("k", "X/.keep")]), "move", 7);
    let (queue, _) = send_and_get(queue)?;
    let queue = names_taken(
        queue,
        vec![NameAdjustment {
            rejected: "X/.keep".into(),
            replacement: None,
        }],
    );
    check(queue, empty())
}

#[test]
fn a_collision_that_matches_no_move_fails_the_batch_instead_of_looping() -> TestResult {
    let queue = enqueue(empty(), "", batch(&[("a", "X/a")]), "move", 7);
    let (queue, _) = send_and_get(queue)?;
    let queue = names_taken(
        queue,
        vec![NameAdjustment {
            rejected: "elsewhere".into(),
            replacement: Some("elsewhere 2".into()),
        }],
    );
    check(
        queue.lanes.first().map(|lane| lane.status.clone()),
        Some(LaneStatus::Failed {
            reason: "name_taken".into(),
        }),
    )?;
    check(moves_of(&queue).len(), 1)
}

fn head_message(queue: &Queue) -> Option<Message> {
    queue
        .lanes
        .first()
        .and_then(|lane| lane.messages.first())
        .map(|queued| queued.message.clone())
}

#[test]
fn a_taken_destination_folder_moves_the_whole_batch_to_the_replacement() -> TestResult {
    let folder_move = Message::RenameFiles {
        moves: vec![RenameMove {
            file_id: "a".into(),
            to: "New/a".into(),
        }],
        folder: Some(FolderMove {
            from: "Old".into(),
            to: "New".into(),
        }),
    };
    let queue = enqueue(empty(), "", folder_move, "move", 7);
    let (queue, _) = send_and_get(queue)?;
    let queue = names_taken(
        queue,
        vec![NameAdjustment {
            rejected: "New".into(),
            replacement: Some("New 2".into()),
        }],
    );

    check(
        head_message(&queue),
        Some(Message::RenameFiles {
            moves: vec![RenameMove {
                file_id: "a".into(),
                to: "New 2/a".into(),
            }],
            folder: Some(FolderMove {
                from: "Old".into(),
                to: "New 2".into(),
            }),
        }),
    )
}

#[test]
fn a_queued_batch_survives_encoding() -> TestResult {
    let queue = enqueue(empty(), "", batch(&[("a", "X/a")]), "move", 7);
    let decoded = decode_queue(&encode_queue(&queue)).map_err(|e| format!("{e:?}"))?;
    check(decoded, queue)
}
