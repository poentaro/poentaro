use serde::{Deserialize, Serialize};

use super::Outcome;

/// A request to one of the web app's anyx protocols (`POST /public`,
/// `/app` or `/test`). One row per request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum ProtocolEvent {
    /// A request the endpoint ran.
    Called(ProtocolCalled),
    /// A request refused before it ran.
    Refused(ProtocolRefused),
}

/// `protocol` is the endpoint's (`public`, `app`, `test`); `calls` are the
/// full names of the methods called, never their arguments, which can hold
/// passwords.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolCalled {
    pub protocol: String,
    pub calls: Vec<String>,
    /// Whether a call failed with an internal error.
    #[serde(default)]
    pub failed: bool,
}

/// `status` is what the request was answered: `401` for no live session,
/// `409` for a client built from another schema (its `Poentaro-Schema`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolRefused {
    pub protocol: String,
    pub status: u16,
}

impl ProtocolEvent {
    pub(super) fn outcome(&self) -> Outcome {
        match self {
            ProtocolEvent::Called(event) if event.failed => Outcome::Error,
            ProtocolEvent::Called(_) => Outcome::Ok,
            ProtocolEvent::Refused(_) => Outcome::Rejected,
        }
    }
}
