use serde::{Deserialize, Serialize};

use super::Outcome;

/// An API key's life, and what an agent did with it (`POST /agent`). A key
/// is named by its `prefix`, never in full.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum AgentEvent {
    KeyCreated(KeyEvent),
    KeyRevoked(KeyEvent),
    /// A request with no key, or one that names no key.
    KeyRejected(KeyRejected),
    /// A request with a valid key.
    Called(Called),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyEvent {
    pub prefix: String,
}

/// `prefix` is the start of what was sent, when it looked like a key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyRejected {
    #[serde(default)]
    pub prefix: Option<String>,
}

/// One request's calls, each as its method and how it ended: `read _: ok`,
/// `write _ at revision _ content _: conflict at revision _ content _`, or
/// `…: internal error: <cause>`. Empty when the request couldn't be read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Called {
    pub prefix: String,
    pub calls: Vec<String>,
    /// Whether a call failed with an internal error.
    #[serde(default)]
    pub failed: bool,
}

impl AgentEvent {
    pub(super) fn outcome(&self) -> Outcome {
        match self {
            AgentEvent::KeyCreated(_) | AgentEvent::KeyRevoked(_) => Outcome::Ok,
            AgentEvent::KeyRejected(_) => Outcome::Rejected,
            AgentEvent::Called(event) if event.failed => Outcome::Error,
            AgentEvent::Called(_) => Outcome::Ok,
        }
    }

    pub(super) fn subject(&self) -> Option<String> {
        let key = |prefix: &str| Some(format!("key:{prefix}"));
        match self {
            AgentEvent::KeyCreated(event) | AgentEvent::KeyRevoked(event) => key(&event.prefix),
            AgentEvent::KeyRejected(event) => event.prefix.as_deref().and_then(key),
            AgentEvent::Called(event) => key(&event.prefix),
        }
    }
}
