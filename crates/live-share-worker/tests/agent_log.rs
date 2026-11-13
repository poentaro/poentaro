//! Row shapes of the `app_log` events `/api-keys` and `POST /agent` write.
#![allow(clippy::disallowed_macros)]

use live_share_worker::app_log::{AgentEvent, Called, KeyEvent, KeyRejected, LogEvent, Outcome};

fn called(failed: bool) -> LogEvent {
    LogEvent::Agent(AgentEvent::Called(Called {
        prefix: "poe_AbCd1234".to_string(),
        calls: vec![
            "read _: ok".to_string(),
            "write _ at revision _ content _: rejected _".to_string(),
        ],
        failed,
    }))
}

#[test]
fn a_call_row_names_the_key_by_its_prefix() -> Result<(), serde_json::Error> {
    let row = called(false).to_row()?;

    assert_eq!(row.flow, "agent");
    assert_eq!(row.kind, "called");
    assert_eq!(row.outcome, Outcome::Ok);
    assert_eq!(row.subject.as_deref(), Some("key:poe_AbCd1234"));
    Ok(())
}

#[test]
fn a_request_with_an_internal_error_is_an_error() -> Result<(), serde_json::Error> {
    assert_eq!(called(true).to_row()?.outcome, Outcome::Error);
    Ok(())
}

#[test]
fn a_refused_key_is_rejected_and_named_only_when_it_looked_like_one(
) -> Result<(), serde_json::Error> {
    let refused = |prefix: Option<&str>| {
        LogEvent::Agent(AgentEvent::KeyRejected(KeyRejected {
            prefix: prefix.map(str::to_string),
        }))
        .to_row()
    };

    let named = refused(Some("poe_AbCd1234"))?;
    let unnamed = refused(None)?;

    assert_eq!(named.outcome, Outcome::Rejected);
    assert_eq!(named.subject.as_deref(), Some("key:poe_AbCd1234"));
    assert_eq!(unnamed.subject, None);
    Ok(())
}

#[test]
fn an_expired_key_is_rejected_and_named() -> Result<(), serde_json::Error> {
    let row = LogEvent::Agent(AgentEvent::KeyExpired(KeyEvent {
        prefix: "poe_AbCd1234".to_string(),
    }))
    .to_row()?;

    assert_eq!(row.kind, "key_expired");
    assert_eq!(row.outcome, Outcome::Rejected);
    assert_eq!(row.subject.as_deref(), Some("key:poe_AbCd1234"));
    Ok(())
}

#[test]
fn every_agent_event_decodes_back() -> Result<(), serde_json::Error> {
    let key = KeyEvent {
        prefix: "poe_AbCd1234".to_string(),
    };
    let events = [
        LogEvent::Agent(AgentEvent::KeyCreated(key.clone())),
        LogEvent::Agent(AgentEvent::KeyRevoked(key.clone())),
        LogEvent::Agent(AgentEvent::KeyExpired(key)),
        LogEvent::Agent(AgentEvent::KeyRejected(KeyRejected { prefix: None })),
        called(true),
    ];

    for event in events {
        assert_eq!(LogEvent::try_from(&event.to_row()?)?, event);
    }
    Ok(())
}
