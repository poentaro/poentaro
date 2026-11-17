use serde::{Deserialize, Serialize};

use super::Outcome;

/// How a request to any route ended, when it ended in a failure. Written
/// once per failed request, by the router (`handlers::fail`), never at a
/// call site.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum RequestEvent {
    RequestFailed(RequestFailed),
}

/// A request was answered with an `ApiError`. `route` is the method and the
/// route's path template (`POST /files/{id}/content`), or the raw path when
/// no route matched; `code` is the `ApiError`'s, and `message` its
/// free-text message when it has one (an `internal` error's cause).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestFailed {
    pub route: String,
    pub status: u16,
    pub code: String,
    #[serde(default)]
    pub message: Option<String>,
}

impl RequestEvent {
    pub(super) fn outcome(&self) -> Outcome {
        match self {
            RequestEvent::RequestFailed(event) if event.status >= 500 => Outcome::Error,
            RequestEvent::RequestFailed(_) => Outcome::Rejected,
        }
    }
}
